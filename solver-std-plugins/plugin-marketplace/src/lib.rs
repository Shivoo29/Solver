use anyhow::Result;
use async_trait::async_trait;
use solver_core::{BrowserPlugin, BrowserEvent, BrowserCore, PluginMetadata};
use solver_plugins::plugin_metadata;
use std::sync::Arc;
use parking_lot::RwLock;

mod plugin_discovery;
mod plugin_installer;
mod plugin_sandbox;
mod community_plugins;

pub use plugin_discovery::{PluginDiscovery, PluginListing};
pub use plugin_installer::{PluginInstaller, InstallStatus};
pub use plugin_sandbox::{PluginSandbox, SecurityLevel};
pub use community_plugins::CommunityPlugins;

/// Plugin Marketplace Plugin
///
/// Provides complete plugin ecosystem:
/// - Plugin discovery (search, browse, trending)
/// - Plugin installation (one-click install)
/// - Security sandboxing (permissions, isolation)
/// - Community plugins (ratings, reviews)
pub struct PluginMarketplacePlugin {
    metadata: PluginMetadata,
    discovery: Arc<RwLock<PluginDiscovery>>,
    installer: Arc<RwLock<PluginInstaller>>,
    sandbox: Arc<RwLock<PluginSandbox>>,
    community: Arc<RwLock<CommunityPlugins>>,
    enabled: bool,
}

impl PluginMarketplacePlugin {
    pub fn new() -> Self {
        Self {
            metadata: plugin_metadata!(
                "Plugin Marketplace",
                "0.1.0",
                "Community plugin marketplace with discovery, installation, and security",
                "Solver Team"
            ),
            discovery: Arc::new(RwLock::new(PluginDiscovery::new())),
            installer: Arc::new(RwLock::new(PluginInstaller::new())),
            sandbox: Arc::new(RwLock::new(PluginSandbox::new())),
            community: Arc::new(RwLock::new(CommunityPlugins::new())),
            enabled: true,
        }
    }

    /// Load community plugins
    pub fn load_community_plugins(&mut self) -> Result<()> {
        let mut discovery = self.discovery.write();
        let mut community = self.community.write();

        // Load example community plugins
        let plugins = community.get_example_plugins();
        for plugin in plugins {
            discovery.add_plugin(plugin);
        }

        Ok(())
    }
}

impl Default for PluginMarketplacePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BrowserPlugin for PluginMarketplacePlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn init(&mut self, _core: &mut BrowserCore) -> Result<()> {
        eprintln!("[Plugin Marketplace] Initializing...");

        // Load community plugins
        self.load_community_plugins()?;

        let discovery = self.discovery.read();
        eprintln!("[Plugin Marketplace] ✓ Loaded {} plugins", discovery.count());
        eprintln!("[Plugin Marketplace] ✓ Plugin discovery enabled");
        eprintln!("[Plugin Marketplace] ✓ One-click installation enabled");
        eprintln!("[Plugin Marketplace] ✓ Security sandbox enabled");
        eprintln!("[Plugin Marketplace] ✓ Community features enabled");

        Ok(())
    }

    async fn handle_event(&mut self, event: BrowserEvent, core: &mut BrowserCore) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        match event {
            BrowserEvent::Custom { name, data } if name == "SearchPlugins" => {
                // Search for plugins
                let discovery = self.discovery.read();
                let results = discovery.search(&data);

                eprintln!("[Marketplace] 🔍 Search '{}': {} results", data, results.len());

                let results_json = serde_json::to_string(&results)?;
                core.emit_event(BrowserEvent::Custom {
                    name: "PluginSearchResults".to_string(),
                    data: results_json,
                });
            }

            BrowserEvent::Custom { name, data: _ } if name == "GetTrendingPlugins" => {
                // Get trending plugins
                let discovery = self.discovery.read();
                let trending = discovery.trending();

                eprintln!("[Marketplace] 📈 Trending: {} plugins", trending.len());

                let trending_json = serde_json::to_string(&trending)?;
                core.emit_event(BrowserEvent::Custom {
                    name: "TrendingPlugins".to_string(),
                    data: trending_json,
                });
            }

            BrowserEvent::Custom { name, data: _ } if name == "GetFeaturedPlugins" => {
                // Get featured plugins
                let discovery = self.discovery.read();
                let featured = discovery.featured();

                eprintln!("[Marketplace] ⭐ Featured: {} plugins", featured.len());

                let featured_json = serde_json::to_string(&featured)?;
                core.emit_event(BrowserEvent::Custom {
                    name: "FeaturedPlugins".to_string(),
                    data: featured_json,
                });
            }

            BrowserEvent::Custom { name, data } if name == "InstallPlugin" => {
                // Install a plugin
                eprintln!("[Marketplace] 📥 Installing plugin: {}", data);

                let mut installer = self.installer.write();
                let sandbox = self.sandbox.read();

                // Check security
                if !sandbox.is_safe(&data) {
                    eprintln!("[Marketplace] ⚠️ Security check failed for: {}", data);
                    core.emit_event(BrowserEvent::Custom {
                        name: "PluginInstallFailed".to_string(),
                        data: format!("Security check failed: {}", data),
                    });
                    return Ok(());
                }

                // Install
                match installer.install(&data) {
                    Ok(_) => {
                        eprintln!("[Marketplace] ✓ Installed: {}", data);
                        core.emit_event(BrowserEvent::Custom {
                            name: "PluginInstalled".to_string(),
                            data: data.clone(),
                        });
                    }
                    Err(e) => {
                        eprintln!("[Marketplace] ✗ Install failed: {}", e);
                        core.emit_event(BrowserEvent::Custom {
                            name: "PluginInstallFailed".to_string(),
                            data: format!("Install failed: {}", e),
                        });
                    }
                }
            }

            BrowserEvent::Custom { name, data } if name == "UninstallPlugin" => {
                // Uninstall a plugin
                eprintln!("[Marketplace] 🗑️ Uninstalling plugin: {}", data);

                let mut installer = self.installer.write();
                installer.uninstall(&data)?;

                eprintln!("[Marketplace] ✓ Uninstalled: {}", data);
                core.emit_event(BrowserEvent::Custom {
                    name: "PluginUninstalled".to_string(),
                    data,
                });
            }

            BrowserEvent::Custom { name, data } if name == "RatePlugin" => {
                // Rate a plugin (format: "plugin_id|rating")
                let parts: Vec<&str> = data.split('|').collect();
                if parts.len() == 2 {
                    let plugin_id = parts[0];
                    let rating: f32 = parts[1].parse().unwrap_or(0.0);

                    let mut community = self.community.write();
                    community.rate_plugin(plugin_id, rating);

                    eprintln!("[Marketplace] ⭐ Rated {} with {}/5", plugin_id, rating);

                    core.emit_event(BrowserEvent::Custom {
                        name: "PluginRated".to_string(),
                        data: plugin_id.to_string(),
                    });
                }
            }

            BrowserEvent::Custom { name, data: _ } if name == "GetMarketplaceStats" => {
                // Get marketplace statistics
                let discovery = self.discovery.read();
                let installer = self.installer.read();
                let community = self.community.read();

                let stats = serde_json::json!({
                    "total_plugins": discovery.count(),
                    "installed_plugins": installer.installed_count(),
                    "trending": discovery.trending().len(),
                    "featured": discovery.featured().len(),
                    "total_ratings": community.total_ratings(),
                });

                core.emit_event(BrowserEvent::Custom {
                    name: "MarketplaceStats".to_string(),
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
