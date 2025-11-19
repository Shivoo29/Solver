use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Privacy metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyMetrics {
    // Per-page metrics
    current_page: String,

    // Blocked items
    blocked_items: Vec<BlockedItem>,

    // Totals
    pub total_trackers_blocked: usize,
    pub total_cookies_blocked: usize,
    pub total_fingerprints_blocked: usize,

    // Per-category breakdown
    pub blocked_by_category: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedItem {
    pub url: String,
    pub reason: BlockReason,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockReason {
    Tracker,
    ThirdParty,
    Cookie,
    Fingerprint,
    WebRTC,
}

impl std::fmt::Display for BlockReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockReason::Tracker => write!(f, "Tracker"),
            BlockReason::ThirdParty => write!(f, "Third-party"),
            BlockReason::Cookie => write!(f, "Cookie"),
            BlockReason::Fingerprint => write!(f, "Fingerprint"),
            BlockReason::WebRTC => write!(f, "WebRTC"),
        }
    }
}

impl PrivacyMetrics {
    pub fn new() -> Self {
        Self {
            current_page: String::new(),
            blocked_items: Vec::new(),
            total_trackers_blocked: 0,
            total_cookies_blocked: 0,
            total_fingerprints_blocked: 0,
            blocked_by_category: HashMap::new(),
        }
    }

    /// Start tracking for a new page
    pub fn new_page(&mut self, url: &str) {
        self.current_page = url.to_string();
        self.blocked_items.clear();
    }

    /// Record a blocked item
    pub fn add_blocked(&mut self, url: &str, reason: BlockReason) {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.blocked_items.push(BlockedItem {
            url: url.to_string(),
            reason,
            timestamp,
        });

        // Update totals
        match reason {
            BlockReason::Tracker => self.total_trackers_blocked += 1,
            BlockReason::Cookie => self.total_cookies_blocked += 1,
            BlockReason::Fingerprint => self.total_fingerprints_blocked += 1,
            BlockReason::ThirdParty => self.total_trackers_blocked += 1,
            BlockReason::WebRTC => self.total_trackers_blocked += 1,
        }

        // Update category
        let category = reason.to_string();
        *self.blocked_by_category.entry(category).or_insert(0) += 1;
    }

    /// Get blocked items for current page
    pub fn current_page_blocks(&self) -> &[BlockedItem] {
        &self.blocked_items
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "🛡️  Privacy Stats: {} trackers, {} cookies, {} fingerprints blocked",
            self.total_trackers_blocked,
            self.total_cookies_blocked,
            self.total_fingerprints_blocked
        )
    }

    /// Get detailed report
    pub fn detailed_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Privacy Report for: {}\n", self.current_page));
        report.push_str(&"=".repeat(60));
        report.push_str("\n\n");

        report.push_str("Total Blocked:\n");
        report.push_str(&format!("  Trackers: {}\n", self.total_trackers_blocked));
        report.push_str(&format!("  Cookies: {}\n", self.total_cookies_blocked));
        report.push_str(&format!("  Fingerprints: {}\n", self.total_fingerprints_blocked));
        report.push_str("\n");

        if !self.blocked_items.is_empty() {
            report.push_str("Recent Blocks:\n");
            for item in self.blocked_items.iter().take(10) {
                report.push_str(&format!("  🚫 {} ({})\n", item.url, item.reason));
            }
        }

        report
    }
}

impl Default for PrivacyMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics() {
        let mut metrics = PrivacyMetrics::new();
        metrics.new_page("https://example.com");

        metrics.add_blocked("https://tracker.com", BlockReason::Tracker);
        metrics.add_blocked("https://ads.com", BlockReason::Tracker);

        assert_eq!(metrics.total_trackers_blocked, 2);
        assert_eq!(metrics.current_page_blocks().len(), 2);
    }
}
