use anyhow::Result;
use async_trait::async_trait;
use solver_core::{BrowserPlugin, BrowserEvent, BrowserCore, PluginMetadata};
use solver_plugins::plugin_metadata;

/// Networking plugin for HTTP/HTTPS requests
pub struct NetworkingPlugin {
    metadata: PluginMetadata,
    client: reqwest::Client,
}

impl NetworkingPlugin {
    pub fn new() -> Self {
        Self {
            metadata: plugin_metadata!(
                "Network Stack",
                "0.1.0",
                "HTTP/HTTPS networking with TLS support",
                "Solver Team"
            ),
            client: reqwest::Client::builder()
                .user_agent("Solver/0.1.0")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    async fn fetch_url(&self, url: &str) -> Result<String> {
        eprintln!("[Networking Plugin] Fetching: {}", url);

        // Handle file:// URLs
        if url.starts_with("file://") {
            let path = url.strip_prefix("file://").unwrap();
            return std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e));
        }

        // Fetch from network
        let response = self.client
            .get(url)
            .send()
            .await?
            .text()
            .await?;

        Ok(response)
    }
}

impl Default for NetworkingPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BrowserPlugin for NetworkingPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn init(&mut self, _core: &mut BrowserCore) -> Result<()> {
        eprintln!("[Networking Plugin] Initialized with HTTPS/TLS support");
        Ok(())
    }

    async fn handle_event(&mut self, event: BrowserEvent, core: &mut BrowserCore) -> Result<()> {
        match event {
            BrowserEvent::PageLoadStart { url } => {
                eprintln!("[Networking Plugin] Loading: {}", url);

                // Fetch the HTML
                match self.fetch_url(&url).await {
                    Ok(html) => {
                        core.emit_event(BrowserEvent::HtmlFetched {
                            url: url.clone(),
                            html,
                        });
                    }
                    Err(e) => {
                        eprintln!("[Networking Plugin] Error fetching {}: {}", url, e);
                    }
                }
            }
            BrowserEvent::NetworkRequest { url, method, .. } => {
                eprintln!("[Networking Plugin] {} request to: {}", method, url);
                // Handle network requests from JavaScript
            }
            _ => {}
        }

        Ok(())
    }
}
