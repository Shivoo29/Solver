use anyhow::Result;
use async_trait::async_trait;
use solver_core::{BrowserPlugin, BrowserEvent, BrowserCore, PluginMetadata};
use solver_plugins::plugin_metadata;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

mod tab_suspension;
mod intelligent_preload;
mod battery_aware;
mod performance_metrics;

pub use tab_suspension::{TabSuspender, TabState};
pub use intelligent_preload::IntelligentPreloader;
pub use battery_aware::{BatteryMonitor, BatteryState, PowerMode};
pub use performance_metrics::{PerformanceMetrics, ResourceUsage};

/// Performance mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceMode {
    Maximum,     // Best performance, use all resources
    Balanced,    // Balance performance and battery
    PowerSaver,  // Optimize for battery life
    Aggressive,  // Maximum battery saving, may impact UX
}

/// Performance & Battery Plugin
///
/// Provides aggressive optimization:
/// - Tab suspension after inactivity
/// - Intelligent resource preloading
/// - Battery-aware rendering throttling
/// - Memory pressure handling
/// - Performance metrics
pub struct PerformancePlugin {
    metadata: PluginMetadata,
    mode: PerformanceMode,
    tab_suspender: Arc<RwLock<TabSuspender>>,
    preloader: Arc<RwLock<IntelligentPreloader>>,
    battery_monitor: Arc<RwLock<BatteryMonitor>>,
    metrics: Arc<RwLock<PerformanceMetrics>>,
    enabled: bool,
}

impl PerformancePlugin {
    pub fn new() -> Self {
        Self {
            metadata: plugin_metadata!(
                "Performance & Battery",
                "0.1.0",
                "Aggressive performance optimization and battery saving",
                "Solver Team"
            ),
            mode: PerformanceMode::Balanced,
            tab_suspender: Arc::new(RwLock::new(TabSuspender::new())),
            preloader: Arc::new(RwLock::new(IntelligentPreloader::new())),
            battery_monitor: Arc::new(RwLock::new(BatteryMonitor::new())),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::new())),
            enabled: true,
        }
    }

    pub fn with_mode(mut self, mode: PerformanceMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn set_mode(&mut self, mode: PerformanceMode) {
        self.mode = mode;
        eprintln!("[Performance] Switched to {:?} mode", mode);

        // Adjust settings based on mode
        let mut suspender = self.tab_suspender.write();
        match mode {
            PerformanceMode::Maximum => {
                suspender.set_timeout(Duration::from_secs(3600)); // Never suspend
            }
            PerformanceMode::Balanced => {
                suspender.set_timeout(Duration::from_secs(300)); // 5 minutes
            }
            PerformanceMode::PowerSaver => {
                suspender.set_timeout(Duration::from_secs(60)); // 1 minute
            }
            PerformanceMode::Aggressive => {
                suspender.set_timeout(Duration::from_secs(30)); // 30 seconds
            }
        }
    }

    /// Auto-adjust mode based on battery
    fn auto_adjust_mode(&mut self) {
        let (battery_state, new_mode) = {
            let battery = self.battery_monitor.read();

            let new_mode = match battery.state() {
                BatteryState::Charging => PerformanceMode::Maximum,
                BatteryState::Discharging(level) if level > 50.0 => PerformanceMode::Balanced,
                BatteryState::Discharging(level) if level > 20.0 => PerformanceMode::PowerSaver,
                BatteryState::Discharging(_) => PerformanceMode::Aggressive,
                BatteryState::Unknown => PerformanceMode::Balanced,
            };

            (battery.state(), new_mode)
        };

        if new_mode != self.mode {
            eprintln!("[Performance] Auto-adjusting to {:?} (battery: {:?})",
                      new_mode, battery_state);
            self.set_mode(new_mode);
        }
    }
}

