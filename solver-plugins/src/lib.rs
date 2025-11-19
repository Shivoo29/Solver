// Re-export core types for convenience
pub use solver_core::{BrowserPlugin, BrowserEvent, BrowserCore, PluginMetadata};

/// Helper macro to create plugin metadata
#[macro_export]
macro_rules! plugin_metadata {
    ($name:expr, $version:expr, $description:expr, $author:expr) => {
        $crate::PluginMetadata {
            name: $name.to_string(),
            version: $version.to_string(),
            description: $description.to_string(),
            author: $author.to_string(),
        }
    };
}

/// Helper trait for plugins that need initialization
pub trait InitializablePlugin {
    fn initialize(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Helper for creating simple event-handling plugins
pub struct SimplePlugin<F>
where
    F: Fn(&BrowserEvent, &mut BrowserCore) -> anyhow::Result<()> + Send + Sync,
{
    pub metadata: PluginMetadata,
    pub handler: F,
}

impl<F> SimplePlugin<F>
where
    F: Fn(&BrowserEvent, &mut BrowserCore) -> anyhow::Result<()> + Send + Sync,
{
    pub fn new(metadata: PluginMetadata, handler: F) -> Self {
        Self { metadata, handler }
    }
}

#[async_trait::async_trait]
impl<F> BrowserPlugin for SimplePlugin<F>
where
    F: Fn(&BrowserEvent, &mut BrowserCore) -> anyhow::Result<()> + Send + Sync,
{
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn init(&mut self, _core: &mut BrowserCore) -> anyhow::Result<()> {
        Ok(())
    }

    async fn handle_event(&mut self, event: BrowserEvent, core: &mut BrowserCore) -> anyhow::Result<()> {
        (self.handler)(&event, core)
    }
}
