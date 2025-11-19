use sha2::{Sha256, Digest};

/// Fingerprint protection against canvas, WebGL, and audio fingerprinting
pub struct FingerprintProtection {
    // Randomization seed per session
    session_seed: [u8; 32],

    // Protection level
    protect_canvas: bool,
    protect_webgl: bool,
    protect_audio: bool,
    protect_fonts: bool,
}

impl FingerprintProtection {
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Generate random seed for this session
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let mut hasher = Sha256::new();
        hasher.update(timestamp.to_le_bytes());
        let session_seed: [u8; 32] = hasher.finalize().into();

        Self {
            session_seed,
            protect_canvas: true,
            protect_webgl: true,
            protect_audio: true,
            protect_fonts: true,
        }
    }

    /// Add noise to canvas data
    /// This prevents canvas fingerprinting while keeping images visually identical
    pub fn protect_canvas_data(&self, data: &mut [u8]) {
        if !self.protect_canvas {
            return;
        }

        // Add imperceptible noise based on session seed
        // Change every 100th pixel by +/-1 (invisible to human eye)
        for (i, byte) in data.iter_mut().enumerate() {
            if i % 100 == 0 {
                let noise = (self.session_seed[i % 32] % 3) as i16 - 1; // -1, 0, or 1
                *byte = (*byte as i16 + noise).clamp(0, 255) as u8;
            }
        }
    }

    /// Protect WebGL parameters
    /// Returns modified parameters that prevent fingerprinting
    pub fn protect_webgl_params(&self, params: &str) -> String {
        if !self.protect_webgl {
            return params.to_string();
        }

        // Normalize WebGL parameters to common values
        // This makes all users look the same
        match params {
            "VENDOR" => "WebGL".to_string(),
            "RENDERER" => "Generic Renderer".to_string(),
            "VERSION" => "WebGL 1.0".to_string(),
            "SHADING_LANGUAGE_VERSION" => "WebGL GLSL ES 1.0".to_string(),
            _ => params.to_string(),
        }
    }

    /// Protect audio fingerprinting
    /// Add noise to AudioContext output
    pub fn protect_audio_data(&self, data: &mut [f32]) {
        if !self.protect_audio {
            return;
        }

        // Add imperceptible noise (< 0.001)
        for (i, sample) in data.iter_mut().enumerate() {
            let noise = (self.session_seed[i % 32] as f32 / 255.0 - 0.5) * 0.0001;
            *sample += noise;
        }
    }

    /// Get fake font list (prevents font enumeration)
    pub fn get_fake_font_list(&self) -> Vec<String> {
        if !self.protect_fonts {
            return Vec::new();
        }

        // Return common fonts only
        vec![
            "Arial".to_string(),
            "Helvetica".to_string(),
            "Times New Roman".to_string(),
            "Courier New".to_string(),
            "Verdana".to_string(),
        ]
    }

    /// Check if script is trying to fingerprint
    pub fn is_fingerprinting_attempt(&self, script: &str) -> bool {
        let fingerprint_keywords = [
            "toDataURL",
            "getImageData",
            "measureText",
            "getParameter",
            "AudioContext",
            "webkitAudioContext",
            "getChannelData",
            "navigator.plugins",
            "navigator.mimeTypes",
        ];

        fingerprint_keywords.iter().any(|keyword| script.contains(keyword))
    }

    /// Get statistics
    pub fn stats(&self) -> FingerprintStats {
        FingerprintStats {
            canvas_protected: self.protect_canvas,
            webgl_protected: self.protect_webgl,
            audio_protected: self.protect_audio,
            fonts_protected: self.protect_fonts,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FingerprintStats {
    pub canvas_protected: bool,
    pub webgl_protected: bool,
    pub audio_protected: bool,
    pub fonts_protected: bool,
}

impl Default for FingerprintProtection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_protection() {
        let fp = FingerprintProtection::new();
        let mut data = vec![128u8; 1000];
        let original = data.clone();

        fp.protect_canvas_data(&mut data);

        // Data should be slightly different
        assert_ne!(data, original);

        // But not too different (only ~1% changed)
        let diff_count = data.iter().zip(original.iter())
            .filter(|(a, b)| a != b)
            .count();
        assert!(diff_count < 20); // Less than 2% changed
    }

    #[test]
    fn test_fingerprinting_detection() {
        let fp = FingerprintProtection::new();

        assert!(fp.is_fingerprinting_attempt("canvas.toDataURL()"));
        assert!(fp.is_fingerprinting_attempt("gl.getParameter(gl.VENDOR)"));
        assert!(!fp.is_fingerprinting_attempt("console.log('hello')"));
    }
}
