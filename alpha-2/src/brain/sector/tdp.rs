use serde::{Serialize, Deserialize};

/// TOS Display Protocol (TDP) v1 Framing
/// Optimized for low-latency (<50ms) remote desktop streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdpFrame {
    pub sector_id: uuid::Uuid,
    pub timestamp: u64,
    pub width: u32,
    pub height: u32,
    pub format: String, // "RGBA", "H264", "JPEG"
    pub payload: String, // Base64 encoded payload
    pub quality: u8,    // 0-100
}

pub struct TdpStreamer {
    pub current_quality: u8,
}

impl TdpStreamer {
    pub fn new() -> Self {
        Self { current_quality: 75 }
    }

    /// Encodes a raw frame buffer into a TDP frame.
    /// In the Alpha release, this prepares the snapshot for Face delivery.
    pub fn encode_frame(&self, sector_id: uuid::Uuid, raw_data: &[u8], width: u32, height: u32) -> TdpFrame {
        // In production, this would use turbojpeg or an H264 encoder.
        // For Alpha stage, we simulate the compression by converting to base64.
        let encoded_payload = base64::Engine::encode(&base64::prelude::BASE64_STANDARD, raw_data);
        
        TdpFrame {
            sector_id,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64,
            width,
            height,
            format: "RGBA".to_string(),
            payload: encoded_payload,
            quality: self.current_quality,
        }
    }

    pub fn set_quality(&mut self, quality: u8) {
        self.current_quality = quality.min(100);
        tracing::info!("TDP: Stream quality adjusted to {}", self.current_quality);
    }
}
