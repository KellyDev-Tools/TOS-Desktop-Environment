//! Headless renderer for environments without GPU or compositor.
//! 
//! Buffers are stored in CPU RAM. Per Architecture §15.3, this supports
//! testing, SSH environments, and remote streaming scenarios.

use std::collections::HashMap;
use crate::platform::{Renderer, SurfaceHandle, SurfaceConfig, SurfaceContent};

pub struct HeadlessRenderer {
    pub surfaces: HashMap<u32, HeadlessSurface>,
    next_handle: u32,
}

pub struct HeadlessSurface {
    pub config: SurfaceConfig,
    pub buffer: Vec<u8>, // RGBA or similar format
}

impl HeadlessRenderer {
    pub fn new() -> Self {
        Self {
            surfaces: HashMap::new(),
            next_handle: 1,
        }
    }
}

impl Renderer for HeadlessRenderer {
    fn create_surface(&mut self, config: SurfaceConfig) -> SurfaceHandle {
        let handle_id = self.next_handle;
        self.next_handle += 1;
        
        let buffer_size = (config.width * config.height * 4) as usize; // RGBA
        let surface = HeadlessSurface {
            config,
            buffer: vec![0u8; buffer_size],
        };
        
        self.surfaces.insert(handle_id, surface);
        SurfaceHandle(handle_id)
    }
    
    fn update_surface(&mut self, handle: SurfaceHandle, content: &dyn SurfaceContent) {
        if let Some(surface) = self.surfaces.get_mut(&handle.0) {
            // In a real implementation, we would copy pixel data here.
            // For now, we'll just track that an update happened.
            let data = content.pixel_data();
            if !data.is_empty() && data.len() <= surface.buffer.len() {
                surface.buffer[..data.len()].copy_from_slice(data);
            }
        }
    }

    fn register_pid(&mut self, _pid: u32, _handle: SurfaceHandle) {
        // No-op in headless
    }
    
    fn composite(&mut self) {
        // No GPU composite needed — surfaces are already in RAM
    }

    fn get_capture_backend(&self) -> std::sync::Arc<dyn crate::services::capture::CaptureBackend> {
        std::sync::Arc::new(HeadlessCaptureBackend)
    }
}

pub struct HeadlessCaptureBackend;

impl crate::services::capture::CaptureBackend for HeadlessCaptureBackend {
    fn capture_window(&self, _pid: u32) -> Option<crate::services::capture::FrameCapture> {
        // Headless mode currently returns a generic mock capture.
        // In the future, this would extract the CPU-side buffers.
        Some(crate::services::capture::FrameCapture {
            data: "data:image/raw;base64,SEVBRExFU1NfU05BUFNIT1RfUEFDVUVU".to_string(), // "HEADLESS_SNAPSHOT_PACKET"
            width: 320,
            height: 180,
        })
    }
}
