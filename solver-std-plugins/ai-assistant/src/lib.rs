use anyhow::Result;
use async_trait::async_trait;
use solver_core::{BrowserPlugin, BrowserEvent, BrowserCore, PluginMetadata};
use solver_plugins::plugin_metadata;
use serde::{Deserialize, Serialize};

mod local_ai;
mod cloud_ai;

pub use local_ai::LocalAI;
pub use cloud_ai::CloudAI;

/// AI mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIMode {
    Local,   // Privacy-first, offline, Llama 3.2
    Cloud,   // More powerful, requires internet, Gemini
    Auto,    // Try local first, fallback to cloud if needed
}

/// AI Assistant Plugin
///
/// Provides intelligent browsing assistance:
/// - Page summarization
/// - Q&A about current page
/// - Smart form filling
/// - Content extraction
///
/// Supports both local (Llama) and cloud (Gemini) modes
pub struct AIAssistantPlugin {
    metadata: PluginMetadata,
    mode: AIMode,
    local_ai: Option<LocalAI>,
    cloud_ai: Option<CloudAI>,
    enabled: bool,
}

impl AIAssistantPlugin {
    pub fn new() -> Self {
        Self {
            metadata: plugin_metadata!(
                "AI Assistant",
                "0.1.0",
                "Intelligent browsing assistant with local and cloud AI",
                "Solver Team"
            ),
            mode: AIMode::Auto,
            local_ai: None,
            cloud_ai: None,
            enabled: true,
        }
    }

    pub fn with_mode(mut self, mode: AIMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn set_mode(&mut self, mode: AIMode) {
        self.mode = mode;
        eprintln!("[AI Assistant] Switched to {:?} mode", mode);
    }

    async fn summarize_page(&mut self, html: &str) -> Result<String> {
        match self.mode {
            AIMode::Local => {
                if let Some(ref mut local) = self.local_ai {
                    local.summarize(html).await
                } else {
                    Err(anyhow::anyhow!("Local AI not initialized"))
                }
            }
            AIMode::Cloud => {
                if let Some(ref mut cloud) = self.cloud_ai {
                    cloud.summarize(html).await
                } else {
                    Err(anyhow::anyhow!("Cloud AI not initialized"))
                }
            }
            AIMode::Auto => {
                // Try local first, fallback to cloud
                if let Some(ref mut local) = self.local_ai {
                    if let Ok(result) = local.summarize(html).await {
                        return Ok(result);
                    }
                }

                if let Some(ref mut cloud) = self.cloud_ai {
                    cloud.summarize(html).await
                } else {
                    Err(anyhow::anyhow!("No AI backend available"))
                }
            }
        }
    }

    async fn answer_question(&mut self, question: &str, context: &str) -> Result<String> {
        match self.mode {
            AIMode::Local => {
                if let Some(ref mut local) = self.local_ai {
                    local.answer_question(question, context).await
                } else {
                    Err(anyhow::anyhow!("Local AI not initialized"))
                }
            }
            AIMode::Cloud => {
                if let Some(ref mut cloud) = self.cloud_ai {
                    cloud.answer_question(question, context).await
                } else {
                    Err(anyhow::anyhow!("Cloud AI not initialized"))
                }
            }
            AIMode::Auto => {
                // Try local first
                if let Some(ref mut local) = self.local_ai {
                    if let Ok(result) = local.answer_question(question, context).await {
                        return Ok(result);
                    }
                }

                if let Some(ref mut cloud) = self.cloud_ai {
                    cloud.answer_question(question, context).await
                } else {
                    Err(anyhow::anyhow!("No AI backend available"))
                }
            }
        }
    }
}

impl Default for AIAssistantPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BrowserPlugin for AIAssistantPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn init(&mut self, _core: &mut BrowserCore) -> Result<()> {
        eprintln!("[AI Assistant] Initializing in {:?} mode...", self.mode);

        // Initialize local AI if available
        #[cfg(feature = "local-ai")]
        {
            match LocalAI::new() {
                Ok(local) => {
                    eprintln!("[AI Assistant] ✓ Local AI (Llama 3.2) initialized");
                    self.local_ai = Some(local);
                }
                Err(e) => {
                    eprintln!("[AI Assistant] ✗ Local AI initialization failed: {}", e);
                }
            }
        }

        // Initialize cloud AI if available
        #[cfg(feature = "cloud-ai")]
        {
            match CloudAI::new() {
                Ok(cloud) => {
                    eprintln!("[AI Assistant] ✓ Cloud AI (Gemini) initialized");
                    self.cloud_ai = Some(cloud);
                }
                Err(e) => {
                    eprintln!("[AI Assistant] ✗ Cloud AI initialization failed: {}", e);
                }
            }
        }

        if self.local_ai.is_none() && self.cloud_ai.is_none() {
            eprintln!("[AI Assistant] ⚠️  No AI backend available");
            self.enabled = false;
        }

        Ok(())
    }

    async fn handle_event(&mut self, event: BrowserEvent, core: &mut BrowserCore) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        match event {
            BrowserEvent::HtmlFetched { url, html } => {
                eprintln!("[AI Assistant] Page loaded: {}", url);

                // Summarize in background
                match self.summarize_page(&html).await {
                    Ok(summary) => {
                        eprintln!("[AI Assistant] Summary: {}", summary);

                        // Emit custom event with summary
                        core.emit_event(BrowserEvent::Custom {
                            name: "AISummary".to_string(),
                            data: summary,
                        });
                    }
                    Err(e) => {
                        eprintln!("[AI Assistant] Summarization failed: {}", e);
                    }
                }
            }

            BrowserEvent::Custom { name, data } if name == "AIQuery" => {
                eprintln!("[AI Assistant] Answering question: {}", data);

                // Get current page context from browser state
                let context = {
                    let state = core.state();
                    let state_lock = state.read().unwrap();
                    state_lock.current_url.clone().unwrap_or_default()
                };

                match self.answer_question(&data, &context).await {
                    Ok(answer) => {
                        eprintln!("[AI Assistant] Answer: {}", answer);

                        core.emit_event(BrowserEvent::Custom {
                            name: "AIAnswer".to_string(),
                            data: answer,
                        });
                    }
                    Err(e) => {
                        eprintln!("[AI Assistant] Q&A failed: {}", e);
                    }
                }
            }

            _ => {}
        }

        Ok(())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Extract text content from HTML (simplified)
pub fn extract_text_from_html(html: &str) -> String {
    // Remove script and style tags
    let mut text = html.to_string();

    // Simple regex-free approach
    while let Some(start) = text.find("<script") {
        if let Some(end) = text[start..].find("</script>") {
            text.replace_range(start..start + end + 9, "");
        } else {
            break;
        }
    }

    while let Some(start) = text.find("<style") {
        if let Some(end) = text[start..].find("</style>") {
            text.replace_range(start..start + end + 8, "");
        } else {
            break;
        }
    }

    // Remove all HTML tags
    let mut result = String::new();
    let mut in_tag = false;

    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    // Clean up whitespace
    result.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
