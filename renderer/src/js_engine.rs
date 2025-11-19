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
            self.setup_browser_apis(ctx.clone())?;

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

    fn setup_browser_apis<'js>(&self, ctx: rquickjs::Ctx<'js>) -> Result<()> {
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

        // Set up document object with DOM access
        let document = Object::new(ctx.clone())?;
        document.set("title", "Solver Browser")?;

        // Clone dom_root for use in closures
        let dom_root_clone = Arc::clone(&self.dom_root);

        // document.getElementById(id) - simplified stub
        let get_element_by_id = Function::new(ctx.clone(), move |id: String| {
            let dom = dom_root_clone.lock().unwrap();
            if let Some(ref root) = *dom {
                if let Some(_element) = find_element_by_id(root, &id) {
                    eprintln!("[JS] getElementById('{}') found element", id);
                    // Return a simple object indicating found
                    return Ok::<Option<String>, rquickjs::Error>(Some(format!("[Element: id='{}']", id)));
                }
            }
            eprintln!("[JS] getElementById('{}') element not found", id);
            Ok::<Option<String>, rquickjs::Error>(None)
        })?;
        document.set("getElementById", get_element_by_id)?;

        // Clone dom_root for querySelector
        let dom_root_clone2 = Arc::clone(&self.dom_root);

        // document.querySelector(selector) - simplified stub
        let query_selector = Function::new(ctx.clone(), move |selector: String| {
            let dom = dom_root_clone2.lock().unwrap();
            if let Some(ref root) = *dom {
                // Simple selector parsing: #id, .class, tagname
                if selector.starts_with('#') {
                    let id = &selector[1..];
                    if let Some(_element) = find_element_by_id(root, id) {
                        eprintln!("[JS] querySelector('{}') found element", selector);
                        return Ok::<Option<String>, rquickjs::Error>(Some(format!("[Element: {}]", selector)));
                    }
                } else if selector.starts_with('.') {
                    let class = &selector[1..];
                    if let Some(_element) = find_element_by_class(root, class) {
                        eprintln!("[JS] querySelector('{}') found element", selector);
                        return Ok::<Option<String>, rquickjs::Error>(Some(format!("[Element: {}]", selector)));
                    }
                } else {
                    if let Some(_element) = find_element_by_tag(root, &selector) {
                        eprintln!("[JS] querySelector('{}') found element", selector);
                        return Ok::<Option<String>, rquickjs::Error>(Some(format!("[Element: {}]", selector)));
                    }
                }
            }
            eprintln!("[JS] querySelector('{}') element not found", selector);
            Ok::<Option<String>, rquickjs::Error>(None)
        })?;
        document.set("querySelector", query_selector)?;

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

        // Set up setTimeout and setInterval (simplified - logs warning)
        let set_timeout = Function::new(ctx.clone(), |callback: String, delay: u32| {
            eprintln!("[JS] setTimeout called with delay {}ms (not fully implemented)", delay);
            eprintln!("[JS] Callback: {}", callback);
            Ok::<i32, rquickjs::Error>(0) // Return timer ID
        })?;
        ctx.globals().set("setTimeout", set_timeout)?;

        let set_interval = Function::new(ctx.clone(), |callback: String, delay: u32| {
            eprintln!("[JS] setInterval called with delay {}ms (not fully implemented)", delay);
            eprintln!("[JS] Callback: {}", callback);
            Ok::<i32, rquickjs::Error>(0) // Return timer ID
        })?;
        ctx.globals().set("setInterval", set_interval)?;

        let clear_timeout = Function::new(ctx.clone(), |_id: i32| {
            Ok::<(), rquickjs::Error>(())
        })?;
        ctx.globals().set("clearTimeout", clear_timeout)?;

        let clear_interval = Function::new(ctx.clone(), |_id: i32| {
            Ok::<(), rquickjs::Error>(())
        })?;
        ctx.globals().set("clearInterval", clear_interval)?;

        // Set up localStorage (in-memory implementation)
        let storage = Object::new(ctx.clone())?;
        let storage_data = Arc::new(Mutex::new(HashMap::<String, String>::new()));

        let storage_data_get = Arc::clone(&storage_data);
        let get_item = Function::new(ctx.clone(), move |key: String| {
            let data = storage_data_get.lock().unwrap();
            Ok::<Option<String>, rquickjs::Error>(data.get(&key).cloned())
        })?;
        storage.set("getItem", get_item)?;

        let storage_data_set = Arc::clone(&storage_data);
        let set_item = Function::new(ctx.clone(), move |key: String, value: String| {
            let mut data = storage_data_set.lock().unwrap();
            data.insert(key, value);
            Ok::<(), rquickjs::Error>(())
        })?;
        storage.set("setItem", set_item)?;

        let storage_data_remove = Arc::clone(&storage_data);
        let remove_item = Function::new(ctx.clone(), move |key: String| {
            let mut data = storage_data_remove.lock().unwrap();
            data.remove(&key);
            Ok::<(), rquickjs::Error>(())
        })?;
        storage.set("removeItem", remove_item)?;

        let storage_data_clear = Arc::clone(&storage_data);
        let clear = Function::new(ctx.clone(), move || {
            let mut data = storage_data_clear.lock().unwrap();
            data.clear();
            Ok::<(), rquickjs::Error>(())
        })?;
        storage.set("clear", clear)?;

        ctx.globals().set("localStorage", storage.clone())?;
        ctx.globals().set("sessionStorage", storage)?; // For now, both use same storage

        // Set up fetch API (stub implementation)
        let fetch = Function::new(ctx.clone(), |url: String| {
            eprintln!("[JS] fetch('{}') called (not yet fully implemented)", url);
            // Return a simple string for now
            Ok::<String, rquickjs::Error>(format!("[Promise: fetch('{}')]", url))
        })?;
        ctx.globals().set("fetch", fetch)?;

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

// Helper functions for DOM traversal

/// Find an element by ID in the DOM tree
fn find_element_by_id<'a>(node: &'a Node, id: &str) -> Option<&'a Node> {
    if let Some(node_id) = node.get_id() {
        if node_id == id {
            return Some(node);
        }
    }

    for child in &node.children {
        if let Some(found) = find_element_by_id(child, id) {
            return Some(found);
        }
    }

    None
}

