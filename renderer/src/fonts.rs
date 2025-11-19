use fontdue::{Font, FontSettings};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct FontCache {
    fonts: Arc<Mutex<HashMap<String, Font>>>,
    default_font: Option<Font>,
}

impl FontCache {
    pub fn new() -> Self {
        // Try to load a system font
        let default_font = Self::load_system_font();

        Self {
            fonts: Arc::new(Mutex::new(HashMap::new())),
            default_font,
        }
    }

    fn load_system_font() -> Option<Font> {
        // Try common font locations
        let font_paths = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
            "C:\\Windows\\Fonts\\Arial.ttf",
        ];

        for path in &font_paths {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(font) = Font::from_bytes(data as &[u8], FontSettings::default()) {
                    return Some(font);
                }
            }
        }

        None
    }

    pub fn get_font(&self, _family: &str) -> Option<&Font> {
        // For now, just return default font
        self.default_font.as_ref()
    }

    pub fn render_text(&self, text: &str, font_size: f32, font_family: &str) -> TextRenderInfo {
        let mut glyphs = Vec::new();
        let mut cursor_x = 0.0;
        let mut max_height = font_size;

        if let Some(font) = self.get_font(font_family) {
            for ch in text.chars() {
                let (metrics, bitmap) = font.rasterize(ch, font_size);

                glyphs.push(GlyphInfo {
                    character: ch,
                    bitmap,
                    metrics,
                    x: cursor_x,
                    y: 0.0,
                });

                cursor_x += metrics.advance_width;
                max_height = max_height.max(metrics.height as f32);
            }
        } else {
            // Fallback: estimate dimensions without real font
            let char_width = font_size * 0.6;
            cursor_x = text.len() as f32 * char_width;
            max_height = font_size;
        }

        TextRenderInfo {
            glyphs,
            total_width: cursor_x,
            total_height: max_height,
        }
    }
}

#[derive(Clone)]
pub struct GlyphInfo {
    pub character: char,
    pub bitmap: Vec<u8>, // Grayscale bitmap
    pub metrics: fontdue::Metrics,
    pub x: f32,
    pub y: f32,
}

pub struct TextRenderInfo {
    pub glyphs: Vec<GlyphInfo>,
    pub total_width: f32,
    pub total_height: f32,
}
