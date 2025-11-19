use anyhow::Result;
use async_trait::async_trait;
use crate::event::BrowserEvent;
use crate::core::BrowserCore;

/// Metadata about a plugin
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

/// Core trait that all browser plugins must implement
///
/// This is the foundation of Solver's modular architecture.
/// Every component (rendering, JS, networking, privacy, etc.) is a plugin.
#[async_trait]
pub trait BrowserPlugin: Send + Sync {
    /// Plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Initialize the plugin
    /// Called once when the plugin is registered
    fn init(&mut self, core: &mut BrowserCore) -> Result<()>;

    /// Handle a browser event
    /// Plugins can react to events and emit new events
    async fn handle_event(&mut self, event: BrowserEvent, core: &mut BrowserCore) -> Result<()>;

    /// Shutdown the plugin
    /// Called when the browser is closing
    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    /// Whether this plugin is enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

/// Plugin priority for event handling order
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PluginPriority {
    Critical = 0,  // Security, privacy plugins
    High = 1,      // Networking, rendering plugins
    Normal = 2,    // Standard plugins
    Low = 3,       // Non-essential plugins
}

/// Wrapper for registered plugins
pub struct RegisteredPlugin {
    pub plugin: Box<dyn BrowserPlugin>,
    pub priority: PluginPriority,
    pub enabled: bool,
}

impl RegisteredPlugin {
    pub fn new(plugin: Box<dyn BrowserPlugin>, priority: PluginPriority) -> Self {
        Self {
            plugin,
            priority,
            enabled: true,
        }
    }
}
