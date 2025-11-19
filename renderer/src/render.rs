use crate::css_parser::Color;
use crate::dom::NodeType;
use crate::layout::{BoxType, FormElementData, FormElementType, LayoutBox, Rect};
use crate::fonts::FontCache;
use crate::images::ImageData;

pub struct Canvas {
    pub pixels: Vec<u8>,
    pub width: usize,
    pub height: usize,
    font_cache: FontCache,
    focused_input: Option<usize>,
    cursor_position: usize,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Canvas {
        let white = 255;
        Canvas {
            pixels: vec![white; width * height * 4],
            width,
            height,
            font_cache: FontCache::new(),
            focused_input: None,
            cursor_position: 0,
        }
    }

    pub fn set_focus_state(&mut self, focused_input: Option<usize>, cursor_position: usize) {
        self.focused_input = focused_input;
        self.cursor_position = cursor_position;
    }

    pub fn paint_item(&mut self, layout_box: &LayoutBox) {
        self.render_background(layout_box);
        self.render_borders(layout_box);

        // Render based on box type
        match &layout_box.box_type {
            BoxType::ImageNode(_, Some(image_data)) => {
                self.render_image(layout_box, image_data);
            }
            BoxType::FormElement(_, form_data) => {
                self.render_form_element(layout_box, form_data);
            }
            _ => {
                self.render_text(layout_box);
            }
        }

        for child in &layout_box.children {
            self.paint_item(child);
        }
    }

    fn render_background(&mut self, layout_box: &LayoutBox) {
        if let Some(color) = get_background_color(layout_box) {
            let rect = layout_box.dimensions.border_box();
            self.paint_rect(rect, color);
        }
    }

    fn render_borders(&mut self, layout_box: &LayoutBox) {
        let color = match get_border_color(layout_box) {
            Some(c) => c,
            None => return,
        };

        let d = &layout_box.dimensions;
        let border_box = d.border_box();

        // Top border
        self.paint_rect(
            Rect {
                x: border_box.x,
                y: border_box.y,
                width: border_box.width,
                height: d.border.top,
            },
            color,
        );

        // Bottom border
        self.paint_rect(
            Rect {
                x: border_box.x,
                y: border_box.y + border_box.height - d.border.bottom,
                width: border_box.width,
                height: d.border.bottom,
            },
            color,
        );

        // Left border
        self.paint_rect(
            Rect {
                x: border_box.x,
                y: border_box.y,
                width: d.border.left,
                height: border_box.height,
            },
            color,
        );

        // Right border
        self.paint_rect(
            Rect {
                x: border_box.x + border_box.width - d.border.right,
                y: border_box.y,
                width: d.border.right,
                height: border_box.height,
            },
            color,
        );
    }

    fn render_image(&mut self, layout_box: &LayoutBox, image_data: &ImageData) {
        let rect = layout_box.dimensions.content;
        let dest_x = rect.x as usize;
        let dest_y = rect.y as usize;
        let dest_width = rect.width as usize;
        let dest_height = rect.height as usize;

        eprintln!("Rendering image at ({}, {}) size {}x{}", dest_x, dest_y, dest_width, dest_height);

        // Simple nearest-neighbor scaling for now
        for dy in 0..dest_height {
            for dx in 0..dest_width {
                let canvas_x = dest_x + dx;
                let canvas_y = dest_y + dy;

                if canvas_x >= self.width || canvas_y >= self.height {
                    continue;
                }

                // Map destination pixel to source pixel
                let src_x = ((dx as f32 / dest_width as f32) * image_data.width as f32) as usize;
                let src_y = ((dy as f32 / dest_height as f32) * image_data.height as f32) as usize;

                if src_x >= image_data.width as usize || src_y >= image_data.height as usize {
                    continue;
                }

                // Get source pixel (RGBA format)
                let src_offset = (src_y * image_data.width as usize + src_x) * 4;
                let src_r = image_data.pixels[src_offset];
                let src_g = image_data.pixels[src_offset + 1];
                let src_b = image_data.pixels[src_offset + 2];
                let src_a = image_data.pixels[src_offset + 3];

                // Get destination pixel offset
                let dest_offset = (canvas_y * self.width + canvas_x) * 4;

                // Alpha blending
                if src_a == 255 {
                    // Fully opaque - just copy
                    self.pixels[dest_offset] = src_r;
                    self.pixels[dest_offset + 1] = src_g;
                    self.pixels[dest_offset + 2] = src_b;
                    self.pixels[dest_offset + 3] = 255;
                } else if src_a > 0 {
                    // Blend with background
                    let alpha = src_a as f32 / 255.0;
                    let inv_alpha = 1.0 - alpha;

                    let bg_r = self.pixels[dest_offset] as f32;
                    let bg_g = self.pixels[dest_offset + 1] as f32;
                    let bg_b = self.pixels[dest_offset + 2] as f32;

                    self.pixels[dest_offset] = ((src_r as f32 * alpha) + (bg_r * inv_alpha)) as u8;
                    self.pixels[dest_offset + 1] = ((src_g as f32 * alpha) + (bg_g * inv_alpha)) as u8;
                    self.pixels[dest_offset + 2] = ((src_b as f32 * alpha) + (bg_b * inv_alpha)) as u8;
                    self.pixels[dest_offset + 3] = 255;
                }
            }
        }
    }

