use anyhow::{Context, Result};
use shared::{BrowserMessage, RendererMessage};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

#[cfg(feature = "gui")]
mod gui;

fn main() -> Result<()> {
    #[cfg(feature = "gui")]
    {
        return gui::run_gui();
    }

    #[cfg(not(feature = "gui"))]
    {
        run_headless()
    }
}

#[cfg(not(feature = "gui"))]
fn run_headless() -> Result<()> {
    use std::env;

    println!("Solver Browser - Headless Mode");
    println!("==============================\n");

    let args: Vec<String> = env::args().collect();

    let (url, output_file) = if args.len() >= 3 {
        (args[1].clone(), args[2].clone())
    } else {
        println!("Usage: {} <url> <output.png>", args.get(0).unwrap_or(&"solver".to_string()));
        println!("Example: {} test output.png", args.get(0).unwrap_or(&"solver".to_string()));
        println!("\nUsing defaults: url=test, output=output.png");
        ("test".to_string(), "output.png".to_string())
    };

    println!("Fetching: {}", url);
    let html = fetch_url(&url)?;

    println!("Starting renderer process...");
    let mut renderer = RendererProcess::start()?;

    println!("Rendering page...");
    let (width, height) = (1024, 768);

    renderer.send(&BrowserMessage::RenderHtml {
        url: url.clone(),
        html,
        width,
        height,
    })?;

    match renderer.receive()? {
        RendererMessage::FrameReady { width, height, pixels } => {
            println!("Received rendered frame: {}x{}", width, height);

            // Save to PNG file
            save_to_png(&output_file, &pixels, width, height)?;

            println!("\n✓ Successfully rendered page to: {}", output_file);
            println!("  Dimensions: {}x{}", width, height);
        }
        RendererMessage::Error { message } => {
            eprintln!("Renderer error: {}", message);
        }
        _ => {}
    }

    renderer.shutdown()?;
    Ok(())
}

struct RendererProcess {
    process: Child,
    stdin: ChildStdin,
}

impl RendererProcess {
    fn start() -> Result<Self> {
        let renderer_path = find_renderer_binary()?;

        let mut child = Command::new(renderer_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn renderer process")?;

        let stdin = child.stdin.take().context("Failed to get renderer stdin")?;
        let stdout = child.stdout.take().context("Failed to get renderer stdout")?;

        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let response: RendererMessage = serde_json::from_str(&line)?;
        if !matches!(response, RendererMessage::Ready) {
            anyhow::bail!("Expected Ready message from renderer");
        }

        // Put stdout back
        child.stdout = Some(reader.into_inner());

        let process = RendererProcess {
            process: child,
            stdin,
        };

        Ok(process)
    }

    fn send(&mut self, message: &BrowserMessage) -> Result<()> {
        let json = serde_json::to_string(message)?;
        writeln!(self.stdin, "{}", json)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn receive(&mut self) -> Result<RendererMessage> {
        let stdout = self.process.stdout.take().context("No stdout from renderer")?;

        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let response: RendererMessage = serde_json::from_str(&line)?;

        // Put stdout back
        self.process.stdout = Some(reader.into_inner());

        Ok(response)
    }

    fn shutdown(mut self) -> Result<()> {
        let _ = self.send(&BrowserMessage::Shutdown);
        let _ = self.process.wait();
        Ok(())
    }
}

fn fetch_url(url: &str) -> Result<String> {
    // Handle simple test case
    if url.starts_with("test:") || url == "test" {
        return Ok(r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        h1 { color: blue; font-size: 32px; }
        p { color: green; font-size: 16px; }
        .highlight { background-color: yellow; color: red; }
        div { margin: 10px; padding: 5px; }
    </style>
    <script>
        console.log('Solver Browser JavaScript is WORKING!');
        console.log('1 + 1 =', 1 + 1);
        var message = 'Hello from JavaScript!';
        console.log(message);
        alert('JavaScript Enabled!');
    </script>
</head>
<body>
    <h1>Welcome to Solver Browser!</h1>
    <p>This is a minimal web browser built with Rust.</p>
    <div class="highlight">Security, Privacy, and Performance - Built Right.</div>
    <p>Image Rendering Test:</p>
    <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAKCAYAAACNMs+9AAAAFUlEQVR42mP8z8BQz0AEYBxVSF+FABJADveWkH6oAAAAAElFTkSuQmCC" alt="Test"/>
    <p>Features:</p>
    <div>
        <p>Memory-safe Rust rendering engine</p>
        <p>Multi-process sandboxed architecture</p>
        <p>No telemetry or tracking</p>
        <p>JavaScript engine (QuickJS via rquickjs) - WORKING!</p>
        <p>Image rendering - TESTING!</p>
    </div>
    <script>
        console.log('Second script tag also executing!');
        var x = 10;
        var y = 20;
        console.log('Math test: 10 + 20 =', x + y);
    </script>
</body>
</html>
        "#.to_string());
    }

    // Add http:// if no scheme
    let url = if !url.contains("://") {
        format!("http://{}", url)
    } else {
        url.to_string()
    };

    // Fetch from network
    let response = reqwest::blocking::get(&url)
        .context("Failed to fetch URL")?;

    let html = response.text().context("Failed to read response body")?;
    Ok(html)
}

fn find_renderer_binary() -> Result<String> {
    // Try common locations
    let locations = [
        "./target/debug/renderer",
        "./target/release/renderer",
        "../target/debug/renderer",
        "../target/release/renderer",
        "../../target/debug/renderer",
        "../../target/release/renderer",
    ];

    for loc in &locations {
        if std::path::Path::new(loc).exists() {
            return Ok(loc.to_string());
        }
    }

    anyhow::bail!("Renderer binary not found. Please build the project first with 'cargo build'")
}

fn save_to_png(filename: &str, pixels: &[u8], width: u32, height: u32) -> Result<()> {
    use image::{ImageBuffer, Rgba};

    let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(width, height, pixels.to_vec())
        .context("Failed to create image buffer")?;

    img.save(filename)
        .context("Failed to save PNG file")?;

    Ok(())
}
