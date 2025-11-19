use anyhow::Result;
use clap::Parser;
use solver_core::{BrowserCore, BrowserEvent, PluginPriority};
use solver_networking_plugin::NetworkingPlugin;
use solver_rendering_plugin::RenderingPlugin;
use solver_javascript_plugin::JavaScriptPlugin;

#[derive(Parser, Debug)]
#[command(name = "Solver Browser Demo")]
#[command(about = "Demonstrating the plugin architecture", long_about = None)]
struct Args {
    /// URL to load
    url: String,

    /// Output PNG file
    #[arg(short, long, default_value = "output.png")]
    output: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("Solver Browser - Plugin Architecture Demo");
    println!("=========================================\n");

    // Create the browser core
    let mut core = BrowserCore::new();

    println!("Registering plugins...");

    // Register plugins in priority order
    // Higher priority = executes first

    // 1. Networking (HIGH) - needs to fetch HTML first
    core.register_plugin(
        Box::new(NetworkingPlugin::new()),
        PluginPriority::High
    )?;

    // 2. Rendering (NORMAL) - processes HTML
    core.register_plugin(
        Box::new(RenderingPlugin::new()),
        PluginPriority::Normal
    )?;

    // 3. JavaScript (NORMAL) - executes scripts
    core.register_plugin(
        Box::new(JavaScriptPlugin::new()),
        PluginPriority::Normal
    )?;

    println!("\nRegistered plugins:");
    for plugin in core.list_plugins() {
        println!("  - {}", plugin);
    }

    println!("\nLoading URL: {}", args.url);

    // Emit navigation event
    core.emit_event(BrowserEvent::NavigationRequested {
        url: args.url.clone()
    });

    // Emit page load start
    core.emit_event(BrowserEvent::PageLoadStart {
        url: args.url.clone()
    });

    // Process all events
    println!("\nProcessing events...");
    core.process_events().await?;

    println!("\n✓ Demo complete!");
    println!("\nThis demonstrates:");
    println!("  1. Modular plugin architecture");
    println!("  2. Event-driven browser engine");
    println!("  3. Plugin priority system");
    println!("  4. Extensible design");
    println!("\nNext steps:");
    println!("  - Integrate full rendering pipeline");
    println!("  - Add AI assistant plugin");
    println!("  - Add privacy sandbox plugin");
    println!("  - Build plugin manager UI");

    Ok(())
}
