// use std::sync::{Arc, Mutex}; // Unused after TosState decoupling
// use crate::common::TosState; // Unused after TosState decoupling

pub struct AudioService {}

impl AudioService {
    pub fn new() -> Self {
        Self {}
    }

    /// Trigger a specific system earcon (audio notification).
    pub fn play_earcon(&self, name: &str) {
        // Alpha-2 logic: just log for verification
        println!("[EARCON TRIGGER] Playing cue: {}", name);
    }
}