/// Find an element by class name in the DOM tree
fn find_element_by_class<'a>(node: &'a Node, class: &str) -> Option<&'a Node> {
    let classes = node.get_classes();
    if classes.contains(&class) {
        return Some(node);
    }

    for child in &node.children {
        if let Some(found) = find_element_by_class(child, class) {
            return Some(found);
        }
    }

    None
}

/// Find an element by tag name in the DOM tree
fn find_element_by_tag<'a>(node: &'a Node, tag: &str) -> Option<&'a Node> {
    if let crate::dom::NodeType::Element(elem) = &node.node_type {
        if elem.tag_name.eq_ignore_ascii_case(tag) {
            return Some(node);
        }
    }

    for child in &node.children {
        if let Some(found) = find_element_by_tag(child, tag) {
            return Some(found);
        }
    }

    None
}

/// Get text content of a node (recursively)
fn get_text_content(node: &Node) -> String {
    match &node.node_type {
        crate::dom::NodeType::Text(text) => text.clone(),
        crate::dom::NodeType::Element(_) => {
            node.children
                .iter()
                .map(|child| get_text_content(child))
                .collect::<Vec<_>>()
                .join("")
        }
    }
}

/// Create a JavaScript object representing a DOM element
fn create_element_object<'js>(ctx: rquickjs::Ctx<'js>, node: &Node) -> Result<Object<'js>> {
    let element = Object::new(ctx.clone())?;

    if let crate::dom::NodeType::Element(elem) = &node.node_type {
        // Set basic properties
        element.set("tagName", elem.tag_name.to_uppercase())?;
        element.set("nodeName", elem.tag_name.to_uppercase())?;

        // Set id attribute if present
        if let Some(id) = node.get_id() {
            element.set("id", id)?;
        } else {
            element.set("id", "")?;
        }

        // Set className
        if let Some(class_attr) = node.get_attribute("class") {
            element.set("className", class_attr)?;
        } else {
            element.set("className", "")?;
        }

        // Set textContent
        element.set("textContent", get_text_content(node))?;

        // Set innerHTML (simplified - just the text content for now)
        element.set("innerHTML", get_text_content(node))?;

        // Create getAttribute function
        let attrs = elem.attributes.clone();
        let get_attr = Function::new(ctx.clone(), move |name: String| {
            Ok::<Option<String>, rquickjs::Error>(attrs.get(&name).cloned())
        })?;
        element.set("getAttribute", get_attr)?;

        // Create setAttribute function (currently non-functional - would need DOM mutation)
        let set_attr = Function::new(ctx.clone(), |name: String, value: String| {
            eprintln!("[JS] setAttribute('{}', '{}') called (not yet implemented)", name, value);
            Ok::<(), rquickjs::Error>(())
        })?;
        element.set("setAttribute", set_attr)?;

        // Create addEventListener function
        let add_event_listener = Function::new(ctx.clone(), |event_type: String, _callback: rquickjs::Function| {
            eprintln!("[JS] addEventListener('{}') called (not yet fully implemented)", event_type);
            Ok::<(), rquickjs::Error>(())
        })?;
        element.set("addEventListener", add_event_listener)?;

        // Create removeEventListener function
        let remove_event_listener = Function::new(ctx.clone(), |event_type: String, _callback: rquickjs::Function| {
            eprintln!("[JS] removeEventListener('{}') called (not yet implemented)", event_type);
            Ok::<(), rquickjs::Error>(())
        })?;
        element.set("removeEventListener", remove_event_listener)?;

        // Create click() method
        let click = Function::new(ctx.clone(), || {
            eprintln!("[JS] element.click() called (not yet implemented)");
            Ok::<(), rquickjs::Error>(())
        })?;
        element.set("click", click)?;

        // Create style object (simplified)
        let style = Object::new(ctx.clone())?;
        element.set("style", style)?;
    }

    Ok(element)
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
