use anyhow::Result;
use rquickjs::{Context, Runtime, Function, Object, CatchResultExt};
use crate::dom::Node;
use crate::canvas::{CanvasRenderingContext2D, parse_color_string};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct JavaScriptEngine {
    runtime: Runtime,
    context: Context,
    dom_root: Arc<Mutex<Option<Node>>>,
    canvases: Arc<Mutex<HashMap<String, CanvasRenderingContext2D>>>,
}

impl JavaScriptEngine {
    pub fn new() -> Result<Self> {
        let runtime = Runtime::new()?;
        let context = Context::full(&runtime)?;

        Ok(Self {
            runtime,
            context,
            dom_root: Arc::new(Mutex::new(None)),
            canvases: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn create_canvas(&self, id: &str, width: u32, height: u32) {
        let canvas = CanvasRenderingContext2D::new(width, height);
        self.canvases.lock().unwrap().insert(id.to_string(), canvas);
    }

    pub fn get_canvas(&self, id: &str) -> Option<CanvasRenderingContext2D> {
        self.canvases.lock().unwrap().get(id).cloned()
    }

    pub fn set_dom(&mut self, root: Node) {
        *self.dom_root.lock().unwrap() = Some(root);
    }

    pub fn execute(&self, script: &str) -> Result<String> {
        self.context.with(|ctx| {
            // Set up browser globals
            self.setup_browser_apis(&ctx)?;

            // Execute the script and get a Value
            let result: rquickjs::Value = ctx.eval(script).catch(&ctx)
                .map_err(|e| anyhow::anyhow!("JavaScript error: {:?}", e))?;

            // Convert result to string based on type
            let result_str = if result.is_undefined() {
                "undefined".to_string()
            } else if result.is_null() {
                "null".to_string()
            } else if let Some(b) = result.as_bool() {
                b.to_string()
            } else if let Some(n) = result.as_int() {
                n.to_string()
            } else if let Some(n) = result.as_float() {
                n.to_string()
            } else if let Some(s) = result.as_string() {
                s.to_string().unwrap_or_else(|_| String::from("[string]"))
            } else {
                format!("{:?}", result)
            };

            Ok(result_str)
        })
    }

    fn setup_browser_apis(&self, ctx: &rquickjs::Ctx) -> Result<()> {
        // Set up console API
        let console = Object::new(ctx.clone())?;

        // Create log function
        let log_fn = Function::new(ctx.clone(), |msg: String| {
            eprintln!("[JS Console] {}", msg);
            Ok::<(), rquickjs::Error>(())
        })?;

        console.set("log", log_fn.clone())?;
        console.set("error", log_fn.clone())?;
        console.set("warn", log_fn.clone())?;
        console.set("info", log_fn)?;

        ctx.globals().set("console", console)?;

        // Set up alert
        let alert_fn = Function::new(ctx.clone(), |msg: String| {
            eprintln!("[JS Alert] {}", msg);
            Ok::<(), rquickjs::Error>(())
        })?;
        ctx.globals().set("alert", alert_fn)?;

        // Set up basic document object
        let document = Object::new(ctx.clone())?;
        document.set("title", "Solver Browser")?;
        ctx.globals().set("document", document)?;

        // Set up window object
        let window = Object::new(ctx.clone())?;
        window.set("console", ctx.globals().get::<_, Object>("console")?)?;
        window.set("document", ctx.globals().get::<_, Object>("document")?)?;
        ctx.globals().set("window", window)?;

        // Set up navigator
        let navigator = Object::new(ctx.clone())?;
        navigator.set("userAgent", "Solver/1.0 (Rust; rv:1.0)")?;
        navigator.set("language", "en-US")?;
        navigator.set("onLine", true)?;
        ctx.globals().set("navigator", navigator)?;

        // Set up location
        let location = Object::new(ctx.clone())?;
        location.set("href", "about:blank")?;
        location.set("protocol", "http:")?;
        location.set("host", "")?;
        location.set("pathname", "/")?;
        ctx.globals().set("location", location)?;

        Ok(())
    }
}

/// Extract JavaScript from <script> tags in HTML
pub fn extract_scripts(node: &Node) -> Vec<String> {
    let mut scripts = Vec::new();

    match &node.node_type {
        crate::dom::NodeType::Element(elem) => {
            if elem.tag_name == "script" {
                // Get text content of script tag
                for child in &node.children {
                    if let crate::dom::NodeType::Text(text) = &child.node_type {
                        scripts.push(text.clone());
                    }
                }
            }
        }
        _ => {}
    }

    // Recursively extract from children
    for child in &node.children {
        scripts.extend(extract_scripts(child));
    }

    scripts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_execution() {
        let engine = JavaScriptEngine::new().unwrap();
        let result = engine.execute("1 + 1").unwrap();
        assert_eq!(result, "2");
    }

    #[test]
    fn test_console_log() {
        let engine = JavaScriptEngine::new().unwrap();
        // This should print to stderr
        let _ = engine.execute("console.log('Hello from JavaScript!')");
    }

    #[test]
    fn test_variables() {
        let engine = JavaScriptEngine::new().unwrap();
        let result = engine.execute("var x = 10; var y = 20; x + y").unwrap();
        assert_eq!(result, "30");
    }
}
