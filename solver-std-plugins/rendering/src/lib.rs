use anyhow::Result;
use async_trait::async_trait;
use solver_core::{BrowserPlugin, BrowserEvent, BrowserCore, PluginMetadata};
use solver_plugins::plugin_metadata;

mod rendering_engine;

/// Rendering plugin that handles HTML/CSS rendering
pub struct RenderingPlugin {
    metadata: PluginMetadata,
    width: u32,
    height: u32,
}

impl RenderingPlugin {
    pub fn new() -> Self {
        Self {
            metadata: plugin_metadata!(
                "Standard Renderer",
                "0.1.0",
                "HTML/CSS rendering engine",
                "Solver Team"
            ),
            width: 1024,
            height: 768,
        }
    }

    fn render_page(&self, url: &str, html: &str) -> Result<Vec<u8>> {
        eprintln!("[Rendering Plugin] Rendering: {}", url);

        // TODO: Port the actual rendering code here
        // For now, we'll integrate with the existing renderer module

        // This will be refactored to use the full rendering pipeline
        Ok(vec![0; (self.width * self.height * 4) as usize])
    }
}

impl Default for RenderingPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BrowserPlugin for RenderingPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn init(&mut self, core: &mut BrowserCore) -> Result<()> {
        eprintln!("[Rendering Plugin] Initialized");

        // Get window dimensions from core state
        let state = core.state();
        let state_lock = state.read().unwrap();
        self.width = state_lock.window_width;
        self.height = state_lock.window_height;

        Ok(())
    }

    async fn handle_event(&mut self, event: BrowserEvent, core: &mut BrowserCore) -> Result<()> {
        match event {
            BrowserEvent::HtmlFetched { url, html } => {
                eprintln!("[Rendering Plugin] Got HTML for: {}", url);

                // Emit DOM ready event
                core.emit_event(BrowserEvent::DomReady {
                    url: url.clone()
                });

                // Render the page
                let pixels = self.render_page(&url, &html)?;

                // Emit render frame event
                core.emit_event(BrowserEvent::RenderFrame {
                    url,
                    width: self.width,
                    height: self.height,
                    pixels,
                });
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = RenderingPlugin::new();
        assert_eq!(plugin.metadata().name, "Standard Renderer");
    }
}
