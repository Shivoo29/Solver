use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Tab suspension manager
pub struct TabSuspender {
    tabs: HashMap<String, TabInfo>,
    suspension_timeout: Duration,
}

#[derive(Debug, Clone)]
struct TabInfo {
    url: String,
    state: TabState,
    last_active: Instant,
    memory_usage_mb: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabState {
    Active,
    Inactive,
    Suspended,
}

impl TabSuspender {
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            suspension_timeout: Duration::from_secs(300), // 5 minutes default
        }
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.suspension_timeout = timeout;
    }

    /// Mark tab as active
    pub fn mark_active(&mut self, url: &str) {
        if let Some(tab) = self.tabs.get_mut(url) {
            tab.state = TabState::Active;
            tab.last_active = Instant::now();
        } else {
            self.tabs.insert(url.to_string(), TabInfo {
                url: url.to_string(),
                state: TabState::Active,
                last_active: Instant::now(),
                memory_usage_mb: 0.0,
            });
        }
    }

    /// Mark tab as inactive
    pub fn mark_inactive(&mut self, url: &str) {
        if let Some(tab) = self.tabs.get_mut(url) {
            if tab.state == TabState::Active {
                tab.state = TabState::Inactive;
                tab.last_active = Instant::now();
            }
        }
    }

    /// Check if tab is suspended
    pub fn is_suspended(&self, url: &str) -> bool {
        self.tabs.get(url)
            .map(|t| t.state == TabState::Suspended)
            .unwrap_or(false)
    }

    /// Suspend a tab
    pub fn suspend(&mut self, url: &str) {
        if let Some(tab) = self.tabs.get_mut(url) {
            if tab.state != TabState::Suspended {
                tab.state = TabState::Suspended;
                // In real implementation: Free memory, stop timers, etc.
            }
        }
    }

    /// Resume a suspended tab
    pub fn resume(&mut self, url: &str) {
        if let Some(tab) = self.tabs.get_mut(url) {
            if tab.state == TabState::Suspended {
                tab.state = TabState::Active;
                tab.last_active = Instant::now();
                // In real implementation: Restore memory, restart timers, etc.
            }
        }
    }

    /// Check for tabs that should be suspended
    pub fn check_for_suspension(&mut self) -> Vec<String> {
        let now = Instant::now();
        let mut to_suspend = Vec::new();

        for (url, tab) in &self.tabs {
            if tab.state == TabState::Inactive {
                let inactive_duration = now.duration_since(tab.last_active);
                if inactive_duration >= self.suspension_timeout {
                    to_suspend.push(url.clone());
                }
            }
        }

        to_suspend
    }

    /// Get count of suspended tabs
    pub fn suspended_count(&self) -> usize {
        self.tabs.values()
            .filter(|t| t.state == TabState::Suspended)
            .count()
    }

    /// Get count of active tabs
    pub fn active_count(&self) -> usize {
        self.tabs.values()
            .filter(|t| t.state == TabState::Active)
            .count()
    }

    /// Get memory savings from suspended tabs
    pub fn memory_saved_mb(&self) -> f64 {
        self.tabs.values()
            .filter(|t| t.state == TabState::Suspended)
            .map(|t| t.memory_usage_mb)
            .sum()
    }

    /// Get all tab states
    pub fn get_all_states(&self) -> Vec<(String, TabState)> {
        self.tabs.iter()
            .map(|(url, tab)| (url.clone(), tab.state))
            .collect()
    }
}

impl Default for TabSuspender {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_suspension() {
        let mut suspender = TabSuspender::new();
        suspender.set_timeout(Duration::from_millis(100));

        // Mark tab as active
        suspender.mark_active("https://example.com");
        assert_eq!(suspender.active_count(), 1);

        // Mark as inactive
        suspender.mark_inactive("https://example.com");
        assert_eq!(suspender.active_count(), 0);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));

        // Should be ready for suspension
        let to_suspend = suspender.check_for_suspension();
        assert_eq!(to_suspend.len(), 1);

        // Suspend
        suspender.suspend(&to_suspend[0]);
        assert_eq!(suspender.suspended_count(), 1);

        // Resume
        suspender.resume("https://example.com");
        assert_eq!(suspender.suspended_count(), 0);
        assert_eq!(suspender.active_count(), 1);
    }
}
