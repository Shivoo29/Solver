use std::collections::HashMap;

/// Cookie jar with per-domain isolation
pub struct CookieJar {
    // Domain -> Cookie storage
    cookies: HashMap<String, Vec<Cookie>>,

    // Allow third-party cookies?
    allow_third_party: bool,
}

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl CookieJar {
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
            allow_third_party: false,  // Block by default
        }
    }

    /// Filter cookies from request headers
    /// Removes third-party cookies based on policy
    pub fn filter_cookies(
        &mut self,
        headers: &[(String, String)],
        request_url: &str,
        page_url: &str,
    ) -> Vec<(String, String)> {
        let mut filtered = Vec::new();
        let is_third_party = self.is_third_party(request_url, page_url);

        for (key, value) in headers {
            if key.to_lowercase() == "cookie" {
                // Filter cookies
                if !is_third_party || self.allow_third_party {
                    filtered.push((key.clone(), value.clone()));
                } else {
                    eprintln!("[Cookie Jar] 🍪 Blocked third-party cookie for: {}", request_url);
                }
            } else {
                // Keep other headers
                filtered.push((key.clone(), value.clone()));
            }
        }

        filtered
    }

    /// Store a cookie
    pub fn store(&mut self, cookie: Cookie) {
        let domain = cookie.domain.clone();
        self.cookies
            .entry(domain)
            .or_insert_with(Vec::new)
            .push(cookie);
    }

    /// Get cookies for a domain
    pub fn get(&self, domain: &str) -> Vec<Cookie> {
        self.cookies
            .get(domain)
            .cloned()
            .unwrap_or_default()
    }

    /// Clear all cookies
    pub fn clear_all(&mut self) {
        self.cookies.clear();
    }

    /// Clear cookies for a domain
    pub fn clear_domain(&mut self, domain: &str) {
        self.cookies.remove(domain);
    }

    /// Check if request is third-party
    fn is_third_party(&self, request_url: &str, page_url: &str) -> bool {
        if let (Ok(req), Ok(page)) = (
            url::Url::parse(request_url),
            url::Url::parse(page_url)
        ) {
            if let (Some(req_domain), Some(page_domain)) = (
                req.host_str(),
                page.host_str()
            ) {
                return req_domain != page_domain;
            }
        }
        false
    }

    /// Get statistics
    pub fn stats(&self) -> CookieStats {
        let total: usize = self.cookies.values().map(|v| v.len()).sum();
        CookieStats {
            domains: self.cookies.len(),
            total_cookies: total,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CookieStats {
    pub domains: usize,
    pub total_cookies: usize,
}

impl Default for CookieJar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_isolation() {
        let mut jar = CookieJar::new();

        let headers = vec![
            ("Cookie".to_string(), "session=abc123".to_string()),
        ];

        // Same-party: should allow
        let filtered = jar.filter_cookies(
            &headers,
            "https://example.com/api",
            "https://example.com/page",
        );
        assert_eq!(filtered.len(), 1);

        // Third-party: should block
        let filtered = jar.filter_cookies(
            &headers,
            "https://tracker.com/api",
            "https://example.com/page",
        );
        assert_eq!(filtered.len(), 0);
    }
}
