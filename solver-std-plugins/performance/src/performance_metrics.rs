use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Performance metrics tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    // Page load metrics
    total_page_loads: u64,
    total_page_load_time_ms: u64,

    // Resource usage
    current_memory_mb: f64,
    peak_memory_mb: f64,

    // Tab metrics
    tabs_suspended: u64,
    memory_saved_mb: f64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_page_loads: 0,
            total_page_load_time_ms: 0,
            current_memory_mb: 0.0,
            peak_memory_mb: 0.0,
            tabs_suspended: 0,
            memory_saved_mb: 0.0,
        }
    }

    /// Record a page load
    pub fn record_page_load(&mut self, duration: Duration) {
        self.total_page_loads += 1;
        self.total_page_load_time_ms += duration.as_millis() as u64;
    }

    /// Record memory usage
    pub fn record_memory_usage(&mut self, mb: f64) {
        self.current_memory_mb = mb;
        if mb > self.peak_memory_mb {
            self.peak_memory_mb = mb;
        }
    }

    /// Record tab suspension
    pub fn record_tab_suspension(&mut self, memory_saved_mb: f64) {
        self.tabs_suspended += 1;
        self.memory_saved_mb += memory_saved_mb;
    }

    /// Get average page load time
    pub fn avg_page_load_ms(&self) -> u64 {
        if self.total_page_loads == 0 {
            0
        } else {
            self.total_page_load_time_ms / self.total_page_loads
        }
    }

    /// Get total memory usage
    pub fn total_memory_mb(&self) -> f64 {
        self.current_memory_mb
    }

    /// Get memory saved by tab suspension
    pub fn memory_saved(&self) -> f64 {
        self.memory_saved_mb
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "⚡ Performance: {}ms avg load, {:.1}MB memory, {} tabs suspended",
            self.avg_page_load_ms(),
            self.current_memory_mb,
            self.tabs_suspended
        )
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_mb: f64,
    pub cpu_percent: f32,
    pub network_kbps: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics() {
        let mut metrics = PerformanceMetrics::new();

        // Record some page loads
        metrics.record_page_load(Duration::from_millis(100));
        metrics.record_page_load(Duration::from_millis(200));
        metrics.record_page_load(Duration::from_millis(300));

        assert_eq!(metrics.avg_page_load_ms(), 200);
        assert_eq!(metrics.total_page_loads, 3);

        // Record memory
        metrics.record_memory_usage(100.0);
        metrics.record_memory_usage(150.0);

        assert_eq!(metrics.total_memory_mb(), 150.0);
        assert_eq!(metrics.peak_memory_mb, 150.0);

        // Record suspensions
        metrics.record_tab_suspension(50.0);
        metrics.record_tab_suspension(30.0);

        assert_eq!(metrics.memory_saved(), 80.0);
    }
}
