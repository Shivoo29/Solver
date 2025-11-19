use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin Discovery System
pub struct PluginDiscovery {
    plugins: HashMap<String, PluginListing>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginListing {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub category: PluginCategory,
    pub downloads: u32,
    pub rating: f32,
    pub featured: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PluginCategory {
    Security,
    Privacy,
    Productivity,
    Entertainment,
    Developer,
    Social,
    Utility,
}

impl PluginDiscovery {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Add a plugin to the marketplace
    pub fn add_plugin(&mut self, plugin: PluginListing) {
        self.plugins.insert(plugin.id.clone(), plugin);
    }

    /// Search plugins by query
    pub fn search(&self, query: &str) -> Vec<PluginListing> {
        let query_lower = query.to_lowercase();

        self.plugins
            .values()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// Get trending plugins
    pub fn trending(&self) -> Vec<PluginListing> {
        let mut plugins: Vec<_> = self.plugins.values().cloned().collect();
        plugins.sort_by(|a, b| b.downloads.cmp(&a.downloads));
        plugins.into_iter().take(10).collect()
    }

    /// Get featured plugins
    pub fn featured(&self) -> Vec<PluginListing> {
        self.plugins
            .values()
            .filter(|p| p.featured)
            .cloned()
            .collect()
    }

    /// Get plugins by category
    pub fn by_category(&self, category: PluginCategory) -> Vec<PluginListing> {
        self.plugins
            .values()
            .filter(|p| p.category == category)
            .cloned()
            .collect()
    }

    /// Get plugin by ID
    pub fn get(&self, id: &str) -> Option<PluginListing> {
        self.plugins.get(id).cloned()
    }

    /// Get total plugin count
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Get all plugins
    pub fn all(&self) -> Vec<PluginListing> {
        self.plugins.values().cloned().collect()
    }
}

impl Default for PluginDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_discovery() {
        let mut discovery = PluginDiscovery::new();

        let plugin = PluginListing {
            id: "dark-mode".to_string(),
            name: "Dark Mode".to_string(),
            description: "Automatic dark mode for all websites".to_string(),
            author: "community".to_string(),
            version: "1.0.0".to_string(),
            category: PluginCategory::Utility,
            downloads: 1000,
            rating: 4.8,
            featured: true,
            tags: vec!["dark".to_string(), "theme".to_string()],
        };

        discovery.add_plugin(plugin);

        // Search
        let results = discovery.search("dark");
        assert_eq!(results.len(), 1);

        // Featured
        let featured = discovery.featured();
        assert_eq!(featured.len(), 1);

        // Count
        assert_eq!(discovery.count(), 1);
    }
}
