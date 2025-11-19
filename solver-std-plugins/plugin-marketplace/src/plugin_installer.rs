use anyhow::Result;
use std::collections::HashMap;

/// Plugin Installer
pub struct PluginInstaller {
    installed: HashMap<String, InstalledPlugin>,
}

#[derive(Debug, Clone)]
pub struct InstalledPlugin {
    pub id: String,
    pub version: String,
    pub install_date: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStatus {
    Downloading,
    Installing,
    Installed,
    Failed,
}

impl PluginInstaller {
    pub fn new() -> Self {
        Self {
            installed: HashMap::new(),
        }
    }

    /// Install a plugin
    pub fn install(&mut self, plugin_id: &str) -> Result<()> {
        // Check if already installed
        if self.is_installed(plugin_id) {
            return Err(anyhow::anyhow!("Plugin already installed: {}", plugin_id));
        }

        // In real implementation:
        // 1. Download plugin from marketplace
        // 2. Verify signature
        // 3. Extract and install
        // 4. Run security checks
        // 5. Enable plugin

        // For now, mock installation
        let plugin = InstalledPlugin {
            id: plugin_id.to_string(),
            version: "1.0.0".to_string(),
            install_date: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            enabled: true,
        };

        self.installed.insert(plugin_id.to_string(), plugin);

        Ok(())
    }

    /// Uninstall a plugin
    pub fn uninstall(&mut self, plugin_id: &str) -> Result<()> {
        if !self.is_installed(plugin_id) {
            return Err(anyhow::anyhow!("Plugin not installed: {}", plugin_id));
        }

        self.installed.remove(plugin_id);

        Ok(())
    }

    /// Check if plugin is installed
    pub fn is_installed(&self, plugin_id: &str) -> bool {
        self.installed.contains_key(plugin_id)
    }

    /// Enable a plugin
    pub fn enable(&mut self, plugin_id: &str) -> Result<()> {
        let plugin = self.installed.get_mut(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not installed: {}", plugin_id))?;

        plugin.enabled = true;

        Ok(())
    }

    /// Disable a plugin
    pub fn disable(&mut self, plugin_id: &str) -> Result<()> {
        let plugin = self.installed.get_mut(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not installed: {}", plugin_id))?;

        plugin.enabled = false;

        Ok(())
    }

    /// Get installed plugin
    pub fn get(&self, plugin_id: &str) -> Option<&InstalledPlugin> {
        self.installed.get(plugin_id)
    }

    /// Get all installed plugins
    pub fn all(&self) -> Vec<InstalledPlugin> {
        self.installed.values().cloned().collect()
    }

    /// Get installed count
    pub fn installed_count(&self) -> usize {
        self.installed.len()
    }

    /// Get enabled count
    pub fn enabled_count(&self) -> usize {
        self.installed.values().filter(|p| p.enabled).count()
    }
}

impl Default for PluginInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_installer() {
        let mut installer = PluginInstaller::new();

        // Install
        installer.install("dark-mode").unwrap();
        assert!(installer.is_installed("dark-mode"));
        assert_eq!(installer.installed_count(), 1);

        // Disable
        installer.disable("dark-mode").unwrap();
        assert_eq!(installer.enabled_count(), 0);

        // Enable
        installer.enable("dark-mode").unwrap();
        assert_eq!(installer.enabled_count(), 1);

        // Uninstall
        installer.uninstall("dark-mode").unwrap();
        assert!(!installer.is_installed("dark-mode"));
        assert_eq!(installer.installed_count(), 0);
    }
}
