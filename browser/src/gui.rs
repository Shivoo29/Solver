use anyhow::{Context, Result};
use gdk::prelude::*;
use gdk_pixbuf::Pixbuf;
use gtk::glib;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, DrawingArea, Entry, Orientation};
use shared::{BrowserMessage, RendererMessage};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};

const APP_ID: &str = "com.solver.Browser";
const DEFAULT_WIDTH: i32 = 1024;
const DEFAULT_HEIGHT: i32 = 768;
const CONTENT_HEIGHT: i32 = 700;

struct BrowserState {
    renderer_process: Option<Child>,
    renderer_stdin: Option<ChildStdin>,
    current_pixbuf: Option<Pixbuf>,
}

impl BrowserState {
    fn new() -> Self {
        Self {
            renderer_process: None,
            renderer_stdin: None,
            current_pixbuf: None,
        }
    }
}

pub fn run_gui() -> Result<()> {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
    Ok(())
}

fn build_ui(app: &Application) {
    let state = Arc::new(Mutex::new(BrowserState::new()));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Solver Browser")
        .default_width(DEFAULT_WIDTH)
        .default_height(DEFAULT_HEIGHT)
        .build();

    let vbox = GtkBox::new(Orientation::Vertical, 5);
    vbox.set_margin_top(5);
    vbox.set_margin_bottom(5);
    vbox.set_margin_start(5);
    vbox.set_margin_end(5);

    let hbox = GtkBox::new(Orientation::Horizontal, 5);

    let url_entry = Entry::builder()
        .placeholder_text("Enter URL...")
        .hexpand(true)
        .build();

    let go_button = Button::with_label("Go");

    hbox.append(&url_entry);
    hbox.append(&go_button);

    let drawing_area = DrawingArea::builder()
        .content_height(CONTENT_HEIGHT)
        .hexpand(true)
        .vexpand(true)
        .build();

    let state_clone = Arc::clone(&state);
    drawing_area.set_draw_func(move |_area, cr, _width, _height| {
        let state = state_clone.lock().unwrap();
        if let Some(ref pixbuf) = state.current_pixbuf {
            cr.set_source_pixbuf(pixbuf, 0.0, 0.0);
            let _ = cr.paint();
        } else {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            let _ = cr.paint();
        }
    });

    vbox.append(&hbox);
    vbox.append(&drawing_area);

    window.set_child(Some(&vbox));

    let state_clone = Arc::clone(&state);
    let url_entry_clone = url_entry.clone();
    let drawing_area_clone = drawing_area.clone();
    go_button.connect_clicked(move |_| {
        let url = url_entry_clone.text().to_string();
        if !url.is_empty() {
            let state = Arc::clone(&state_clone);
            let drawing_area = drawing_area_clone.clone();

            std::thread::spawn(move || {
                if let Err(e) = handle_navigation(url, state, drawing_area) {
                    eprintln!("Navigation error: {}", e);
                }
            });
        }
    });

    let go_button_clone = go_button.clone();
    url_entry.connect_activate(move |_| {
        go_button_clone.emit_clicked();
    });

    let state_clone = Arc::clone(&state);
    window.connect_close_request(move |_| {
        shutdown_renderer(&state_clone);
        glib::Propagation::Proceed
    });

    window.present();
}

fn handle_navigation(url: String, state: Arc<Mutex<BrowserState>>, drawing_area: DrawingArea) -> Result<()> {
    println!("Navigating to: {}", url);

    let html = super::fetch_url(&url)?;

    ensure_renderer_running(&state)?;

    let message = BrowserMessage::RenderHtml {
        url: url.clone(),
        html,
        width: 1000,
        height: CONTENT_HEIGHT as u32,
    };

    send_to_renderer(&state, &message)?;

    let response = read_from_renderer(&state)?;

    match response {
        RendererMessage::FrameReady { width, height, pixels } => {
            println!("Received rendered frame: {}x{}", width, height);

            let pixbuf = Pixbuf::from_bytes(
                &glib::Bytes::from(&pixels),
                gdk_pixbuf::Colorspace::Rgb,
                true,
                8,
                width as i32,
                height as i32,
                (width * 4) as i32,
            );

            {
                let mut state = state.lock().unwrap();
                state.current_pixbuf = Some(pixbuf);
            }

            drawing_area.queue_draw();
        }
        RendererMessage::Error { message } => {
            eprintln!("Renderer error: {}", message);
        }
        _ => {}
    }

    Ok(())
}

fn ensure_renderer_running(state: &Arc<Mutex<BrowserState>>) -> Result<()> {
    let mut state = state.lock().unwrap();

    if state.renderer_process.is_none() {
        println!("Starting renderer process...");

        let renderer_path = super::find_renderer_binary()?;

        let mut child = Command::new(renderer_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn renderer process")?;

        let stdin = child.stdin.take().context("Failed to get renderer stdin")?;

        state.renderer_stdin = Some(stdin);
        state.renderer_process = Some(child);

        println!("Renderer process started");

        drop(state);
        let response = read_from_renderer_unlocked(state)?;
        if !matches!(response, RendererMessage::Ready) {
            anyhow::bail!("Expected Ready message from renderer");
        }
        println!("Renderer ready");
    }

    Ok(())
}

fn send_to_renderer(state: &Arc<Mutex<BrowserState>>, message: &BrowserMessage) -> Result<()> {
    let mut state = state.lock().unwrap();

    if let Some(ref mut stdin) = state.renderer_stdin {
        let json = serde_json::to_string(message)?;
        writeln!(stdin, "{}", json)?;
        stdin.flush()?;
    } else {
        anyhow::bail!("Renderer not running");
    }

    Ok(())
}

fn read_from_renderer(state: &Arc<Mutex<BrowserState>>) -> Result<RendererMessage> {
    read_from_renderer_unlocked(state)
}

fn read_from_renderer_unlocked(state: &Arc<Mutex<BrowserState>>) -> Result<RendererMessage> {
    let mut state = state.lock().unwrap();

    if let Some(ref mut process) = state.renderer_process {
        if let Some(ref mut stdout) = process.stdout {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            reader.read_line(&mut line)?;

            let response: RendererMessage = serde_json::from_str(&line)?;
            Ok(response)
        } else {
            anyhow::bail!("No stdout from renderer");
        }
    } else {
        anyhow::bail!("Renderer not running");
    }
}

fn shutdown_renderer(state: &Arc<Mutex<BrowserState>>) {
    let mut state = state.lock().unwrap();

    if state.renderer_process.is_some() {
        let _ = send_to_renderer_internal(&mut state, &BrowserMessage::Shutdown);

        if let Some(mut child) = state.renderer_process.take() {
            let _ = child.wait();
        }
    }
}

fn send_to_renderer_internal(state: &mut BrowserState, message: &BrowserMessage) -> Result<()> {
    if let Some(ref mut stdin) = state.renderer_stdin {
        let json = serde_json::to_string(message)?;
        writeln!(stdin, "{}", json)?;
        stdin.flush()?;
    }
    Ok(())
}
