use anyhow::Result;
use async_trait::async_trait;
use solver_core::{BrowserPlugin, BrowserEvent, BrowserCore, PluginMetadata};
use solver_plugins::plugin_metadata;
use std::sync::Arc;
use parking_lot::RwLock;

mod service_worker;
mod app_manifest;
mod offline_storage;
mod background_sync;
mod push_notifications;

pub use service_worker::{ServiceWorker, ServiceWorkerScope};
pub use app_manifest::{AppManifest, ManifestDisplay};
pub use offline_storage::OfflineStorage;
pub use background_sync::BackgroundSync;
pub use push_notifications::PushNotifications;

/// PWA Platform Plugin
///
/// Provides complete Progressive Web App support:
/// - Service Workers (intercept requests, offline caching)
/// - App Manifests (install as native app)
/// - Offline Storage (IndexedDB)
/// - Background Sync (queue when offline)
/// - Push Notifications (subscription management)
pub struct PWAPlatformPlugin {
    metadata: PluginMetadata,
    service_workers: Arc<RwLock<Vec<ServiceWorker>>>,
    manifests: Arc<RwLock<Vec<AppManifest>>>,
    storage: Arc<RwLock<OfflineStorage>>,
    background_sync: Arc<RwLock<BackgroundSync>>,
    push_notifications: Arc<RwLock<PushNotifications>>,
    enabled: bool,
}

impl PWAPlatformPlugin {
    pub fn new() -> Self {
        Self {
            metadata: plugin_metadata!(
                "PWA Platform",
                "0.1.0",
                "Complete Progressive Web App support with offline capabilities",
                "Solver Team"
            ),
            service_workers: Arc::new(RwLock::new(Vec::new())),
            manifests: Arc::new(RwLock::new(Vec::new())),
            storage: Arc::new(RwLock::new(OfflineStorage::new())),
            background_sync: Arc::new(RwLock::new(BackgroundSync::new())),
            push_notifications: Arc::new(RwLock::new(PushNotifications::new())),
            enabled: true,
        }
    }

    /// Register a service worker for a scope
    pub fn register_service_worker(&mut self, scope: &str, script_url: &str) -> Result<()> {
        let mut workers = self.service_workers.write();

        // Check if already registered
        if workers.iter().any(|w| w.scope() == scope) {
            eprintln!("[PWA] Service worker already registered for scope: {}", scope);
            return Ok(());
        }

        let worker = ServiceWorker::new(scope, script_url)?;
        eprintln!("[PWA] ✓ Registered service worker for scope: {}", scope);
        workers.push(worker);

        Ok(())
    }

    /// Install PWA from manifest
    pub fn install_pwa(&mut self, manifest: AppManifest) -> Result<()> {
        eprintln!("[PWA] Installing PWA: {}", manifest.name());

        let mut manifests = self.manifests.write();
        manifests.push(manifest);

        eprintln!("[PWA] ✓ PWA installed and ready");
        Ok(())
    }

    /// Check if URL is controlled by a service worker
    fn find_service_worker(&self, url: &str) -> Option<ServiceWorker> {
        let workers = self.service_workers.read();

        for worker in workers.iter() {
            if url.starts_with(worker.scope()) {
                return Some(worker.clone());
            }
        }

        None
    }

    /// Get offline-first response if available
    async fn try_offline_first(&self, url: &str) -> Option<String> {
        // Check service worker cache
        if let Some(worker) = self.find_service_worker(url) {
            if let Some(cached) = worker.get_cached(url) {
                eprintln!("[PWA] 📦 Served from cache: {}", url);
                return Some(cached);
            }
        }

        // Check offline storage
        let storage = self.storage.read();
        if let Some(data) = storage.get(url) {
            eprintln!("[PWA] 💾 Served from offline storage: {}", url);
            return Some(data);
        }

        None
    }
}

