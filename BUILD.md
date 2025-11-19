# Solver Browser - Build Instructions

This document provides step-by-step instructions to build and run the Solver web browser on Ubuntu Linux.

## Prerequisites

Solver Browser requires:
- Rust (stable toolchain)
- GTK4 development libraries
- C compiler and build tools

## Installation Steps

### 1. Install System Dependencies

```bash
# Update package list
sudo apt-get update

# Install build essentials
sudo apt-get install -y build-essential pkg-config

# Install GTK4 and its dependencies
sudo apt-get install -y libgtk-4-dev libglib2.0-dev libcairo2-dev libpango1.0-dev libgdk-pixbuf-2.0-dev

# Install additional required libraries
sudo apt-get install -y libssl-dev
```

### 2. Install Rust

If you don't have Rust installed, install it using rustup:

```bash
# Download and install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts and select the default installation

# Add Rust to your PATH (or restart your terminal)
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 3. Build Solver Browser

Navigate to the Solver directory and build the project.

**Option A: Build with GUI support (GTK4)**

If you have GTK4 installed and want the graphical interface:

```bash
cd /path/to/Solver

# Build with GUI feature enabled
cargo build --release --features gui
```

**Option B: Build headless mode (no GUI required)**

For headless operation or systems without GTK4:

```bash
cd /path/to/Solver

# Build without GUI (default)
cargo build --release
```

The build process will compile three components:
- `shared`: Common IPC message types
- `renderer`: Rust-based rendering engine (sandboxed process)
- `browser`: Main browser process (with or without GUI)

### 4. Run Solver Browser

**GUI Mode** (requires GTK4 and `--features gui` build):

```bash
./target/release/solver
```

When Solver launches in GUI mode, you'll see a window with an address bar at the top. Type a URL and press Enter to browse.

**Headless Mode** (default build):

```bash
# Render to a PNG file
./target/release/solver <url> <output.png>

# Examples:
./target/release/solver test output.png
./target/release/solver http://example.com example.png
```

In headless mode, the browser:
- Fetches the specified URL
- Renders it through the complete pipeline
- Saves the output as a PNG file

## Usage

**GUI Mode:**
1. Launch the browser: `./target/release/solver`
2. Type a URL in the address bar (e.g., `test` or `http://example.com`)
3. Press Enter or click "Go"
4. View the rendered page in the content area

**Headless Mode:**
1. Run with URL and output file: `./target/release/solver test output.png`
2. The browser will render and save to PNG
3. Open the PNG file to view the rendered page

**Both modes:**
- Fetch HTML content from the network (or use built-in test page)
- Parse HTML and CSS
- Build the DOM tree
- Compute styles and layout
- Render the page in the Rust-based rendering engine
- Display or save the result

## Architecture Overview

Solver Browser implements a multi-process architecture:

### Browser Process (`browser/`)
- GTK4-based UI with address bar and content display
- Network service for fetching URLs
- Process manager that spawns and manages the renderer
- IPC coordinator for browser-renderer communication

### Renderer Process (`renderer/`)
- **Sandboxed** process (runs as separate OS process)
- Pure Rust implementation for memory safety
- Components:
  - `html_parser`: Parses HTML into DOM tree
  - `css_parser`: Parses CSS stylesheets
  - `dom`: DOM tree structures
  - `style`: Applies CSS rules to DOM nodes
  - `layout`: Computes box layout (positions and sizes)
  - `render`: Renders layout tree to RGBA bitmap

### IPC (`shared/`)
- JSON-based message protocol over stdin/stdout
- Messages for render requests and frame responses

## Security Features

- **Memory Safety**: Renderer written in Rust prevents memory vulnerabilities
- **Process Isolation**: Renderer runs as a separate sandboxed process
- **No Telemetry**: Zero tracking or data collection
- **Privacy-First**: No analytics, no user tracking, no phone-home

## Supported HTML/CSS Features

This minimal browser prototype supports:

### HTML Elements
- `<html>`, `<head>`, `<body>`
- `<h1>` through `<h6>` headers
- `<p>` paragraphs
- `<div>` containers
- `<style>` for embedded CSS

### CSS Properties
- `color`: Text color
- `background-color`: Background color
- `font-size`: Text size
- `display`: block, inline, none
- `margin`, `margin-top`, `margin-bottom`, `margin-left`, `margin-right`
- `padding`, `padding-top`, `padding-bottom`, `padding-left`, `padding-right`
- `border-width`, `border-color`
- `width`, `height`

### CSS Selectors
- Tag selectors (e.g., `h1`, `p`, `div`)
- Class selectors (e.g., `.highlight`)
- ID selectors (e.g., `#header`)

### CSS Values
- Color names (black, white, red, green, blue, yellow, etc.)
- Hex colors (e.g., `#FF5733`)
- Pixel lengths (e.g., `16px`)

## Troubleshooting

### Build Errors

**Issue**: `pkg-config` not found
```bash
sudo apt-get install pkg-config
```

**Issue**: GTK4 libraries not found
```bash
sudo apt-get install libgtk-4-dev
```

**Issue**: OpenSSL errors
```bash
sudo apt-get install libssl-dev
```

### Runtime Errors

**Issue**: "Renderer binary not found"
- Make sure you've run `cargo build` before launching the browser
- The browser looks for the renderer in `target/debug/` or `target/release/`

**Issue**: GTK warnings or errors
- Ensure you're running in a graphical environment (not SSH without X forwarding)
- Install GTK4 runtime: `sudo apt-get install gtk4`

**Issue**: Network request fails
- Check your internet connection
- Some sites may block requests from non-standard user agents
- Try the built-in test page by entering `test` in the address bar

## Development

To modify the browser:

1. **Browser UI/Network**: Edit files in `browser/src/`
2. **Rendering Engine**: Edit files in `renderer/src/`
3. **IPC Protocol**: Edit `shared/src/lib.rs`

After making changes, rebuild:
```bash
cargo build --release
```

## Clean Build

To remove build artifacts and start fresh:

```bash
cargo clean
```

## System Requirements

- **OS**: Ubuntu 20.04 or newer (other Linux distributions may work)
- **RAM**: Minimum 512MB, 2GB+ recommended
- **Disk**: ~500MB for dependencies and build artifacts
- **Display**: X11 or Wayland with GTK4 support

## License

Solver Browser - A minimal, secure, privacy-focused web browser built with Rust.

---

Built with ❤️ and Rust for Security, Privacy, and Performance.
