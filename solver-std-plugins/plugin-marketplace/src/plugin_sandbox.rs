use std::collections::HashSet;

/// Plugin Security Sandbox
pub struct PluginSandbox {
    trusted_authors: HashSet<String>,
    blacklist: HashSet<String>,
    security_level: SecurityLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    Strict,   // Only verified plugins
    Standard, // Most plugins allowed
    Permissive, // All plugins allowed
}

impl PluginSandbox {
    pub fn new() -> Self {
        let mut trusted_authors = HashSet::new();
        trusted_authors.insert("solver-team".to_string());
        trusted_authors.insert("verified".to_string());

        Self {
            trusted_authors,
            blacklist: HashSet::new(),
            security_level: SecurityLevel::Standard,
        }
    }

    /// Check if a plugin is safe to install
    pub fn is_safe(&self, plugin_id: &str) -> bool {
        // Check blacklist
        if self.blacklist.contains(plugin_id) {
            eprintln!("[Sandbox] Plugin blacklisted: {}", plugin_id);
            return false;
        }

        // In Strict mode, only trusted authors
        if self.security_level == SecurityLevel::Strict {
            // Extract author from plugin_id (format: author/plugin-name)
            if let Some(author) = plugin_id.split('/').next() {
                if !self.trusted_authors.contains(author) {
                    eprintln!("[Sandbox] Author not trusted: {}", author);
                    return false;
                }
            }
        }

        // Check for suspicious patterns
        if self.contains_suspicious_patterns(plugin_id) {
            eprintln!("[Sandbox] Suspicious patterns detected: {}", plugin_id);
            return false;
        }

        true
    }

    /// Check for suspicious patterns in plugin ID
    fn contains_suspicious_patterns(&self, plugin_id: &str) -> bool {
        let suspicious = [
            "malware",
            "virus",
            "hack",
            "crack",
            "keylogger",
            "backdoor",
        ];

        let plugin_lower = plugin_id.to_lowercase();
        suspicious.iter().any(|&s| plugin_lower.contains(s))
    }

    /// Add to trusted authors
    pub fn trust_author(&mut self, author: &str) {
        self.trusted_authors.insert(author.to_string());
    }

    /// Add to blacklist
    pub fn blacklist(&mut self, plugin_id: &str) {
        self.blacklist.insert(plugin_id.to_string());
    }

    /// Set security level
    pub fn set_security_level(&mut self, level: SecurityLevel) {
        self.security_level = level;
    }

    /// Get security level
    pub fn security_level(&self) -> SecurityLevel {
        self.security_level
    }

    /// Check if author is trusted
    pub fn is_trusted_author(&self, author: &str) -> bool {
        self.trusted_authors.contains(author)
    }

    /// Check if blacklisted
    pub fn is_blacklisted(&self, plugin_id: &str) -> bool {
        self.blacklist.contains(plugin_id)
    }
}

impl Default for PluginSandbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_sandbox() {
        let mut sandbox = PluginSandbox::new();

        // Safe plugin
        assert!(sandbox.is_safe("verified/dark-mode"));

        // Suspicious plugin
        assert!(!sandbox.is_safe("evil/malware-plugin"));

        // Blacklist
        sandbox.blacklist("bad/plugin");
        assert!(!sandbox.is_safe("bad/plugin"));

        // Strict mode
        sandbox.set_security_level(SecurityLevel::Strict);
        assert!(!sandbox.is_safe("unknown/plugin"));

        sandbox.trust_author("unknown");
        assert!(sandbox.is_safe("unknown/plugin"));
    }
}
