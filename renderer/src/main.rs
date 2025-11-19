mod css_parser;
mod dom;
mod html_parser;
mod layout;
mod render;
mod style;
mod js_engine;
mod images;
mod fonts;
mod page_state;
mod canvas;

use anyhow::Result;
use shared::{BrowserMessage, RendererMessage};
use std::io::{self, BufRead, Write};

fn main() -> Result<()> {
    eprintln!("Renderer process started");

    // Send ready message
    send_message(&RendererMessage::Ready)?;

    let stdin = io::stdin();
    let mut _stdout = io::stdout();

    // Track current page state
    let mut current_page: Option<page_state::PageState> = None;

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

                // Create new page state
                current_page = Some(page_state::PageState::new(url.clone(), html.clone(), width, height));

                match render_page_state(&current_page.as_ref().unwrap()) {
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
            Ok(BrowserMessage::MouseClick { x, y }) => {
                eprintln!("Mouse click: ({}, {})", x, y);

                if let Some(ref mut page) = current_page {
                    // Need to re-render to get layout tree for hit testing
                    match render_and_handle_click(page, x, y) {
                        Ok((changed, pixels)) => {
                            if changed {
                                let message = RendererMessage::FrameReady {
                                    width: page.width,
                                    height: page.height,
                                    pixels,
                                };
                                send_message(&message)?;
                            }
                        }
                        Err(e) => {
                            eprintln!("Click handling error: {}", e);
                        }
                    }
                }
            }
            Ok(BrowserMessage::KeyPress { key }) => {
                eprintln!("Key press: {:?}", key);

                if let Some(ref mut page) = current_page {
                    let changed = page.handle_key(&key);
                    if changed {
                        match render_page_state(page) {
                            Ok(pixels) => {
                                let message = RendererMessage::FrameReady {
                                    width: page.width,
                                    height: page.height,
                                    pixels,
                                };
                                send_message(&message)?;
                            }
                            Err(e) => {
                                eprintln!("Re-render error: {}", e);
                            }
                        }
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

fn render_page_state(page: &page_state::PageState) -> Result<Vec<u8>> {
    // Parse HTML
    eprintln!("Parsing HTML...");
    let dom = html_parser::HtmlParser::parse(page.html.clone());

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

    // Build layout tree (with form values from page state)
    eprintln!("Computing layout...");
    let mut viewport = layout::Dimensions::default();
    viewport.content.width = page.width as f32;
    viewport.content.height = page.height as f32;

    // Create image cache for loading images
    let image_cache = images::ImageCache::new();
    let layout_root = layout::layout_tree(
        &styled_root,
        viewport,
        &image_cache,
        Some(&page.form_values),
    );

    // Render to canvas (with focus state)
    eprintln!("Rendering to canvas...");
    let canvas = render::render(
        &layout_root,
        page.width as usize,
        page.height as usize,
        page.focused_input,
        page.cursor_position,
    );

    eprintln!("Render complete!");
    Ok(canvas.pixels)
}

fn render_and_handle_click(page: &mut page_state::PageState, x: f32, y: f32) -> Result<(bool, Vec<u8>)> {
    // Parse HTML and build layout to get hit test
    let dom = html_parser::HtmlParser::parse(page.html.clone());
    let css = extract_css(&dom);
    let stylesheet = css_parser::CssParser::parse(css);
    let styled_root = style::style_tree(&dom, &stylesheet);

    let mut viewport = layout::Dimensions::default();
    viewport.content.width = page.width as f32;
    viewport.content.height = page.height as f32;

    let image_cache = images::ImageCache::new();
    let layout_root = layout::layout_tree(
        &styled_root,
        viewport,
        &image_cache,
        Some(&page.form_values),
    );

    // Handle the click
    let changed = page.handle_click(&layout_root, x, y);

    // Re-render with updated state
    let canvas = render::render(
        &layout_root,
        page.width as usize,
        page.height as usize,
        page.focused_input,
        page.cursor_position,
    );

    Ok((changed, canvas.pixels))
}

fn execute_javascript(dom: &dom::Node) -> Result<()> {
    let scripts = js_engine::extract_scripts(dom);

    if !scripts.is_empty() {
        eprintln!("Found {} script(s) to execute", scripts.len());
        let mut engine = js_engine::JavaScriptEngine::new()?;

        // Set the DOM in the JavaScript engine so it can be accessed
        engine.set_dom(dom.clone());

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
