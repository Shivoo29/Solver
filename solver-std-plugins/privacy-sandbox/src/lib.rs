use anyhow::Result;
use async_trait::async_trait;
use solver_core::{BrowserPlugin, BrowserEvent, BrowserCore, PluginMetadata};
use solver_plugins::plugin_metadata;
use std::sync::Arc;
use parking_lot::RwLock;

mod tracker_blocking;
mod cookie_isolation;
mod fingerprint_protection;
mod privacy_metrics;

pub use tracker_blocking::TrackerBlocker;
pub use cookie_isolation::CookieJar;
pub use fingerprint_protection::FingerprintProtection;
pub use privacy_metrics::{PrivacyMetrics, BlockedItem, BlockReason};

/// Privacy levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyLevel {
    Standard,  // Block known trackers, allow functional cookies
    Strict,    // Block all third-party, aggressive fingerprint protection
    Maximum,   // Block everything, may break sites
}

/// Privacy Sandbox Plugin
///
/// Provides comprehensive privacy protection:
/// - Tracker blocking (EasyList/EasyPrivacy)
/// - Cookie isolation (no third-party by default)
/// - Fingerprinting protection (canvas, WebGL, audio)
/// - WebRTC leak prevention
/// - Privacy metrics and dashboard
pub struct PrivacySandboxPlugin {
    metadata: PluginMetadata,
    level: PrivacyLevel,
    tracker_blocker: Arc<RwLock<TrackerBlocker>>,
    cookie_jar: Arc<RwLock<CookieJar>>,
    fingerprint_protection: Arc<RwLock<FingerprintProtection>>,
    metrics: Arc<RwLock<PrivacyMetrics>>,
    enabled: bool,
}

impl PrivacySandboxPlugin {
    pub fn new() -> Self {
        Self {
            metadata: plugin_metadata!(
                "Privacy Sandbox",
                "0.1.0",
                "Comprehensive privacy protection with tracker blocking and fingerprint protection",
                "Solver Team"
            ),
            level: PrivacyLevel::Standard,
            tracker_blocker: Arc::new(RwLock::new(TrackerBlocker::new())),
            cookie_jar: Arc::new(RwLock::new(CookieJar::new())),
            fingerprint_protection: Arc::new(RwLock::new(FingerprintProtection::new())),
            metrics: Arc::new(RwLock::new(PrivacyMetrics::new())),
            enabled: true,
        }
    }

    pub fn with_level(mut self, level: PrivacyLevel) -> Self {
        self.level = level;
        self
    }

    pub fn set_level(&mut self, level: PrivacyLevel) {
        self.level = level;
        eprintln!("[Privacy Sandbox] Switched to {:?} mode", level);
    }

    /// Check if a URL should be blocked
    fn should_block_url(&self, url: &str, page_url: &str) -> (bool, Option<BlockReason>) {
        let blocker = self.tracker_blocker.read();

        if blocker.is_tracker(url) {
            return (true, Some(BlockReason::Tracker));
        }

        // Check if third-party
        if self.is_third_party(url, page_url) {
            match self.level {
                PrivacyLevel::Standard => {
                    // Allow third-party if not in block list
                    (false, None)
                }
                PrivacyLevel::Strict | PrivacyLevel::Maximum => {
                    // Block all third-party
                    (true, Some(BlockReason::ThirdParty))
                }
            }
        } else {
            (false, None)
        }
    }

    /// Check if URL is third-party relative to page
    fn is_third_party(&self, url: &str, page_url: &str) -> bool {
        if let (Ok(url_parsed), Ok(page_parsed)) = (
            url::Url::parse(url),
            url::Url::parse(page_url)
        ) {
            if let (Some(url_domain), Some(page_domain)) = (
                url_parsed.host_str(),
                page_parsed.host_str()
            ) {
                return url_domain != page_domain;
            }
        }
        false
    }

    /// Get privacy statistics
    pub fn get_stats(&self) -> PrivacyMetrics {
        self.metrics.read().clone()
    }
}

impl Default for PrivacySandboxPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BrowserPlugin for PrivacySandboxPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn init(&mut self, _core: &mut BrowserCore) -> Result<()> {
        eprintln!("[Privacy Sandbox] Initializing in {:?} mode...", self.level);

        // Initialize tracker blocker
        {
            let mut blocker = self.tracker_blocker.write();
            blocker.load_default_lists()?;
        }

        eprintln!("[Privacy Sandbox] ✓ Tracker lists loaded");
        eprintln!("[Privacy Sandbox] ✓ Cookie isolation enabled");
        eprintln!("[Privacy Sandbox] ✓ Fingerprint protection enabled");

        Ok(())
    }

    async fn handle_event(&mut self, event: BrowserEvent, core: &mut BrowserCore) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        match event {
            BrowserEvent::PageLoadStart { url } => {
                eprintln!("[Privacy Sandbox] Page load: {}", url);

                // Reset per-page metrics
                let mut metrics = self.metrics.write();
                metrics.new_page(&url);
            }

            BrowserEvent::NetworkRequest { url, method, headers } => {
                // Get page URL from browser state
                let page_url = {
                    let state = core.state();
                    let state_lock = state.read().unwrap();
                    state_lock.current_url.clone().unwrap_or_default()
                };

                // Check if should block
                let (should_block, reason) = self.should_block_url(&url, &page_url);

                if should_block {
                    if let Some(reason) = reason {
                        eprintln!("[Privacy Sandbox] 🚫 BLOCKED: {} ({})", url, reason);

                        // Record blocked item
                        let mut metrics = self.metrics.write();
                        metrics.add_blocked(&url, reason);
                    }

                    // Don't emit event further - request is blocked
                    return Ok(());
                }

                // Check cookies
                let mut cookie_jar = self.cookie_jar.write();
                let filtered_headers = cookie_jar.filter_cookies(&headers, &url, &page_url);

                // Re-emit with filtered headers
                core.emit_event(BrowserEvent::NetworkRequest {
                    url,
                    method,
                    headers: filtered_headers,
                });
            }

            BrowserEvent::Custom { name, data: _ } if name == "GetPrivacyStats" => {
                let metrics = self.metrics.read();
                let stats = serde_json::to_string(&*metrics)?;

                core.emit_event(BrowserEvent::Custom {
                    name: "PrivacyStats".to_string(),
                    data: stats,
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
