use anyhow::Result;
use std::collections::HashMap;

/// Service Worker
///
/// Intercepts network requests and provides offline caching
#[derive(Debug, Clone)]
pub struct ServiceWorker {
    scope: String,
    script_url: String,
    cache: HashMap<String, String>,
    state: ServiceWorkerState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceWorkerState {
    Installing,
    Installed,
    Activating,
    Activated,
    Redundant,
}

#[derive(Debug, Clone)]
pub struct ServiceWorkerScope {
    pub path: String,
}

impl ServiceWorker {
    pub fn new(scope: &str, script_url: &str) -> Result<Self> {
        Ok(Self {
            scope: scope.to_string(),
            script_url: script_url.to_string(),
            cache: HashMap::new(),
            state: ServiceWorkerState::Installing,
        })
    }

    pub fn scope(&self) -> &str {
        &self.scope
    }

    pub fn script_url(&self) -> &str {
        &self.script_url
    }

    pub fn state(&self) -> ServiceWorkerState {
        self.state
    }

    /// Activate the service worker
    pub fn activate(&mut self) {
        self.state = ServiceWorkerState::Activated;
    }

    /// Cache a resource
    pub fn cache(&mut self, url: &str, content: &str) {
        self.cache.insert(url.to_string(), content.to_string());
    }

    /// Get cached resource
    pub fn get_cached(&self, url: &str) -> Option<String> {
        self.cache.get(url).cloned()
    }

    /// Check if URL is cached
    pub fn is_cached(&self, url: &str) -> bool {
        self.cache.contains_key(url)
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Get cached URLs
    pub fn cached_urls(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }

    /// Intercept fetch request
    pub fn intercept_fetch(&self, url: &str) -> FetchResult {
        if let Some(cached) = self.get_cached(url) {
            FetchResult::Cached(cached)
        } else {
            FetchResult::Network
        }
    }
}

#[derive(Debug, Clone)]
pub enum FetchResult {
    Cached(String),
    Network,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_worker() {
        let mut sw = ServiceWorker::new("/", "/sw.js").unwrap();

        assert_eq!(sw.scope(), "/");
        assert_eq!(sw.cache_size(), 0);

        // Cache a resource
        sw.cache("https://example.com/", "<html>test</html>");
        assert_eq!(sw.cache_size(), 1);
        assert!(sw.is_cached("https://example.com/"));

        // Get cached
        let cached = sw.get_cached("https://example.com/");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), "<html>test</html>");

        // Intercept fetch
        match sw.intercept_fetch("https://example.com/") {
            FetchResult::Cached(content) => {
                assert_eq!(content, "<html>test</html>");
            }
            _ => panic!("Expected cached result"),
        }
    }
}
