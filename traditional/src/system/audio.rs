pub struct AudioFeedback {
    pub enabled: bool,
    pub effects_enabled: bool, // Tactile chirps
    pub queue: Vec<String>,
}

impl AudioFeedback {
    pub fn new() -> Self {
        Self { 
            enabled: true,
            effects_enabled: true,
            queue: Vec::new(),
        }
    }

    pub fn play_sound(&mut self, sound_name: &str) {
        if self.enabled {
            println!("[Audio] Queueing sound: {}", sound_name);
            self.queue.push(sound_name.to_string());
        }
    }

    pub fn consume_queue(&mut self) -> Vec<String> {
        self.queue.drain(..).collect()
    }
}
