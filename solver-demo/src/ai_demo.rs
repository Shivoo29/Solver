use anyhow::Result;
use solver_core::{BrowserCore, BrowserEvent, PluginPriority};
use solver_ai_assistant::{AIAssistantPlugin, AIMode};
use solver_networking_plugin::NetworkingPlugin;

pub async fn run_ai_demo() -> Result<()> {
    println!("\n🤖 Solver Browser - AI Assistant Demo");
    println!("=====================================\n");

    // Create core
    let mut core = BrowserCore::new();

    // Register plugins
    println!("📦 Registering plugins...");
    core.register_plugin(
        Box::new(NetworkingPlugin::new()),
        PluginPriority::High
    )?;

    core.register_plugin(
        Box::new(AIAssistantPlugin::new().with_mode(AIMode::Cloud)),
        PluginPriority::Normal
    )?;

    println!("\n✓ Plugins registered\n");

    // Test page summarization
    println!("🧪 Test 1: Page Summarization");
    println!("------------------------------");

    let test_html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Rust Programming</title></head>
        <body>
            <h1>Introduction to Rust</h1>
            <p>Rust is a systems programming language that runs blazingly fast, prevents segfaults,
               and guarantees thread safety. It achieves memory safety without using garbage collection.</p>

            <h2>Key Features</h2>
            <ul>
                <li>Zero-cost abstractions</li>
                <li>Move semantics</li>
                <li>Guaranteed memory safety</li>
                <li>Threads without data races</li>
                <li>Trait-based generics</li>
            </ul>

            <p>Rust is used in production by hundreds of companies including Mozilla, Dropbox,
               and Cloudflare. It's particularly popular for building web servers, embedded systems,
               and blockchain applications.</p>
        </body>
        </html>
    "#;

    core.emit_event(BrowserEvent::PageLoadStart {
        url: "https://example.com/rust".to_string()
    });

    core.emit_event(BrowserEvent::HtmlFetched {
        url: "https://example.com/rust".to_string(),
        html: test_html.to_string(),
    });

    // Process events (this will trigger AI summarization)
    core.process_events().await?;

    println!("\n✓ Page summarization complete\n");

    // Test Q&A
    println!("🧪 Test 2: Question Answering");
    println!("------------------------------");
    println!("Question: What is Rust used for?");

    core.emit_event(BrowserEvent::Custom {
        name: "AIQuery".to_string(),
        data: "What is Rust primarily used for?".to_string(),
    });

    core.process_events().await?;

    println!("\n✓ Q&A complete\n");

    println!("🎉 AI Assistant Demo Complete!\n");
    println!("Features demonstrated:");
    println!("  ✓ Cloud AI integration (Gemini)");
    println!("  ✓ Page summarization");
    println!("  ✓ Q&A about page content");
    println!("  ✓ Privacy-first design (local mode available)");
    println!("\nNote: Set GEMINI_API_KEY environment variable to use cloud AI");
    println!("      Or enable local-ai feature to use Llama 3.2 offline");

    Ok(())
}
