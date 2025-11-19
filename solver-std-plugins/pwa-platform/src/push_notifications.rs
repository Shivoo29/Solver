use std::collections::HashMap;

/// Push Notifications
///
/// Manage push notification subscriptions
pub struct PushNotifications {
    subscriptions: HashMap<String, PushSubscription>,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PushSubscription {
    pub endpoint: String,
    pub keys: PushKeys,
}

#[derive(Debug, Clone)]
pub struct PushKeys {
    pub p256dh: String,
    pub auth: String,
}

impl PushNotifications {
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            enabled: true,
        }
    }

    /// Subscribe to push notifications
    pub fn subscribe(&mut self, origin: &str, endpoint: &str) {
        let subscription = PushSubscription {
            endpoint: endpoint.to_string(),
            keys: PushKeys {
                p256dh: "mock-p256dh-key".to_string(),
                auth: "mock-auth-key".to_string(),
            },
        };

        self.subscriptions.insert(origin.to_string(), subscription);
        eprintln!("[Push] ✓ Subscribed: {}", origin);
    }

    /// Unsubscribe from push notifications
    pub fn unsubscribe(&mut self, origin: &str) {
        self.subscriptions.remove(origin);
        eprintln!("[Push] ✗ Unsubscribed: {}", origin);
    }

    /// Get subscription
    pub fn get_subscription(&self, origin: &str) -> Option<&PushSubscription> {
        self.subscriptions.get(origin)
    }

    /// Check if subscribed
    pub fn is_subscribed(&self, origin: &str) -> bool {
        self.subscriptions.contains_key(origin)
    }

    /// Get all subscriptions
    pub fn all_subscriptions(&self) -> Vec<String> {
        self.subscriptions.keys().cloned().collect()
    }

    /// Send push notification (mock)
    pub fn send_notification(&self, origin: &str, title: &str, body: &str) {
        if self.is_subscribed(origin) {
            eprintln!("[Push] 🔔 Notification: {} - {}", title, body);
        } else {
            eprintln!("[Push] ⚠️ Not subscribed: {}", origin);
        }
    }

    /// Enable/disable push
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for PushNotifications {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_notifications() {
        let mut push = PushNotifications::new();

        assert!(!push.is_subscribed("https://example.com"));

        // Subscribe
        push.subscribe("https://example.com", "https://push.example.com/endpoint");
        assert!(push.is_subscribed("https://example.com"));

        // Get subscription
        let sub = push.get_subscription("https://example.com");
        assert!(sub.is_some());

        // Unsubscribe
        push.unsubscribe("https://example.com");
        assert!(!push.is_subscribed("https://example.com"));
    }
}
