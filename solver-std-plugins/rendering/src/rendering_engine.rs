// This module will eventually contain the full rendering pipeline
// For now, it's a placeholder

pub struct RenderingEngine {
    width: u32,
    height: u32,
}

impl RenderingEngine {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn render(&self, _html: &str) -> Vec<u8> {
        // TODO: Integrate full rendering pipeline from renderer/
        vec![0; (self.width * self.height * 4) as usize]
    }
}
