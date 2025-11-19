use std::collections::HashMap;

/// Intelligent resource preloader
/// Learns patterns and predicts what resources to load
pub struct IntelligentPreloader {
    // URL -> Resources typically loaded from this page
    patterns: HashMap<String, Vec<String>>,

    // URL -> How many times it's been visited
    visit_counts: HashMap<String, u32>,
}

impl IntelligentPreloader {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            visit_counts: HashMap::new(),
        }
    }

    /// Record a page load and its resources
    pub fn record_page_load(&mut self, url: &str, _html: &str) {
        // Increment visit count
        *self.visit_counts.entry(url.to_string()).or_insert(0) += 1;

        // In real implementation:
        // - Parse HTML for resources (scripts, styles, images)
        // - Record patterns
        // - Build prediction model

        // For now, just track that we visited
    }

    /// Record a resource load
    pub fn record_resource_load(&mut self, page_url: &str, resource_url: &str) {
        self.patterns
            .entry(page_url.to_string())
            .or_insert_with(Vec::new)
            .push(resource_url.to_string());
    }

    /// Get predicted resources for a URL
    /// Returns resources that should be preloaded
    pub fn get_predicted_resources(&self, url: &str) -> Option<Vec<String>> {
        // Check if we have patterns for this URL
        if let Some(resources) = self.patterns.get(url) {
            // Only preload if we've seen this page multiple times
            if self.visit_counts.get(url).copied().unwrap_or(0) >= 2 {
                return Some(resources.clone());
            }
        }

        // Try to predict based on similar URLs (domain matching)
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(domain) = parsed.domain() {
                // Find similar URLs from same domain
                for (pattern_url, resources) in &self.patterns {
                    if let Ok(pattern_parsed) = url::Url::parse(pattern_url) {
                        if pattern_parsed.domain() == Some(domain) {
                            // Similar domain, might have similar resources
                            return Some(resources.clone());
                        }
                    }
                }
            }
        }

        None
    }

    /// Get statistics
    pub fn stats(&self) -> PreloadStats {
        PreloadStats {
            patterns_learned: self.patterns.len(),
            total_visits: self.visit_counts.values().sum(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreloadStats {
    pub patterns_learned: usize,
    pub total_visits: u32,
}

impl Default for IntelligentPreloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preloading() {
        let mut preloader = IntelligentPreloader::new();

        // First visit - no prediction
        assert!(preloader.get_predicted_resources("https://example.com").is_none());

        // Record resources
        preloader.record_page_load("https://example.com", "<html></html>");
        preloader.record_resource_load("https://example.com", "https://example.com/style.css");
        preloader.record_resource_load("https://example.com", "https://example.com/script.js");

        // Second visit - no prediction yet (need 2+ visits)
        preloader.record_page_load("https://example.com", "<html></html>");

        // Third visit - should predict
        let resources = preloader.get_predicted_resources("https://example.com");
        assert!(resources.is_some());
        assert_eq!(resources.unwrap().len(), 2);
    }
}
