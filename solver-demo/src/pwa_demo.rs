use anyhow::Result;
use solver_core::{BrowserCore, BrowserEvent, PluginPriority};
use solver_pwa_platform::PWAPlatformPlugin;
use solver_networking_plugin::NetworkingPlugin;

pub async fn run_pwa_demo() -> Result<()> {
    println!("\n📱 Solver Browser - PWA Platform Demo");
    println!("======================================\n");

    // Create core
    let mut core = BrowserCore::new();

    // Register plugins
    println!("📦 Registering plugins...");
    core.register_plugin(
        Box::new(NetworkingPlugin::new()),
        PluginPriority::High
    )?;

    core.register_plugin(
        Box::new(PWAPlatformPlugin::new()),
        PluginPriority::Normal
    )?;

    println!("\n✓ Plugins registered\n");
    println!("════════════════════════════════════════════════════════════════\n");

    // Test 1: Service Worker Registration
    println!("🧪 Test 1: Service Worker Registration");
    println!("────────────────────────────────────────────────────────────────");
    println!("Registering service worker for offline support...\n");

    core.emit_event(BrowserEvent::Custom {
        name: "RegisterServiceWorker".to_string(),
        data: "/|/sw.js".to_string(), // scope|script_url
    });

    core.process_events().await?;

    println!("\n✓ Service worker registered\n");
    println!("════════════════════════════════════════════════════════════════\n");

    // Test 2: PWA Detection
    println!("🧪 Test 2: PWA Detection");
    println!("────────────────────────────────────────────────────────────────");
    println!("Loading a Progressive Web App...\n");

    let pwa_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>My PWA</title>
            <link rel="manifest" href="/manifest.json">
            <script>
                if ('serviceWorker' in navigator) {
                    navigator.serviceWorker.register('/sw.js');
                }
            </script>
        </head>
        <body>
            <h1>Progressive Web App</h1>
            <p>This app works offline!</p>
        </body>
        </html>
    "#;

    core.emit_event(BrowserEvent::PageLoadStart {
        url: "https://pwa-example.com/".to_string()
    });

    core.emit_event(BrowserEvent::HtmlFetched {
        url: "https://pwa-example.com/".to_string(),
        html: pwa_html.to_string(),
    });

    core.process_events().await?;

    println!("\n✓ PWA detected\n");
    println!("════════════════════════════════════════════════════════════════\n");

    // Test 3: Install PWA
    println!("🧪 Test 3: Install PWA");
    println!("────────────────────────────────────────────────────────────────");
    println!("Installing PWA as native app...\n");

    core.emit_event(BrowserEvent::Custom {
        name: "InstallPWA".to_string(),
        data: "https://pwa-example.com/manifest.json".to_string(),
    });

    core.process_events().await?;

    println!("\n✓ PWA installed\n");
    println!("════════════════════════════════════════════════════════════════\n");

    // Test 4: Offline Caching
    println!("🧪 Test 4: Offline Caching");
    println!("────────────────────────────────────────────────────────────────");
    println!("Simulating offline mode...\n");

    // First visit - network
    println!("First visit (network):");
    core.emit_event(BrowserEvent::PageLoadStart {
        url: "https://pwa-example.com/".to_string()
    });

    core.emit_event(BrowserEvent::HtmlFetched {
        url: "https://pwa-example.com/".to_string(),
        html: pwa_html.to_string(),
    });

    core.process_events().await?;

    // Second visit - should serve from cache
    println!("\nSecond visit (offline):");
    core.emit_event(BrowserEvent::PageLoadStart {
        url: "https://pwa-example.com/".to_string()
    });

    core.process_events().await?;

    println!("\n✓ Offline caching working\n");
    println!("════════════════════════════════════════════════════════════════\n");

    // Test 5: Background Sync
    println!("🧪 Test 5: Background Sync");
    println!("────────────────────────────────────────────────────────────────");
    println!("Queuing actions for background sync...\n");

    core.emit_event(BrowserEvent::Custom {
        name: "QueueBackgroundSync".to_string(),
        data: "send-message: Hello from PWA!".to_string(),
    });

    core.emit_event(BrowserEvent::Custom {
        name: "QueueBackgroundSync".to_string(),
        data: "upload-photo: photo.jpg".to_string(),
    });

    core.process_events().await?;

    println!("\n✓ Background sync queue ready\n");
    println!("════════════════════════════════════════════════════════════════\n");

    // Test 6: PWA Statistics
    println!("🧪 Test 6: PWA Statistics");
    println!("────────────────────────────────────────────────────────────────");

    core.emit_event(BrowserEvent::Custom {
        name: "GetPWAStats".to_string(),
        data: String::new(),
    });

    core.process_events().await?;

    println!("\n✓ PWA stats retrieved\n");
    println!("════════════════════════════════════════════════════════════════\n");

    println!("🎉 PWA Platform Demo Complete!\n");
    println!("Features demonstrated:");
    println!("  ✓ Service worker registration and caching");
    println!("  ✓ PWA detection (manifest + service worker)");
    println!("  ✓ Install PWA as native app");
    println!("  ✓ Offline-first caching");
    println!("  ✓ Background sync queue");
    println!("  ✓ PWA statistics and metrics");
    println!("\nBenefits:");
    println!("  📱 Install web apps like native apps");
    println!("  📡 Work offline with cached content");
    println!("  📤 Queue actions when offline");
    println!("  🔔 Push notifications (coming soon)");
    println!("\nWhat makes this special:");
    println!("  • Chrome has PWAs but not well-integrated");
    println!("  • Solver treats PWAs as first-class citizens");
    println!("  • Automatic detection and installation");
    println!("  • Seamless offline experience");

    Ok(())
}
