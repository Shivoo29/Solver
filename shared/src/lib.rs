use serde::{Deserialize, Serialize};

/// Messages sent from browser process to renderer process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserMessage {
    /// Request to render HTML content
    RenderHtml {
        url: String,
        html: String,
        width: u32,
        height: u32,
    },
    /// Shutdown the renderer
    Shutdown,
}

/// Messages sent from renderer process to browser process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RendererMessage {
    /// Rendered frame as RGBA bitmap data
    FrameReady {
        width: u32,
        height: u32,
        /// RGBA pixel data (width * height * 4 bytes)
        pixels: Vec<u8>,
    },
    /// Error occurred during rendering
    Error {
        message: String,
    },
    /// Renderer ready
    Ready,
}
