use anyhow::Result;
use rquickjs::{Context, Runtime, Function, Object, Value};
use crate::dom::Node;
use std::sync::{Arc, Mutex};

pub struct JavaScriptEngine {
    runtime: Runtime,
    context: Context,
    dom_root: Arc<Mutex<Option<Node>>>,
}

impl JavaScriptEngine {
    pub fn new() -> Result<Self> {
        let runtime = Runtime::new()?;
        let context = Context::full(&runtime)?;

        Ok(Self {
            runtime,
            context,
            dom_root: Arc::new(Mutex::new(None)),
        })
    }

    pub fn set_dom(&mut self, root: Node) {
        *self.dom_root.lock().unwrap() = Some(root);
    }

    pub fn execute(&self, script: &str) -> Result<String> {
        self.context.with(|ctx| {
            // Set up browser globals
            self.setup_browser_apis(&ctx)?;

            // Execute the script
            match ctx.eval::<Value, _>(script) {
                Ok(value) => {
                    if value.is_undefined() {
                        Ok("undefined".to_string())
                    } else if value.is_null() {
                        Ok("null".to_string())
                    } else {
                        Ok(value.as_string()
                            .and_then(|s| s.to_string().ok())
                            .unwrap_or_else(|| format!("{:?}", value)))
                    }
                }
                Err(e) => {
                    Err(anyhow::anyhow!("JavaScript error: {:?}", e))
                }
            }
        })
    }

    fn setup_browser_apis(&self, ctx: &rquickjs::Ctx) -> Result<()> {
        // Set up console API
        let console = Object::new(ctx.clone())?;

        let log_fn = Function::new(ctx.clone(), |args: Vec<Value>| {
            let messages: Vec<String> = args.iter()
                .filter_map(|v| {
                    v.as_string()
                        .and_then(|s| s.to_string().ok())
                        .or_else(|| Some(format!("{:?}", v)))
                })
                .collect();
            eprintln!("[JS Console] {}", messages.join(" "));
            Ok(())
        })?;

        console.set("log", log_fn)?;
        console.set("error", console.get::<_, Function>("log")?)?;
        console.set("warn", console.get::<_, Function>("log")?)?;
        console.set("info", console.get::<_, Function>("log")?)?;

        ctx.globals().set("console", console)?;

        // Set up alert (just logs for now)
        let alert_fn = Function::new(ctx.clone(), |msg: String| {
            eprintln!("[JS Alert] {}", msg);
            Ok(())
        })?;
        ctx.globals().set("alert", alert_fn)?;

        // Set up basic document object
        let document = Object::new(ctx.clone())?;

        let get_element_by_id = Function::new(ctx.clone(), |_id: String| {
            // Simplified: return null for now
            // In full implementation, this would search the DOM
            Ok(Value::new_null(rquickjs::Ctx::new(&rquickjs::Runtime::new().unwrap())))
        })?;

        document.set("getElementById", get_element_by_id)?;
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

    pub fn call_function(&self, function_name: &str, args: &[&str]) -> Result<String> {
        self.context.with(|ctx| {
            let global = ctx.globals();
            let function: Function = global.get(function_name)?;

            let js_args: Vec<Value> = args.iter()
                .map(|&arg| Value::new_string(ctx.clone(), arg).unwrap())
                .collect();

            let result: Value = function.call(js_args)?;

            Ok(result.as_string()
                .and_then(|s| s.to_string().ok())
                .unwrap_or_else(|| format!("{:?}", result)))
        })
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
}
