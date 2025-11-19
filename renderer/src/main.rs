mod css_parser;
mod dom;
mod html_parser;
mod layout;
mod render;
mod style;
mod js_engine;
mod images;
mod fonts;

use anyhow::Result;
use shared::{BrowserMessage, RendererMessage};
use std::io::{self, BufRead, Write};

fn main() -> Result<()> {
    eprintln!("Renderer process started");

    // Send ready message
    send_message(&RendererMessage::Ready)?;

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;

        match serde_json::from_str::<BrowserMessage>(&line) {
            Ok(BrowserMessage::RenderHtml {
                url,
                html,
                width,
                height,
            }) => {
                eprintln!("Rendering: {} ({}x{})", url, width, height);

                match render_html(&html, width, height) {
                    Ok(pixels) => {
                        let message = RendererMessage::FrameReady {
                            width,
                            height,
                            pixels,
                        };
                        send_message(&message)?;
                    }
                    Err(e) => {
                        eprintln!("Render error: {}", e);
                        let message = RendererMessage::Error {
                            message: format!("{}", e),
                        };
                        send_message(&message)?;
                    }
                }
            }
            Ok(BrowserMessage::Shutdown) => {
                eprintln!("Renderer shutting down");
                break;
            }
            Err(e) => {
                eprintln!("Failed to parse message: {}", e);
            }
        }
    }

    Ok(())
}

fn render_html(html: &str, width: u32, height: u32) -> Result<Vec<u8>> {
    // Parse HTML
    eprintln!("Parsing HTML...");
    let dom = html_parser::HtmlParser::parse(html.to_string());

    // Execute JavaScript
    eprintln!("Executing JavaScript...");
    execute_javascript(&dom)?;

    // Extract and parse CSS
    eprintln!("Parsing CSS...");
    let css = extract_css(&dom);
    let stylesheet = css_parser::CssParser::parse(css);

    // Build style tree
    eprintln!("Building style tree...");
    let styled_root = style::style_tree(&dom, &stylesheet);

    // Build layout tree
    eprintln!("Computing layout...");
    let mut viewport = layout::Dimensions::default();
    viewport.content.width = width as f32;
    viewport.content.height = height as f32;

    let layout_root = layout::layout_tree(&styled_root, viewport);

    // Render to canvas
    eprintln!("Rendering to canvas...");
    let canvas = render::render(&layout_root, width as usize, height as usize);

    eprintln!("Render complete!");
    Ok(canvas.pixels)
}

fn execute_javascript(dom: &dom::Node) -> Result<()> {
    let scripts = js_engine::extract_scripts(dom);

    if !scripts.is_empty() {
        eprintln!("Found {} script(s) to execute", scripts.len());
        let engine = js_engine::JavaScriptEngine::new()?;

        for (idx, script) in scripts.iter().enumerate() {
            eprintln!("Executing script {}...", idx + 1);
            match engine.execute(&script) {
                Ok(result) => {
                    if !result.is_empty() && result != "undefined" {
                        eprintln!("Script {} result: {}", idx + 1, result);
                    }
                }
                Err(e) => {
                    eprintln!("Script {} error: {}", idx + 1, e);
                    // Continue execution even if script fails
                }
            }
        }
    }

    Ok(())
}

fn extract_css(node: &dom::Node) -> String {
    let mut css = String::new();

    // Look for <style> elements
    if let dom::NodeType::Element(ref elem) = node.node_type {
        if elem.tag_name == "style" {
            for child in &node.children {
                if let dom::NodeType::Text(ref text) = child.node_type {
                    css.push_str(text);
                    css.push('\n');
                }
            }
        }
    }

    // Recursively extract from children
    for child in &node.children {
        css.push_str(&extract_css(child));
    }

    css
}

fn send_message(message: &RendererMessage) -> Result<()> {
    let json = serde_json::to_string(message)?;
    println!("{}", json);
    io::stdout().flush()?;
    Ok(())
}
