use anyhow::Result;
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA format
}

pub struct ImageCache {
    cache: Arc<Mutex<HashMap<String, ImageData>>>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn load_image(&self, url: &str) -> Result<ImageData> {
        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(img_data) = cache.get(url) {
                return Ok(img_data.clone());
            }
        }

        // Load the image
        let img_data = self.fetch_and_decode(url)?;

        // Store in cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(url.to_string(), img_data.clone());
        }

        Ok(img_data)
    }

    fn fetch_and_decode(&self, url: &str) -> Result<ImageData> {
        // For data URLs
        if url.starts_with("data:image/") {
            return self.decode_data_url(url);
        }

        // For file URLs
        if url.starts_with("file://") {
            let path = &url[7..];
            return self.load_from_file(path);
        }

        // For HTTP(S) URLs - simplified for now
        if url.starts_with("http://") || url.starts_with("https://") {
            return self.load_from_url(url);
        }

        // Treat as file path
        self.load_from_file(url)
    }

    fn load_from_file(&self, path: &str) -> Result<ImageData> {
        let img = image::open(path)?;
        Ok(Self::image_to_data(img))
    }

    fn load_from_url(&self, url: &str) -> Result<ImageData> {
        let response = reqwest::blocking::get(url)?;
        let bytes = response.bytes()?;

        let img = image::load_from_memory(&bytes)?;
        Ok(Self::image_to_data(img))
    }

    fn decode_data_url(&self, url: &str) -> Result<ImageData> {
        // Parse data URL: data:image/png;base64,iVBORw0KGgo...
        let parts: Vec<&str> = url.splitn(2, ',').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid data URL");
        }

        let header = parts[0];
        let data = parts[1];

        // Check if base64
        if header.contains("base64") {
            let decoded = base64::decode(data)?;
            let img = image::load_from_memory(&decoded)?;
            Ok(Self::image_to_data(img))
        } else {
            anyhow::bail!("Non-base64 data URLs not supported");
        }
    }

    fn image_to_data(img: DynamicImage) -> ImageData {
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8();
        let pixels = rgba.into_raw();

        ImageData {
            width,
            height,
            pixels,
        }
    }
}

// Base64 decode helper
mod base64 {
    use anyhow::Result;

    pub fn decode(input: &str) -> Result<Vec<u8>> {
        use std::io::Read;

        // Simple base64 decoder - in production use the base64 crate
        // For now, return error
        anyhow::bail!("Base64 decoding not yet implemented. Use the base64 crate.")
    }
}
