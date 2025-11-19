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
    /// Mouse click event
    MouseClick {
        x: f32,
        y: f32,
    },
    /// Key press event
    KeyPress {
        key: KeyEvent,
    },
    /// Shutdown the renderer
    Shutdown,
}

/// Keyboard event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyEvent {
    /// Regular character input
    Char(char),
    /// Backspace key
    Backspace,
    /// Delete key
    Delete,
    /// Enter/Return key
    Enter,
    /// Tab key
    Tab,
    /// Arrow keys
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
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
