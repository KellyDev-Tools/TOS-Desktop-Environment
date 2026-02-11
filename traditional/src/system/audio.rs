pub struct AudioFeedback {
    pub enabled: bool,
    pub effects_enabled: bool, // Tactile chirps
    pub ambient_enabled: bool, // Background bridge hums
    pub queue: Vec<String>,
    ambient_timer: u32,
}

impl AudioFeedback {
    pub fn new() -> Self {
        Self { 
            enabled: true,
            effects_enabled: true,
            ambient_enabled: true,
            queue: Vec::new(),
            ambient_timer: 0,
        }
    }

    pub fn tick(&mut self) {
        if !self.enabled || !self.ambient_enabled {
            return;
        }

        self.ambient_timer += 1;
        
        // Every ~100 ticks (approx 10 seconds if tick is 100ms)
        if self.ambient_timer % 100 == 0 {
            self.play_sound("bridge_hum");
        }

        // Random pulses/beeps
        if self.ambient_timer % 47 == 0 {
            self.play_sound("console_pulse");
        }
    }

    pub fn play_sound(&mut self, sound_name: &str) {
        if self.enabled {
            // Check if it's an effect and if effects are enabled
            if (sound_name == "chirp" || sound_name == "beep") && !self.effects_enabled {
                return;
            }
            
            println!("[Audio] Queueing sound: {}", sound_name);
            self.queue.push(sound_name.to_string());
        }
    }

    pub fn consume_queue(&mut self) -> Vec<String> {
        self.queue.drain(..).collect()
    }
}
