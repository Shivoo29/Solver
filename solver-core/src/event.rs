use serde::{Deserialize, Serialize};

/// Events that flow through the browser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserEvent {
    /// Browser initialized
    BrowserInit,

    /// Navigation requested to a URL
    NavigationRequested { url: String },

    /// Page load started
    PageLoadStart { url: String },

    /// HTML fetched from network
    HtmlFetched { url: String, html: String },

    /// DOM tree ready
    DomReady { url: String },

    /// CSS parsed
    CssParsed { url: String },

    /// JavaScript needs execution
    ExecuteScript { url: String, script: String },

    /// Layout computed
    LayoutReady { url: String, width: u32, height: u32 },

    /// Frame ready to render
    RenderFrame { url: String, width: u32, height: u32, pixels: Vec<u8> },

    /// User input event
    UserInput {
        event_type: UserInputType,
        x: Option<f32>,
        y: Option<f32>,
        key: Option<String>,
    },

    /// Network request intercepted
    NetworkRequest {
        url: String,
        method: String,
        headers: Vec<(String, String)>,
    },

    /// Privacy event (tracker detected, permission requested, etc.)
    PrivacyEvent {
        event_type: PrivacyEventType,
        origin: String,
        details: String,
    },

    /// Plugin registered
    PluginRegistered { name: String },

    /// Custom event from plugins
    Custom { name: String, data: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserInputType {
    MouseClick,
    MouseMove,
    KeyPress,
    Scroll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyEventType {
    TrackerDetected,
    PermissionRequested,
    FingerprintAttempt,
    CookieAccess,
}

/// Event bus for plugin communication
pub struct EventBus {
    subscribers: Vec<String>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, plugin_name: String) {
        self.subscribers.push(plugin_name);
    }

    pub fn unsubscribe(&mut self, plugin_name: &str) {
        self.subscribers.retain(|name| name != plugin_name);
    }

    pub fn subscribers(&self) -> &[String] {
        &self.subscribers
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