    fn render_form_element(&mut self, layout_box: &LayoutBox, form_data: &FormElementData) {
        let rect = layout_box.dimensions.content;

        // Check if this input is focused
        let is_focused = form_data.input_index.map_or(false, |idx| self.focused_input == Some(idx));

        // Draw form element border (blue if focused, gray otherwise)
        let border_color = if is_focused {
            Color { r: 0, g: 120, b: 215, a: 255 } // Blue focus border
        } else {
            Color { r: 128, g: 128, b: 128, a: 255 } // Gray border
        };
        let border_thickness = if is_focused { 2.0 } else { 1.0 };
        self.draw_rect_outline(rect, border_color, border_thickness);

        // Draw form element background (white for inputs, light gray for buttons)
        let bg_color = match form_data.element_type {
            FormElementType::Button | FormElementType::Submit => Color { r: 240, g: 240, b: 240, a: 255 },
            FormElementType::Checkbox | FormElementType::Radio => Color { r: 255, g: 255, b: 255, a: 255 },
            _ => Color { r: 255, g: 255, b: 255, a: 255 },
        };

        // Fill background (slightly inset from border)
        let inset = border_thickness;
        let fill_rect = Rect {
            x: rect.x + inset,
            y: rect.y + inset,
            width: rect.width - 2.0 * inset,
            height: rect.height - 2.0 * inset,
        };
        self.paint_rect(fill_rect, bg_color);

        // Render form element content
        match form_data.element_type {
            FormElementType::TextInput | FormElementType::Password |
            FormElementType::Email | FormElementType::Number => {
                self.render_input_field(rect, form_data, is_focused);
            }
            FormElementType::Button | FormElementType::Submit => {
                self.render_button(rect, form_data);
            }
            FormElementType::Textarea => {
                self.render_textarea(rect, form_data, is_focused);
            }
            FormElementType::Checkbox => {
                self.render_checkbox(rect, form_data);
            }
            FormElementType::Radio => {
                self.render_radio(rect, form_data);
            }
        }
    }

    fn render_input_field(&mut self, rect: Rect, form_data: &FormElementData, is_focused: bool) {
        let text_color = Color { r: 0, g: 0, b: 0, a: 255 };
        let placeholder_color = Color { r: 150, g: 150, b: 150, a: 255 };

        let display_text = if !form_data.value.is_empty() {
            if matches!(form_data.element_type, FormElementType::Password) {
                "*".repeat(form_data.value.len())
            } else {
                form_data.value.clone()
            }
        } else if !form_data.placeholder.is_empty() {
            form_data.placeholder.clone()
        } else {
            String::new()
        };

        let text_x = rect.x + 8.0;
        let text_y = rect.y + 6.0;

        // Render text if present
        if !display_text.is_empty() {
            let color = if form_data.value.is_empty() { placeholder_color } else { text_color };
            let text_rect = Rect {
                x: text_x,
                y: text_y,
                width: rect.width - 16.0,
                height: rect.height - 12.0,
            };
            self.render_with_fonts(&display_text, text_rect, color, 14.0);
        }

        // Draw cursor if focused
        if is_focused {
            // Calculate cursor position based on character count
            // Approximate: each char is ~8px wide
            let char_width = 8.0;
            let cursor_x = text_x + (self.cursor_position.min(form_data.value.len()) as f32 * char_width);
            let cursor_rect = Rect {
                x: cursor_x,
                y: rect.y + 4.0,
                width: 1.0,
                height: rect.height - 8.0,
            };
            self.paint_rect(cursor_rect, text_color);
        }
    }

