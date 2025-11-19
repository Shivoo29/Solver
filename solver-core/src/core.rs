use anyhow::Result;
use std::sync::{Arc, RwLock};
use crate::event::{BrowserEvent, EventBus};
use crate::plugin::{RegisteredPlugin, PluginPriority};

/// Global browser state shared across plugins
#[derive(Debug, Clone)]
pub struct BrowserState {
    pub current_url: Option<String>,
    pub window_width: u32,
    pub window_height: u32,
    pub user_agent: String,
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            current_url: None,
            window_width: 1024,
            window_height: 768,
            user_agent: "Solver/0.1.0 (Rust; AI-native)".to_string(),
        }
    }
}

/// Core browser engine
///
/// This is the heart of Solver. It manages plugins and routes events.
pub struct BrowserCore {
    plugins: Vec<RegisteredPlugin>,
    event_bus: EventBus,
    state: Arc<RwLock<BrowserState>>,
    event_queue: Vec<BrowserEvent>,
}

impl BrowserCore {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            event_bus: EventBus::new(),
            state: Arc::new(RwLock::new(BrowserState::default())),
            event_queue: Vec::new(),
        }
    }

    /// Register a new plugin
    pub fn register_plugin(&mut self, mut plugin: Box<dyn crate::plugin::BrowserPlugin>, priority: PluginPriority) -> Result<()> {
        let name = plugin.metadata().name.clone();

        eprintln!("[Core] Registering plugin: {} (priority: {:?})", name, priority);

        // Initialize the plugin
        plugin.init(self)?;

        // Add to event bus
        self.event_bus.subscribe(name.clone());

        // Store plugin
        self.plugins.push(RegisteredPlugin::new(plugin, priority));

        // Sort plugins by priority
        self.plugins.sort_by_key(|p| p.priority);

        // Emit event
        self.emit_event(BrowserEvent::PluginRegistered { name });

        Ok(())
    }

    /// Emit an event to all plugins
    pub fn emit_event(&mut self, event: BrowserEvent) {
        eprintln!("[Core] Event: {:?}", event);
        self.event_queue.push(event);
    }

    /// Process all queued events
    pub async fn process_events(&mut self) -> Result<()> {
        let events: Vec<_> = self.event_queue.drain(..).collect();

        for event in events {
            // Process plugins one at a time to avoid borrow checker issues
            let mut new_events = Vec::new();

            for i in 0..self.plugins.len() {
                let plugin = &mut self.plugins[i];

                if plugin.enabled && plugin.plugin.is_enabled() {
                    // Create a temporary core for the plugin to use
                    // The plugin can't directly mutate the main core during event handling
                    let mut temp_core = BrowserCore {
                        plugins: Vec::new(),
                        event_bus: EventBus::new(),
                        state: Arc::clone(&self.state),
                        event_queue: Vec::new(),
                    };

                    if let Err(e) = plugin.plugin.handle_event(event.clone(), &mut temp_core).await {
                        eprintln!("[Core] Plugin {} error: {}",
                            plugin.plugin.metadata().name, e);
                    }

                    // Collect any events emitted by the plugin
                    new_events.extend(temp_core.event_queue);
                }
            }

            // Add new events to the queue for next iteration
            self.event_queue.extend(new_events);
        }

        Ok(())
    }

    /// Get browser state (read-only)
    pub fn state(&self) -> Arc<RwLock<BrowserState>> {
        Arc::clone(&self.state)
    }

    /// Get plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&RegisteredPlugin> {
        self.plugins.iter().find(|p| p.plugin.metadata().name == name)
    }

    /// Enable/disable a plugin
    pub fn set_plugin_enabled(&mut self, name: &str, enabled: bool) {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.plugin.metadata().name == name) {
            plugin.enabled = enabled;
            eprintln!("[Core] Plugin {} {}", name, if enabled { "enabled" } else { "disabled" });
        }
    }

    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.iter()
            .map(|p| format!("{} v{} ({})",
                p.plugin.metadata().name,
                p.plugin.metadata().version,
                if p.enabled { "enabled" } else { "disabled" }
            ))
            .collect()
    }
}

impl Default for BrowserCore {
    fn default() -> Self {
        Self::new()
    }
}
