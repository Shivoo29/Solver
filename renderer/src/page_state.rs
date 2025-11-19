use crate::layout::{FormElementType, LayoutBox, BoxType};
use anyhow::Result;
use std::collections::HashMap;

/// Represents the current state of a rendered page (UI state only)
pub struct PageState {
    pub url: String,
    pub html: String,
    pub width: u32,
    pub height: u32,
    pub focused_input: Option<usize>, // Index of focused input field
    pub form_values: HashMap<usize, String>, // Values for form inputs
    pub cursor_position: usize, // Cursor position in focused input
}

impl PageState {
    /// Create a new page state from HTML
    pub fn new(url: String, html: String, width: u32, height: u32) -> Self {
        eprintln!("Creating new page state for: {}", url);

        Self {
            url,
            html,
            width,
            height,
            focused_input: None,
            form_values: HashMap::new(),
            cursor_position: 0,
        }
    }

    /// Update page dimensions
    pub fn update_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Handle a mouse click event (requires layout tree to find which element was clicked)
    pub fn handle_click(&mut self, layout_root: &LayoutBox, x: f32, y: f32) -> bool {
        eprintln!("Click at ({}, {})", x, y);

        // Find which layout box was clicked
        if let Some(input_index) = Self::find_input_at_position(layout_root, x, y, 0).0 {
            eprintln!("Clicked on input field {}", input_index);
            self.focused_input = Some(input_index);

            // Set cursor to end of text
            if let Some(value) = self.form_values.get(&input_index) {
                self.cursor_position = value.len();
            } else {
                self.cursor_position = 0;
            }

            return true; // State changed, need re-render
        } else {
            // Clicked outside any input - remove focus
            if self.focused_input.is_some() {
                self.focused_input = None;
                return true; // State changed
            }
        }

        false // No state change
    }

    /// Handle a key press event
    pub fn handle_key(&mut self, key: &shared::KeyEvent) -> bool {
        use shared::KeyEvent;

        if let Some(input_index) = self.focused_input {
            let value = self.form_values.entry(input_index).or_insert_with(String::new);

            match key {
                KeyEvent::Char(c) => {
                    // Insert character at cursor position
                    value.insert(self.cursor_position, *c);
                    self.cursor_position += 1;
                    eprintln!("Input {}: '{}'", input_index, value);
                    return true;
                }
                KeyEvent::Backspace => {
                    if self.cursor_position > 0 {
                        value.remove(self.cursor_position - 1);
                        self.cursor_position -= 1;
                        eprintln!("Input {}: '{}'", input_index, value);
                        return true;
                    }
                }
                KeyEvent::Delete => {
                    if self.cursor_position < value.len() {
                        value.remove(self.cursor_position);
                        eprintln!("Input {}: '{}'", input_index, value);
                        return true;
                    }
                }
                KeyEvent::ArrowLeft => {
                    if self.cursor_position > 0 {
                        self.cursor_position -= 1;
                        return true;
                    }
                }
                KeyEvent::ArrowRight => {
                    if self.cursor_position < value.len() {
                        self.cursor_position += 1;
                        return true;
                    }
                }
                KeyEvent::Enter => {
                    // Could trigger form submission
                    eprintln!("Enter pressed in input {}", input_index);
                }
                _ => {}
            }
        }

        false
    }

    /// Find input field at given position, returns (Option<input_index>, next_count)
    fn find_input_at_position(layout_box: &LayoutBox, x: f32, y: f32, mut input_count: usize) -> (Option<usize>, usize) {
        // Check if this is a form element
        if let BoxType::FormElement(_, form_data) = &layout_box.box_type {
            // Check if it's an input type that can receive focus
            if matches!(form_data.element_type,
                FormElementType::TextInput | FormElementType::Password |
                FormElementType::Email | FormElementType::Number | FormElementType::Textarea) {

                let rect = layout_box.dimensions.content;
                if x >= rect.x && x <= rect.x + rect.width &&
                   y >= rect.y && y <= rect.y + rect.height {
                    return (Some(input_count), input_count + 1);
                }
                input_count += 1;
            }
        }

        // Check children
        for child in &layout_box.children {
            let (found, new_count) = Self::find_input_at_position(child, x, y, input_count);
            if found.is_some() {
                return (found, new_count);
            }
            input_count = new_count;
        }

        (None, input_count)
    }

    /// Get the current value for a form input
    pub fn get_form_value(&self, index: usize) -> Option<&str> {
        self.form_values.get(&index).map(|s| s.as_str())
    }

    /// Check if an input is focused
    pub fn is_input_focused(&self, index: usize) -> bool {
        self.focused_input == Some(index)
    }

    /// Get cursor position (only valid if input is focused)
    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }
}
