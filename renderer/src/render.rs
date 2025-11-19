use crate::css_parser::Color;
use crate::dom::NodeType;
use crate::layout::{BoxType, LayoutBox, Rect};
use crate::fonts::FontCache;
use crate::images::ImageData;

pub struct Canvas {
    pub pixels: Vec<u8>,
    pub width: usize,
    pub height: usize,
    font_cache: FontCache,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Canvas {
        let white = 255;
        Canvas {
            pixels: vec![white; width * height * 4],
            width,
            height,
            font_cache: FontCache::new(),
        }
    }

    pub fn paint_item(&mut self, layout_box: &LayoutBox) {
        self.render_background(layout_box);
        self.render_borders(layout_box);

        // Render image if this is an ImageNode
        if let BoxType::ImageNode(_, Some(image_data)) = &layout_box.box_type {
            self.render_image(layout_box, image_data);
        } else {
            self.render_text(layout_box);
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

    fn render_text(&mut self, layout_box: &LayoutBox) {
        let style_node = match layout_box.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) | BoxType::ImageNode(node, _) => node,
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
        BoxType::BlockNode(style_node) | BoxType::InlineNode(style_node) | BoxType::ImageNode(style_node, _) => {
            style_node.background_color()
        }
        BoxType::AnonymousBlock => None,
    }
}

fn get_border_color(layout_box: &LayoutBox) -> Option<Color> {
    match layout_box.box_type {
        BoxType::BlockNode(style_node) | BoxType::InlineNode(style_node) | BoxType::ImageNode(style_node, _) => {
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

pub fn render(layout_root: &LayoutBox, width: usize, height: usize) -> Canvas {
    let mut canvas = Canvas::new(width, height);
    canvas.paint_item(layout_root);
    canvas
}
