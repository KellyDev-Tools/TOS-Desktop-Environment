pub struct AudioFeedback {
    pub enabled: bool,
}

impl AudioFeedback {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub fn play_sound(&self, sound_name: &str) {
        if self.enabled {
            println!("[Audio] Playing sound: {}", sound_name);
        }
    }
}
