use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;

/// Tracker blocking using filter lists (EasyList/EasyPrivacy format)
pub struct TrackerBlocker {
    // Domain-based blocks (e.g., "doubleclick.net")
    blocked_domains: HashSet<String>,

    // URL pattern blocks (simple substring matching for now)
    blocked_patterns: Vec<String>,

    // Regex patterns for complex rules
    blocked_regex: Vec<Regex>,
}

impl TrackerBlocker {
    pub fn new() -> Self {
        Self {
            blocked_domains: HashSet::new(),
            blocked_patterns: Vec::new(),
            blocked_regex: Vec::new(),
        }
    }

    /// Load default tracker lists
    pub fn load_default_lists(&mut self) -> Result<()> {
        // Top tracking domains (subset of EasyPrivacy)
        let common_trackers = vec![
            // Ad networks
            "doubleclick.net",
            "googleadservices.com",
            "googlesyndication.com",
            "google-analytics.com",
            "googletagmanager.com",
            "facebook.com/tr",
            "facebook.net",
            "connect.facebook.net",

            // Analytics
            "mixpanel.com",
            "segment.com",
            "amplitude.com",
            "hotjar.com",
            "fullstory.com",

            // Ad tech
            "adnxs.com",
            "adsrvr.org",
            "advertising.com",
            "criteo.com",
            "taboola.com",
            "outbrain.com",

            // Social widgets
            "twitter.com/widgets",
            "platform.twitter.com",
            "linkedin.com/analytics",

            // More trackers
            "scorecardresearch.com",
            "quantserve.com",
            "moatads.com",
            "2mdn.net",
        ];

        for domain in common_trackers {
            self.blocked_domains.insert(domain.to_string());
        }

        // Common tracking patterns
        let patterns = vec![
            "/analytics.js",
            "/tracking.js",
            "/ga.js",
            "/gtag",
            "/pixel",
            "/beacon",
            "/track",
        ];

        for pattern in patterns {
            self.blocked_patterns.push(pattern.to_string());
        }

        eprintln!("[Tracker Blocker] Loaded {} domains, {} patterns",
                  self.blocked_domains.len(),
                  self.blocked_patterns.len());

        Ok(())
    }

    /// Check if a URL is a known tracker
    pub fn is_tracker(&self, url: &str) -> bool {
        // Parse URL
        if let Ok(parsed) = url::Url::parse(url) {
            // Check domain
            if let Some(domain) = parsed.host_str() {
                // Exact domain match
                if self.blocked_domains.contains(domain) {
                    return true;
                }

                // Subdomain match (e.g., "www.doubleclick.net" matches "doubleclick.net")
                for blocked in &self.blocked_domains {
                    if domain.ends_with(blocked) {
                        return true;
                    }
                }
            }

            // Check path patterns
            let path = parsed.path();
            for pattern in &self.blocked_patterns {
                if path.contains(pattern) {
                    return true;
                }
            }

            // Check regex patterns
            for regex in &self.blocked_regex {
                if regex.is_match(url) {
                    return true;
                }
            }
        }

        false
    }

    /// Add custom block rule
    pub fn add_rule(&mut self, rule: &str) {
        if rule.contains('*') || rule.contains('?') {
            // Convert to regex
            let regex_pattern = rule
                .replace(".", "\\.")
                .replace("*", ".*")
                .replace("?", ".");

            if let Ok(regex) = Regex::new(&regex_pattern) {
                self.blocked_regex.push(regex);
            }
        } else if rule.contains('/') {
            // URL pattern
            self.blocked_patterns.push(rule.to_string());
        } else {
            // Domain
            self.blocked_domains.insert(rule.to_string());
        }
    }

    /// Get statistics
    pub fn stats(&self) -> TrackerStats {
        TrackerStats {
            domains: self.blocked_domains.len(),
            patterns: self.blocked_patterns.len(),
            regex: self.blocked_regex.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrackerStats {
    pub domains: usize,
    pub patterns: usize,
    pub regex: usize,
}

impl Default for TrackerBlocker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_detection() {
        let mut blocker = TrackerBlocker::new();
        blocker.load_default_lists().unwrap();

        // Should block
        assert!(blocker.is_tracker("https://www.googleadservices.com/pagead/conversion.js"));
        assert!(blocker.is_tracker("https://www.google-analytics.com/analytics.js"));
        assert!(blocker.is_tracker("https://connect.facebook.net/en_US/fbevents.js"));

        // Should not block
        assert!(!blocker.is_tracker("https://www.google.com/search"));
        assert!(!blocker.is_tracker("https://github.com/user/repo"));
    }
}