impl Default for PerformancePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BrowserPlugin for PerformancePlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn init(&mut self, _core: &mut BrowserCore) -> Result<()> {
        eprintln!("[Performance] Initializing in {:?} mode...", self.mode);

        // Initialize battery monitoring
        {
            let mut battery = self.battery_monitor.write();
            battery.start_monitoring()?;
        }

        eprintln!("[Performance] ✓ Tab suspension enabled");
        eprintln!("[Performance] ✓ Intelligent preload enabled");
        eprintln!("[Performance] ✓ Battery monitoring enabled");

        Ok(())
    }

    async fn handle_event(&mut self, event: BrowserEvent, core: &mut BrowserCore) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Record event timing
        let start = Instant::now();

        match event {
            BrowserEvent::PageLoadStart { url } => {
                eprintln!("[Performance] Page load start: {}", url);

                // Check if should preload resources
                let preloader = self.preloader.write();
                if let Some(resources) = preloader.get_predicted_resources(&url) {
                    eprintln!("[Performance] Preloading {} predicted resources", resources.len());

                    for resource in resources {
                        core.emit_event(BrowserEvent::NetworkRequest {
                            url: resource,
                            method: "GET".to_string(),
                            headers: vec![],
                        });
                    }
                }

                // Mark tab as active
                let mut suspender = self.tab_suspender.write();
                suspender.mark_active(&url);
            }

            BrowserEvent::HtmlFetched { url, html } => {
                // Record page load for preloading predictions
                let mut preloader = self.preloader.write();
                preloader.record_page_load(&url, &html);

                // Update metrics
                let mut metrics = self.metrics.write();
                metrics.record_page_load(start.elapsed());
            }

            BrowserEvent::Custom { name, data } if name == "TabInactive" => {
                // Tab became inactive, consider suspending
                let mut suspender = self.tab_suspender.write();
                suspender.mark_inactive(&data);

                eprintln!("[Performance] Tab marked inactive: {}", data);
            }

            BrowserEvent::Custom { name, data } if name == "TabActive" => {
                // Tab became active, resume if suspended
                let mut suspender = self.tab_suspender.write();
                suspender.mark_active(&data);

                if suspender.is_suspended(&data) {
                    eprintln!("[Performance] Resuming tab: {}", data);
                    suspender.resume(&data);

                    core.emit_event(BrowserEvent::Custom {
                        name: "TabResumed".to_string(),
                        data: data.clone(),
                    });
                }
            }

            BrowserEvent::Custom { name, data: _ } if name == "CheckSuspend" => {
                // Periodic check for tabs to suspend
                let mut suspender = self.tab_suspender.write();
                let to_suspend = suspender.check_for_suspension();

                for url in to_suspend {
                    eprintln!("[Performance] 💤 Suspending tab: {}", url);
                    suspender.suspend(&url);

                    core.emit_event(BrowserEvent::Custom {
                        name: "TabSuspended".to_string(),
                        data: url,
                    });
                }
            }

            BrowserEvent::Custom { name, data: _ } if name == "GetPerformanceStats" => {
                // Auto-adjust based on battery
                self.auto_adjust_mode();

                let metrics = self.metrics.read();
                let battery = self.battery_monitor.read();
                let suspender = self.tab_suspender.read();

                let stats = serde_json::json!({
                    "mode": format!("{:?}", self.mode),
                    "battery": {
                        "state": format!("{:?}", battery.state()),
                        "level": battery.level(),
                        "charging": battery.is_charging(),
                    },
                    "tabs": {
                        "suspended": suspender.suspended_count(),
                        "active": suspender.active_count(),
                    },
                    "metrics": {
                        "avg_page_load_ms": metrics.avg_page_load_ms(),
                        "total_memory_mb": metrics.total_memory_mb(),
                    }
                });

                core.emit_event(BrowserEvent::Custom {
                    name: "PerformanceStats".to_string(),
                    data: stats.to_string(),
                });
            }

            _ => {}
        }

        Ok(())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
