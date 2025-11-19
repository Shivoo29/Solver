use serde::{Deserialize, Serialize};

/// App Manifest
///
/// Represents a PWA manifest for installing as native app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    pub name: String,
    pub short_name: Option<String>,
    pub start_url: String,
    pub display: ManifestDisplay,
    pub background_color: Option<String>,
    pub theme_color: Option<String>,
    pub description: Option<String>,
    pub icons: Vec<ManifestIcon>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ManifestDisplay {
    Fullscreen,
    Standalone,
    MinimalUI,
    Browser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestIcon {
    pub src: String,
    pub sizes: String,
    #[serde(rename = "type")]
    pub icon_type: String,
}

impl AppManifest {
    /// Create new manifest
    pub fn new(name: &str, start_url: &str) -> Self {
        Self {
            name: name.to_string(),
            short_name: None,
            start_url: start_url.to_string(),
            display: ManifestDisplay::Standalone,
            background_color: None,
            theme_color: None,
            description: None,
            icons: Vec::new(),
            scope: None,
        }
    }

    /// Parse from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Create stub manifest for testing
    pub fn stub(url: &str) -> Self {
        Self {
            name: "Test PWA".to_string(),
            short_name: Some("PWA".to_string()),
            start_url: url.to_string(),
            display: ManifestDisplay::Standalone,
            background_color: Some("#ffffff".to_string()),
            theme_color: Some("#2196f3".to_string()),
            description: Some("A test Progressive Web App".to_string()),
            icons: vec![ManifestIcon {
                src: "/icon-192.png".to_string(),
                sizes: "192x192".to_string(),
                icon_type: "image/png".to_string(),
            }],
            scope: Some("/".to_string()),
        }
    }

    /// Get app name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get start URL
    pub fn start_url(&self) -> &str {
        &self.start_url
    }

    /// Check if installable
    pub fn is_installable(&self) -> bool {
        // Must have name, start_url, and at least one icon
        !self.name.is_empty() && !self.start_url.is_empty() && !self.icons.is_empty()
    }

    /// Get display mode
    pub fn display(&self) -> &ManifestDisplay {
        &self.display
    }
}

impl Default for AppManifest {
    fn default() -> Self {
        Self::new("Untitled App", "/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest() {
        let manifest = AppManifest::new("My PWA", "https://example.com/");

        assert_eq!(manifest.name(), "My PWA");
        assert_eq!(manifest.start_url(), "https://example.com/");
        assert!(!manifest.is_installable()); // No icons

        // Stub manifest should be installable
        let stub = AppManifest::stub("https://example.com/");
        assert!(stub.is_installable());
    }

    #[test]
    fn test_manifest_json() {
        let json = r#"{
            "name": "Test App",
            "short_name": "Test",
            "start_url": "/",
            "display": "standalone",
            "icons": [
                {
                    "src": "/icon.png",
                    "sizes": "192x192",
                    "type": "image/png"
                }
            ]
        }"#;

        let manifest = AppManifest::from_json(json).unwrap();
        assert_eq!(manifest.name(), "Test App");
        assert!(manifest.is_installable());
    }
}
