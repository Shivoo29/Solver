use crate::css_parser::Color;
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct CanvasRenderingContext2D {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    fill_style: Color,
    stroke_style: Color,
    line_width: f32,
    current_path: Vec<PathCommand>,
}

#[derive(Debug, Clone)]
enum PathCommand {
    MoveTo { x: f32, y: f32 },
    LineTo { x: f32, y: f32 },
    Arc { x: f32, y: f32, radius: f32, start_angle: f32, end_angle: f32 },
}

impl CanvasRenderingContext2D {
    pub fn new(width: u32, height: u32) -> Self {
        // Initialize with transparent white background
        let pixels = vec![255; (width * height * 4) as usize];

        Self {
            width,
            height,
            pixels,
            fill_style: Color { r: 0, g: 0, b: 0, a: 255 },
            stroke_style: Color { r: 0, g: 0, b: 0, a: 255 },
            line_width: 1.0,
            current_path: Vec::new(),
        }
    }

    pub fn get_pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    // Style setters
    pub fn set_fill_style(&mut self, color: Color) {
        self.fill_style = color;
    }

    pub fn set_stroke_style(&mut self, color: Color) {
        self.stroke_style = color;
    }

    pub fn set_line_width(&mut self, width: f32) {
        self.line_width = width;
    }

    // Rectangle operations
    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let x0 = x.max(0.0) as usize;
        let y0 = y.max(0.0) as usize;
        let x1 = (x + width).min(self.width as f32) as usize;
        let y1 = (y + height).min(self.height as f32) as usize;

        for py in y0..y1 {
            for px in x0..x1 {
                self.set_pixel(px, py, self.fill_style);
            }
        }
    }

    pub fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // Draw four lines for the rectangle border
        let thickness = self.line_width as usize;

        // Top border
        self.fill_rect_internal(x, y, width, self.line_width, self.stroke_style);
        // Bottom border
        self.fill_rect_internal(x, y + height - self.line_width, width, self.line_width, self.stroke_style);
        // Left border
        self.fill_rect_internal(x, y, self.line_width, height, self.stroke_style);
        // Right border
        self.fill_rect_internal(x + width - self.line_width, y, self.line_width, height, self.stroke_style);
    }

    pub fn clear_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let clear_color = Color { r: 255, g: 255, b: 255, a: 255 };
        self.fill_rect_internal(x, y, width, height, clear_color);
    }

    fn fill_rect_internal(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        let x0 = x.max(0.0) as usize;
        let y0 = y.max(0.0) as usize;
        let x1 = (x + width).min(self.width as f32) as usize;
        let y1 = (y + height).min(self.height as f32) as usize;

        for py in y0..y1 {
            for px in x0..x1 {
                self.set_pixel(px, py, color);
            }
        }
    }

    // Path operations
    pub fn begin_path(&mut self) {
        self.current_path.clear();
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.current_path.push(PathCommand::MoveTo { x, y });
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.current_path.push(PathCommand::LineTo { x, y });
    }

    pub fn arc(&mut self, x: f32, y: f32, radius: f32, start_angle: f32, end_angle: f32) {
        self.current_path.push(PathCommand::Arc {
            x, y, radius, start_angle, end_angle
        });
    }

    pub fn fill(&mut self) {
        self.render_path(true);
    }

    pub fn stroke(&mut self) {
        self.render_path(false);
    }

    fn render_path(&mut self, fill: bool) {
        if self.current_path.is_empty() {
            return;
        }

        // Clone the path to avoid borrow checker issues
        let path = self.current_path.clone();
        let color = if fill { self.fill_style } else { self.stroke_style };

        let mut current_x = 0.0;
        let mut current_y = 0.0;

        for command in &path {
            match command {
                PathCommand::MoveTo { x, y } => {
                    current_x = *x;
                    current_y = *y;
                }
                PathCommand::LineTo { x, y } => {
                    self.draw_line(current_x, current_y, *x, *y, color);
                    current_x = *x;
                    current_y = *y;
                }
                PathCommand::Arc { x, y, radius, start_angle, end_angle } => {
                    self.draw_arc(*x, *y, *radius, *start_angle, *end_angle, color);
                }
            }
        }
    }

    fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
        // Bresenham's line algorithm
        let mut x0 = x0 as i32;
        let mut y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            if x0 >= 0 && x0 < self.width as i32 && y0 >= 0 && y0 < self.height as i32 {
                self.set_pixel(x0 as usize, y0 as usize, color);
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn draw_arc(&mut self, cx: f32, cy: f32, radius: f32, start_angle: f32, end_angle: f32, color: Color) {
        // Draw arc using line segments
        let segments = 32;
        let angle_step = (end_angle - start_angle) / segments as f32;

        let mut prev_x = cx + radius * start_angle.cos();
        let mut prev_y = cy + radius * start_angle.sin();

        for i in 1..=segments {
            let angle = start_angle + angle_step * i as f32;
            let x = cx + radius * angle.cos();
            let y = cy + radius * angle.sin();

            self.draw_line(prev_x, prev_y, x, y, color);

            prev_x = x;
            prev_y = y;
        }
    }

    // Text operations (simplified)
    pub fn fill_text(&mut self, text: &str, x: f32, y: f32) {
        // Simplified text rendering - just draw a placeholder box
        let char_width = 8.0;
        let char_height = 12.0;
        let width = text.len() as f32 * char_width;

        self.fill_rect_internal(x, y, width, char_height, self.fill_style);
    }

    pub fn stroke_text(&mut self, text: &str, x: f32, y: f32) {
        let char_width = 8.0;
        let char_height = 12.0;
        let width = text.len() as f32 * char_width;

        self.stroke_rect(x, y, width, char_height);
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width as usize || y >= self.height as usize {
            return;
        }

        let offset = (y * self.width as usize + x) * 4;
        self.pixels[offset] = color.r;
        self.pixels[offset + 1] = color.g;
        self.pixels[offset + 2] = color.b;
        self.pixels[offset + 3] = color.a;
    }
}

/// Parse color string (hex or rgb)
pub fn parse_color_string(s: &str) -> Option<Color> {
    let s = s.trim();

    // Hex color: #RRGGBB or #RGB
    if s.starts_with('#') {
        let hex = &s[1..];

        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color { r, g, b, a: 255 });
        } else if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            return Some(Color { r, g, b, a: 255 });
        }
    }

    // RGB color: rgb(r, g, b)
    if s.starts_with("rgb(") && s.ends_with(')') {
        let values = &s[4..s.len()-1];
        let parts: Vec<&str> = values.split(',').map(|p| p.trim()).collect();

        if parts.len() == 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            return Some(Color { r, g, b, a: 255 });
        }
    }

    // Named colors (basic set)
    match s {
        "red" => Some(Color { r: 255, g: 0, b: 0, a: 255 }),
        "green" => Some(Color { r: 0, g: 128, b: 0, a: 255 }),
        "blue" => Some(Color { r: 0, g: 0, b: 255, a: 255 }),
        "black" => Some(Color { r: 0, g: 0, b: 0, a: 255 }),
        "white" => Some(Color { r: 255, g: 255, b: 255, a: 255 }),
        "yellow" => Some(Color { r: 255, g: 255, b: 0, a: 255 }),
        "cyan" => Some(Color { r: 0, g: 255, b: 255, a: 255 }),
        "magenta" => Some(Color { r: 255, g: 0, b: 255, a: 255 }),
        _ => None,
    }
}