impl Default for PWAPlatformPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BrowserPlugin for PWAPlatformPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn init(&mut self, _core: &mut BrowserCore) -> Result<()> {
        eprintln!("[PWA Platform] Initializing...");
        eprintln!("[PWA Platform] ✓ Service worker support enabled");
        eprintln!("[PWA Platform] ✓ App manifest support enabled");
        eprintln!("[PWA Platform] ✓ Offline storage enabled");
        eprintln!("[PWA Platform] ✓ Background sync enabled");
        eprintln!("[PWA Platform] ✓ Push notifications enabled");

        Ok(())
    }

    async fn handle_event(&mut self, event: BrowserEvent, core: &mut BrowserCore) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        match event {
            BrowserEvent::PageLoadStart { url } => {
                eprintln!("[PWA] Checking for PWA: {}", url);

                // Try offline-first
                if let Some(cached_content) = self.try_offline_first(&url).await {
                    // Serve from cache
                    core.emit_event(BrowserEvent::HtmlFetched {
                        url: url.clone(),
                        html: cached_content,
                    });

                    core.emit_event(BrowserEvent::Custom {
                        name: "PWAServedOffline".to_string(),
                        data: url,
                    });
                }
            }

            BrowserEvent::HtmlFetched { url, html } => {
                // Check for manifest link
                if html.contains("manifest.json") || html.contains("manifest.webmanifest") {
                    eprintln!("[PWA] 📱 Detected PWA manifest in: {}", url);

                    core.emit_event(BrowserEvent::Custom {
                        name: "PWADetected".to_string(),
                        data: url.clone(),
                    });
                }

                // Check for service worker registration
                if html.contains("serviceWorker.register") {
                    eprintln!("[PWA] 🔧 Detected service worker registration in: {}", url);

                    core.emit_event(BrowserEvent::Custom {
                        name: "ServiceWorkerDetected".to_string(),
                        data: url.clone(),
                    });
                }

                // Cache the content if service worker is registered
                if let Some(worker) = self.find_service_worker(&url) {
                    let mut workers = self.service_workers.write();
                    if let Some(w) = workers.iter_mut().find(|w| w.scope() == worker.scope()) {
                        w.cache(&url, &html);
                        eprintln!("[PWA] 💾 Cached: {}", url);
                    }
                }
            }

            BrowserEvent::Custom { name, data } if name == "RegisterServiceWorker" => {
                // Format: "scope|script_url"
                let parts: Vec<&str> = data.split('|').collect();
                if parts.len() == 2 {
                    self.register_service_worker(parts[0], parts[1])?;

                    core.emit_event(BrowserEvent::Custom {
                        name: "ServiceWorkerRegistered".to_string(),
                        data: parts[0].to_string(),
                    });
                }
            }

            BrowserEvent::Custom { name, data } if name == "InstallPWA" => {
                // Install PWA from manifest URL
                eprintln!("[PWA] Installing PWA from manifest: {}", data);

                // In real implementation: fetch and parse manifest
                // For now, create a stub manifest
                let manifest = AppManifest::stub(&data);
                self.install_pwa(manifest)?;

                core.emit_event(BrowserEvent::Custom {
                    name: "PWAInstalled".to_string(),
                    data,
                });
            }

            BrowserEvent::Custom { name, data } if name == "QueueBackgroundSync" => {
                // Queue an action for background sync
                let mut sync = self.background_sync.write();
                sync.queue(&data);

                eprintln!("[PWA] 📤 Queued for background sync: {}", data);

                core.emit_event(BrowserEvent::Custom {
                    name: "BackgroundSyncQueued".to_string(),
                    data,
                });
            }

            BrowserEvent::Custom { name, data: _ } if name == "GetPWAStats" => {
                let workers = self.service_workers.read();
                let manifests = self.manifests.read();
                let storage = self.storage.read();
                let sync = self.background_sync.read();

                let stats = serde_json::json!({
                    "service_workers": workers.len(),
                    "installed_pwas": manifests.len(),
                    "cached_items": storage.size(),
                    "sync_queue": sync.queue_size(),
                });

                core.emit_event(BrowserEvent::Custom {
                    name: "PWAStats".to_string(),
                    data: stats.to_string(),
                });
            }

            _ => {}
        }

        Ok(())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
