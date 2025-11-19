use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Cloud AI using Google Gemini 2.0
///
/// Requires API key and internet connection.
/// More powerful than local models but sends data to Google.
pub struct CloudAI {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: String,
}

impl CloudAI {
    pub fn new() -> Result<Self> {
        // Get API key from environment
        let api_key = std::env::var("GEMINI_API_KEY")
            .or_else(|_| std::env::var("GOOGLE_API_KEY"))
            .map_err(|_| anyhow::anyhow!(
                "Gemini API key not found. Set GEMINI_API_KEY environment variable.\n  \
                 Get one at: https://ai.google.dev/"
            ))?;

        let client = reqwest::Client::builder()
            .user_agent("Solver-Browser/0.1.0")
            .build()?;

        Ok(Self {
            client,
            api_key,
            model: "gemini-2.0-flash-exp".to_string(), // Latest Gemini model
        })
    }

    pub async fn summarize(&mut self, html: &str) -> Result<String> {
        let text = crate::extract_text_from_html(html);

        // Limit context size
        let text = text.chars().take(8000).collect::<String>();

        let prompt = format!(
            "Summarize this webpage in 2-3 concise sentences. Focus on the main points:\n\n{}",
            text
        );

        self.generate(&prompt).await
    }

    pub async fn answer_question(&mut self, question: &str, context: &str) -> Result<String> {
        let prompt = format!(
            "Context: {}\n\nQuestion: {}\n\nProvide a clear, concise answer:",
            context, question
        );

        self.generate(&prompt).await
    }

    async fn generate(&self, prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Gemini API error {}: {}", status, error_text));
        }

        let gemini_response: GeminiResponse = response.json().await?;

        let text = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| anyhow::anyhow!("Empty response from Gemini"))?;

        Ok(text)
    }
}
