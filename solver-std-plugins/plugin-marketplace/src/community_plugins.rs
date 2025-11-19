use crate::plugin_discovery::{PluginCategory, PluginListing};
use std::collections::HashMap;

/// Community Plugins Repository
pub struct CommunityPlugins {
    ratings: HashMap<String, Vec<f32>>,
}

impl CommunityPlugins {
    pub fn new() -> Self {
        Self {
            ratings: HashMap::new(),
        }
    }

    /// Get example community plugins
    pub fn get_example_plugins(&self) -> Vec<PluginListing> {
        vec![
            // Security plugins
            PluginListing {
                id: "solver-team/password-manager".to_string(),
                name: "Password Manager".to_string(),
                description: "Secure password manager with auto-fill".to_string(),
                author: "solver-team".to_string(),
                version: "1.0.0".to_string(),
                category: PluginCategory::Security,
                downloads: 5000,
                rating: 4.9,
                featured: true,
                tags: vec!["security".to_string(), "password".to_string(), "autofill".to_string()],
            },
            // Privacy plugins
            PluginListing {
                id: "community/cookie-crusher".to_string(),
                name: "Cookie Crusher".to_string(),
                description: "Advanced cookie management and deletion".to_string(),
                author: "community".to_string(),
                version: "2.1.0".to_string(),
                category: PluginCategory::Privacy,
                downloads: 3500,
                rating: 4.7,
                featured: true,
                tags: vec!["privacy".to_string(), "cookies".to_string()],
            },
            // Productivity plugins
            PluginListing {
                id: "verified/tab-organizer".to_string(),
                name: "Tab Organizer".to_string(),
                description: "Organize tabs into workspaces and collections".to_string(),
                author: "verified".to_string(),
                version: "1.5.0".to_string(),
                category: PluginCategory::Productivity,
                downloads: 8000,
                rating: 4.8,
                featured: true,
                tags: vec!["productivity".to_string(), "tabs".to_string(), "workspace".to_string()],
            },
            PluginListing {
                id: "community/dark-mode-pro".to_string(),
                name: "Dark Mode Pro".to_string(),
                description: "Automatic dark mode for all websites with custom themes".to_string(),
                author: "community".to_string(),
                version: "3.0.0".to_string(),
                category: PluginCategory::Utility,
                downloads: 12000,
                rating: 4.9,
                featured: true,
                tags: vec!["dark".to_string(), "theme".to_string(), "utility".to_string()],
            },
            // Developer plugins
            PluginListing {
                id: "verified/dev-tools-enhanced".to_string(),
                name: "Dev Tools Enhanced".to_string(),
                description: "Enhanced developer tools with AI assistance".to_string(),
                author: "verified".to_string(),
                version: "1.2.0".to_string(),
                category: PluginCategory::Developer,
                downloads: 4500,
                rating: 4.6,
                featured: false,
                tags: vec!["developer".to_string(), "devtools".to_string(), "ai".to_string()],
            },
            // Social plugins
            PluginListing {
                id: "community/social-hub".to_string(),
                name: "Social Hub".to_string(),
                description: "Unified social media notifications and posting".to_string(),
                author: "community".to_string(),
                version: "2.0.0".to_string(),
                category: PluginCategory::Social,
                downloads: 6500,
                rating: 4.5,
                featured: false,
                tags: vec!["social".to_string(), "notifications".to_string()],
            },
            // Entertainment plugins
            PluginListing {
                id: "verified/video-enhancer".to_string(),
                name: "Video Enhancer".to_string(),
                description: "Enhance video quality and add cinema mode".to_string(),
                author: "verified".to_string(),
                version: "1.8.0".to_string(),
                category: PluginCategory::Entertainment,
                downloads: 7000,
                rating: 4.7,
                featured: false,
                tags: vec!["video".to_string(), "entertainment".to_string(), "quality".to_string()],
            },
            // More utility plugins
            PluginListing {
                id: "solver-team/screenshot-pro".to_string(),
                name: "Screenshot Pro".to_string(),
                description: "Advanced screenshot tools with annotations".to_string(),
                author: "solver-team".to_string(),
                version: "1.0.0".to_string(),
                category: PluginCategory::Utility,
                downloads: 5500,
                rating: 4.8,
                featured: true,
                tags: vec!["screenshot".to_string(), "utility".to_string()],
            },
        ]
    }

    /// Rate a plugin
    pub fn rate_plugin(&mut self, plugin_id: &str, rating: f32) {
        let ratings = self.ratings.entry(plugin_id.to_string()).or_insert_with(Vec::new);
        ratings.push(rating.clamp(0.0, 5.0));
    }

    /// Get average rating
    pub fn get_rating(&self, plugin_id: &str) -> Option<f32> {
        let ratings = self.ratings.get(plugin_id)?;
        if ratings.is_empty() {
            return None;
        }

        let sum: f32 = ratings.iter().sum();
        Some(sum / ratings.len() as f32)
    }

    /// Get total ratings count
    pub fn total_ratings(&self) -> usize {
        self.ratings.values().map(|v| v.len()).sum()
    }

    /// Get rating count for plugin
    pub fn rating_count(&self, plugin_id: &str) -> usize {
        self.ratings.get(plugin_id).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for CommunityPlugins {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_community_plugins() {
        let mut community = CommunityPlugins::new();

        // Rate plugin
        community.rate_plugin("dark-mode", 5.0);
        community.rate_plugin("dark-mode", 4.0);
        community.rate_plugin("dark-mode", 5.0);

        // Get rating
        let rating = community.get_rating("dark-mode");
        assert!(rating.is_some());
        assert!((rating.unwrap() - 4.67).abs() < 0.01);

        // Get example plugins
        let plugins = community.get_example_plugins();
        assert!(plugins.len() > 0);
    }
}
