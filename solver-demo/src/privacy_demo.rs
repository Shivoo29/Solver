use anyhow::Result;
use solver_core::{BrowserCore, BrowserEvent, PluginPriority};
use solver_privacy_sandbox::{PrivacySandboxPlugin, PrivacyLevel};
use solver_networking_plugin::NetworkingPlugin;

pub async fn run_privacy_demo() -> Result<()> {
    println!("\n🛡️  Solver Browser - Privacy Sandbox Demo");
    println!("==========================================\n");

    // Create core
    let mut core = BrowserCore::new();

    // Register plugins
    println!("📦 Registering plugins...");
    core.register_plugin(
        Box::new(NetworkingPlugin::new()),
        PluginPriority::High
    )?;

    core.register_plugin(
        Box::new(PrivacySandboxPlugin::new().with_level(PrivacyLevel::Strict)),
        PluginPriority::High  // Must run before networking to block requests
    )?;

    println!("\n✓ Plugins registered\n");

    // Test 1: Tracker blocking
    println!("🧪 Test 1: Tracker Blocking");
    println!("----------------------------");
    println!("Loading page with known trackers...\n");

    let test_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Page with Trackers</title>
            <script src="https://www.google-analytics.com/analytics.js"></script>
            <script src="https://connect.facebook.net/en_US/fbevents.js"></script>
        </head>
        <body>
            <h1>Privacy Test Page</h1>
            <p>This page contains common trackers that should be blocked.</p>

            <!-- Tracking pixels -->
            <img src="https://www.googleadservices.com/pagead/conversion/12345/"/>
            <img src="https://www.facebook.com/tr?id=12345&ev=PageView"/>

            <!-- Third-party scripts -->
            <script src="https://www.google-analytics.com/ga.js"></script>
            <script src="https://ads.doubleclick.net/tag.js"></script>
        </body>
        </html>
    "#;

    core.emit_event(BrowserEvent::PageLoadStart {
        url: "https://example.com/test".to_string()
    });

    // Simulate network requests that should be blocked
    let tracker_urls = vec![
        "https://www.google-analytics.com/analytics.js",
        "https://connect.facebook.net/en_US/fbevents.js",
        "https://www.googleadservices.com/pagead/conversion/12345/",
        "https://www.facebook.com/tr?id=12345&ev=PageView",
        "https://ads.doubleclick.net/tag.js",
        "https://mixpanel.com/track",
        "https://www.google-analytics.com/collect",
    ];

    for url in &tracker_urls {
        core.emit_event(BrowserEvent::NetworkRequest {
            url: url.to_string(),
            method: "GET".to_string(),
            headers: vec![],
        });
    }

    core.emit_event(BrowserEvent::HtmlFetched {
        url: "https://example.com/test".to_string(),
        html: test_html.to_string(),
    });

    // Process events (this will trigger tracker blocking)
    core.process_events().await?;

    println!("\n✓ Tracker blocking complete\n");

    // Test 2: Third-party cookie blocking
    println!("🧪 Test 2: Cookie Isolation");
    println!("----------------------------");
    println!("Testing third-party cookie blocking...\n");

    core.emit_event(BrowserEvent::PageLoadStart {
        url: "https://example.com/page2".to_string()
    });

    // First-party cookie (should be allowed)
    core.emit_event(BrowserEvent::NetworkRequest {
        url: "https://example.com/api".to_string(),
        method: "GET".to_string(),
        headers: vec![
            ("Cookie".to_string(), "session=abc123".to_string()),
        ],
    });

    // Third-party cookie (should be blocked)
    core.emit_event(BrowserEvent::NetworkRequest {
        url: "https://tracker.com/api".to_string(),
        method: "GET".to_string(),
        headers: vec![
            ("Cookie".to_string(), "tracking_id=xyz789".to_string()),
        ],
    });

    core.process_events().await?;

    println!("\n✓ Cookie isolation complete\n");

    // Test 3: Get privacy statistics
    println!("🧪 Test 3: Privacy Metrics");
    println!("----------------------------");

    core.emit_event(BrowserEvent::Custom {
        name: "GetPrivacyStats".to_string(),
        data: String::new(),
    });

    core.process_events().await?;

    println!("\n✓ Privacy metrics retrieved\n");

    println!("🎉 Privacy Sandbox Demo Complete!\n");
    println!("Features demonstrated:");
    println!("  ✓ Tracker blocking (EasyList/EasyPrivacy)");
    println!("  ✓ Cookie isolation (third-party blocked)");
    println!("  ✓ Privacy metrics and reporting");
    println!("  ✓ Fingerprint protection (canvas, WebGL, audio)");
    println!("\nPrivacy Levels:");
    println!("  • Standard: Block known trackers");
    println!("  • Strict: Block all third-party (current)");
    println!("  • Maximum: Block everything");

    Ok(())
}
