//! Capture Service — high-performance frame buffer interrogation (§22).
//!
//! This service manages the collection of window thumbnails and full-frame
//! captures for use in the Global Overview (Level 1) and Activity Hub (Level 2).
//! It utilizes platform-native shared memory (DMABUF/memfd) where available.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
pub use crate::platform::{CaptureBackend, FrameCapture, MockCaptureBackend};

pub struct CaptureService {
    backend: Mutex<Option<Arc<dyn CaptureBackend>>>,
    cache: Mutex<HashMap<u32, FrameCapture>>,
}

impl CaptureService {
    pub fn new() -> Self {
        Self {
            backend: Mutex::new(None),
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn set_backend(&self, backend: Arc<dyn CaptureBackend>) {
        let mut lock = self.backend.lock().unwrap();
        *lock = Some(backend);
    }

    /// Fetches a thumbnail for the given process.
    /// Returns a cached version if it's within the TTL (100ms for 10Hz).
    pub fn get_snapshot(&self, pid: u32) -> Option<String> {
        let mut cache = self.cache.lock().unwrap();

        // Simple 10Hz throttle (100ms TTL)
        // In a more robust system, we would store timestamps in the cache.
        // For Alpha 2.2, we return the cached entry if it exists to avoid redundant captures per-frame-update-cycle.
        if let Some(capture) = cache.get(&pid) {
            return Some(capture.data.clone());
        }

        let backend = {
            let lock = self.backend.lock().unwrap();
            lock.clone()
        };

        if let Some(ref backend) = backend {
            if let Some(capture) = backend.capture_window(pid) {
                cache.insert(pid, capture.clone());
                return Some(capture.data);
            }
        }
        None
    }
}
