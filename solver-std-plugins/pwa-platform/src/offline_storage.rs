use std::collections::HashMap;

/// Offline Storage
///
/// IndexedDB-like storage for offline data
pub struct OfflineStorage {
    stores: HashMap<String, HashMap<String, String>>,
}

impl OfflineStorage {
    pub fn new() -> Self {
        Self {
            stores: HashMap::new(),
        }
    }

    /// Create or open a store
    pub fn open_store(&mut self, name: &str) {
        if !self.stores.contains_key(name) {
            self.stores.insert(name.to_string(), HashMap::new());
        }
    }

    /// Put data in store
    pub fn put(&mut self, store: &str, key: &str, value: &str) {
        if !self.stores.contains_key(store) {
            self.open_store(store);
        }

        if let Some(s) = self.stores.get_mut(store) {
            s.insert(key.to_string(), value.to_string());
        }
    }

    /// Get data from store
    pub fn get(&self, key: &str) -> Option<String> {
        // Search all stores for the key
        for store in self.stores.values() {
            if let Some(value) = store.get(key) {
                return Some(value.clone());
            }
        }
        None
    }

    /// Get from specific store
    pub fn get_from_store(&self, store: &str, key: &str) -> Option<String> {
        self.stores.get(store)?.get(key).cloned()
    }

    /// Delete from store
    pub fn delete(&mut self, store: &str, key: &str) {
        if let Some(s) = self.stores.get_mut(store) {
            s.remove(key);
        }
    }

    /// Clear store
    pub fn clear_store(&mut self, store: &str) {
        if let Some(s) = self.stores.get_mut(store) {
            s.clear();
        }
    }

    /// Get all keys in store
    pub fn keys(&self, store: &str) -> Vec<String> {
        self.stores
            .get(store)
            .map(|s| s.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get total size (all stores)
    pub fn size(&self) -> usize {
        self.stores.values().map(|s| s.len()).sum()
    }

    /// Get store size
    pub fn store_size(&self, store: &str) -> usize {
        self.stores.get(store).map(|s| s.len()).unwrap_or(0)
    }

    /// List all stores
    pub fn list_stores(&self) -> Vec<String> {
        self.stores.keys().cloned().collect()
    }
}

impl Default for OfflineStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_storage() {
        let mut storage = OfflineStorage::new();

        // Open store
        storage.open_store("pages");
        assert_eq!(storage.list_stores().len(), 1);

        // Put data
        storage.put("pages", "https://example.com/", "<html>test</html>");
        assert_eq!(storage.store_size("pages"), 1);

        // Get data
        let data = storage.get_from_store("pages", "https://example.com/");
        assert!(data.is_some());
        assert_eq!(data.unwrap(), "<html>test</html>");

        // Delete
        storage.delete("pages", "https://example.com/");
        assert_eq!(storage.store_size("pages"), 0);
    }
}