    fn render_button(&mut self, rect: Rect, form_data: &FormElementData) {
        let text = if !form_data.value.is_empty() {
            form_data.value.clone()
        } else {
            match form_data.element_type {
                FormElementType::Submit => "Submit".to_string(),
                _ => "Button".to_string(),
            }
        };

        let text_color = Color { r: 0, g: 0, b: 0, a: 255 };
        let text_rect = Rect {
            x: rect.x + 10.0,
            y: rect.y + 8.0,
            width: rect.width - 20.0,
            height: rect.height - 16.0,
        };
        self.render_with_fonts(&text, text_rect, text_color, 14.0);
    }

    fn render_textarea(&mut self, rect: Rect, form_data: &FormElementData, is_focused: bool) {
        let text_color = Color { r: 0, g: 0, b: 0, a: 255 };
        let placeholder_color = Color { r: 150, g: 150, b: 150, a: 255 };

        let display_text = if !form_data.value.is_empty() {
            &form_data.value
        } else if !form_data.placeholder.is_empty() {
            &form_data.placeholder
        } else {
            ""
        };

        let text_x = rect.x + 8.0;
        let text_y = rect.y + 6.0;

        // Render text if present
        if !display_text.is_empty() {
            let color = if form_data.value.is_empty() { placeholder_color } else { text_color };
            let text_rect = Rect {
                x: text_x,
                y: text_y,
                width: rect.width - 16.0,
                height: rect.height - 12.0,
            };
            self.render_with_fonts(display_text, text_rect, color, 14.0);
        }

        // Draw cursor if focused (simplified - just draw at end of text for textarea)
        if is_focused {
            let char_width = 8.0;
            let cursor_x = text_x + (self.cursor_position.min(form_data.value.len()) as f32 * char_width);
            let cursor_rect = Rect {
                x: cursor_x,
                y: text_y,
                width: 1.0,
                height: 16.0,
            };
            self.paint_rect(cursor_rect, text_color);
        }
    }

    fn render_checkbox(&mut self, rect: Rect, _form_data: &FormElementData) {
        // Draw checkbox square (already has border and background from render_form_element)
        // Could add checkmark if checked, but for now just show the box
    }

    fn render_radio(&mut self, rect: Rect, _form_data: &FormElementData) {
        // Draw radio button circle
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = (rect.width.min(rect.height) / 2.0) - 2.0;

        // Draw circle border
        let border_color = Color { r: 128, g: 128, b: 128, a: 255 };
        self.draw_circle(center_x, center_y, radius, border_color);
    }

    fn draw_rect_outline(&mut self, rect: Rect, color: Color, thickness: f32) {
        // Top border
        self.paint_rect(Rect { x: rect.x, y: rect.y, width: rect.width, height: thickness }, color);
        // Bottom border
        self.paint_rect(Rect { x: rect.x, y: rect.y + rect.height - thickness, width: rect.width, height: thickness }, color);
        // Left border
        self.paint_rect(Rect { x: rect.x, y: rect.y, width: thickness, height: rect.height }, color);
        // Right border
        self.paint_rect(Rect { x: rect.x + rect.width - thickness, y: rect.y, width: thickness, height: rect.height }, color);
    }

    fn draw_circle(&mut self, center_x: f32, center_y: f32, radius: f32, color: Color) {
        let x0 = (center_x - radius).max(0.0) as usize;
        let y0 = (center_y - radius).max(0.0) as usize;
        let x1 = (center_x + radius).min(self.width as f32) as usize;
        let y1 = (center_y + radius).min(self.height as f32) as usize;

        for y in y0..y1 {
            for x in x0..x1 {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();

                // Draw if on the circle edge (±1px tolerance)
                if (dist - radius).abs() <= 1.0 {
                    self.paint_pixel(x, y, color);
                }
            }
        }
    }

