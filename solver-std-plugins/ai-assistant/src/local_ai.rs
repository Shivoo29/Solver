use anyhow::Result;

/// Local AI using Llama 3.2
///
/// This runs entirely on-device for maximum privacy.
/// Requires Llama 3.2 model file (~1.5GB for 1B model)
pub struct LocalAI {
    _model_path: String,
}

impl LocalAI {
    pub fn new() -> Result<Self> {
        // Check if model exists
        let model_path = "models/llama-3.2-1b-instruct.gguf";

        if !std::path::Path::new(model_path).exists() {
            return Err(anyhow::anyhow!(
                "Llama model not found at {}. Download it with:\n  \
                 wget https://huggingface.co/TheBloke/Llama-3.2-1B-Instruct-GGUF/resolve/main/llama-3.2-1b-instruct.gguf -P models/",
                model_path
            ));
        }

        // TODO: Initialize llama-cpp-rs when ready
        // For now, this is a placeholder

        Ok(Self {
            _model_path: model_path.to_string(),
        })
    }

    pub async fn summarize(&mut self, html: &str) -> Result<String> {
        let text = crate::extract_text_from_html(html);
        let text = text.chars().take(2000).collect::<String>();

        // TODO: Use actual Llama model
        // For now, return a placeholder
        Ok(format!("[Local AI] Summary of {} chars: {}",
            text.len(),
            &text.chars().take(100).collect::<String>()
        ))
    }

    pub async fn answer_question(&mut self, question: &str, context: &str) -> Result<String> {
        // TODO: Use actual Llama model
        Ok(format!("[Local AI] Q: {} | Context: {}", question, context))
    }
}