    fn render_text(&mut self, layout_box: &LayoutBox) {
        let style_node = match layout_box.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) | BoxType::ImageNode(node, _) | BoxType::FormElement(node, _) => node,
            BoxType::AnonymousBlock => return,
        };

        // Get text content
        let text = get_text_content(style_node.node);
        if text.is_empty() {
            return;
        }

        let color = style_node.color();
        let font_size = style_node.font_size();
        let rect = layout_box.dimensions.content;

        // Use font rendering with fontdue
        self.render_with_fonts(&text, rect, color, font_size);
    }

    fn render_with_fonts(&mut self, text: &str, rect: Rect, color: Color, font_size: f32) {
        let text_info = self.font_cache.render_text(text.trim(), font_size, "default");

        // Fallback to simple rendering if no glyphs (no font loaded)
        if text_info.glyphs.is_empty() {
            self.render_simple_fallback(text, rect, color, font_size);
            return;
        }

        let mut x = rect.x;
        let y = rect.y;

        // Render each glyph
        for glyph in &text_info.glyphs {
            if glyph.character == '\n' {
                continue;
            }

            // Skip if width is zero (no bitmap)
            if glyph.metrics.width == 0 {
                x += glyph.metrics.advance_width;
                continue;
            }

            // Render the glyph bitmap
            let glyph_x = x + glyph.x;
            let glyph_y = y + glyph.y + glyph.metrics.ymin as f32;

            // Draw glyph pixels
            for (row_idx, row) in glyph.bitmap.chunks(glyph.metrics.width).enumerate() {
                for (col_idx, &alpha) in row.iter().enumerate() {
                    if alpha > 0 {
                        let px = (glyph_x + col_idx as f32) as usize;
                        let py = (glyph_y + row_idx as f32) as usize;

                        if px < self.width && py < self.height {
                            let offset = (py * self.width + px) * 4;
                            // Blend the glyph with the foreground color
                            let blend = alpha as f32 / 255.0;
                            self.pixels[offset] = ((color.r as f32) * blend) as u8;
                            self.pixels[offset + 1] = ((color.g as f32) * blend) as u8;
                            self.pixels[offset + 2] = ((color.b as f32) * blend) as u8;
                            self.pixels[offset + 3] = 255;
                        }
                    }
                }
            }

            x += glyph.metrics.advance_width;
            if x > rect.x + rect.width {
                break;
            }
        }
    }

    fn render_simple_fallback(&mut self, text: &str, rect: Rect, color: Color, font_size: f32) {
        // Simple fallback text rendering
        let char_width = font_size * 0.6;
        let char_height = font_size;
        let mut x = rect.x;
        let y = rect.y;

        for c in text.chars() {
            if c == ' ' || c.is_whitespace() {
                x += char_width * 0.5;
                continue;
            }

            // Draw a simple rectangle for each character
            let char_rect = Rect {
                x,
                y,
                width: char_width,
                height: char_height,
            };

            self.paint_rect(char_rect, color);
            x += char_width;

            if x > rect.x + rect.width {
                break;
            }
        }
    }

    fn paint_rect(&mut self, rect: Rect, color: Color) {
        let x0 = rect.x.max(0.0) as usize;
        let y0 = rect.y.max(0.0) as usize;
        let x1 = (rect.x + rect.width).min(self.width as f32) as usize;
        let y1 = (rect.y + rect.height).min(self.height as f32) as usize;

        for y in y0..y1 {
            for x in x0..x1 {
                self.paint_pixel(x, y, color);
            }
        }
    }

    fn paint_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let offset = (y * self.width + x) * 4;
        self.pixels[offset] = color.r;
        self.pixels[offset + 1] = color.g;
        self.pixels[offset + 2] = color.b;
        self.pixels[offset + 3] = color.a;
    }
}

fn get_background_color(layout_box: &LayoutBox) -> Option<Color> {
    match layout_box.box_type {
        BoxType::BlockNode(style_node) | BoxType::InlineNode(style_node) | BoxType::ImageNode(style_node, _) | BoxType::FormElement(style_node, _) => {
            style_node.background_color()
        }
        BoxType::AnonymousBlock => None,
    }
}

fn get_border_color(layout_box: &LayoutBox) -> Option<Color> {
    match layout_box.box_type {
        BoxType::BlockNode(style_node) | BoxType::InlineNode(style_node) | BoxType::ImageNode(style_node, _) | BoxType::FormElement(style_node, _) => {
            style_node.value("border-color").and_then(|v| match v {
                crate::css_parser::Value::Color(c) => Some(c),
                crate::css_parser::Value::Keyword(ref k) => {
                    crate::css_parser::parse_color_keyword(k)
                }
                _ => None,
            })
        }
        BoxType::AnonymousBlock => None,
    }
}

fn get_text_content(node: &crate::dom::Node) -> String {
    let mut text = String::new();

    match node.node_type {
        NodeType::Text(ref s) => {
            text.push_str(s);
        }
        NodeType::Element(_) => {
            for child in &node.children {
                text.push_str(&get_text_content(child));
            }
        }
    }

    text
}

pub fn render(
    layout_root: &LayoutBox,
    width: usize,
    height: usize,
    focused_input: Option<usize>,
    cursor_position: usize,
) -> Canvas {
    let mut canvas = Canvas::new(width, height);
    canvas.set_focus_state(focused_input, cursor_position);
    canvas.paint_item(layout_root);
    canvas
}
