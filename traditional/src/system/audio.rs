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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_initial_state() {
        let audio = AudioFeedback::new();
        assert!(audio.enabled);
        assert!(audio.effects_enabled);
        assert!(audio.ambient_enabled);
        assert_eq!(audio.queue.len(), 0);
    }

    #[test]
    fn test_audio_ambient_timer_triggers_bridge_hum() {
        let mut audio = AudioFeedback::new();
        
        // Tick 100 times to trigger bridge_hum
        for _ in 0..100 {
            audio.tick();
        }
        
        assert!(audio.queue.contains(&"bridge_hum".to_string()));
    }

    #[test]
    fn test_audio_ambient_timer_triggers_console_pulse() {
        let mut audio = AudioFeedback::new();
        
        // Tick 47 times to trigger console_pulse
        for _ in 0..47 {
            audio.tick();
        }
        
        assert!(audio.queue.contains(&"console_pulse".to_string()));
    }

    #[test]
    fn test_audio_ambient_disabled_prevents_tick_sounds() {
        let mut audio = AudioFeedback::new();
        audio.ambient_enabled = false;
        
        // Tick 100 times - should not queue anything
        for _ in 0..100 {
            audio.tick();
        }
        
        assert_eq!(audio.queue.len(), 0);
    }

    #[test]
    fn test_audio_disabled_prevents_all_sounds() {
        let mut audio = AudioFeedback::new();
        audio.enabled = false;
        
        audio.play_sound("test_sound");
        assert_eq!(audio.queue.len(), 0);
        
        audio.tick();
        assert_eq!(audio.queue.len(), 0);
    }

    #[test]
    fn test_effects_disabled_blocks_chirps() {
        let mut audio = AudioFeedback::new();
        audio.effects_enabled = false;
        
        // Chirps and beeps should be blocked
        audio.play_sound("chirp");
        audio.play_sound("beep");
        assert_eq!(audio.queue.len(), 0);
        
        // Other sounds should still work
        audio.play_sound("zoom_in");
        assert_eq!(audio.queue.len(), 1);
        assert_eq!(audio.queue[0], "zoom_in");
    }

    #[test]
    fn test_effects_enabled_allows_chirps() {
        let mut audio = AudioFeedback::new();
        assert!(audio.effects_enabled);
        
        audio.play_sound("chirp");
        audio.play_sound("beep");
        assert_eq!(audio.queue.len(), 2);
    }

    #[test]
    fn test_audio_queue_consumption() {
        let mut audio = AudioFeedback::new();
        
        audio.play_sound("sound1");
        audio.play_sound("sound2");
        audio.play_sound("sound3");
        assert_eq!(audio.queue.len(), 3);
        
        let consumed = audio.consume_queue();
        assert_eq!(consumed.len(), 3);
        assert_eq!(consumed[0], "sound1");
        assert_eq!(consumed[1], "sound2");
        assert_eq!(consumed[2], "sound3");
        
        // Queue should be empty after consumption
        assert_eq!(audio.queue.len(), 0);
    }

    #[test]
    fn test_multiple_tick_cycles() {
        let mut audio = AudioFeedback::new();
        
        // First cycle: tick 47 times (should get console_pulse)
        for _ in 0..47 {
            audio.tick();
        }
        assert_eq!(audio.queue.len(), 1);
        audio.consume_queue();
        
        // Continue ticking to 100 (should get bridge_hum)
        for _ in 47..100 {
            audio.tick();
        }
        assert!(audio.queue.contains(&"bridge_hum".to_string()));
    }

    #[test]
    fn test_play_sound_basic() {
        let mut audio = AudioFeedback::new();
        
        audio.play_sound("zoom_in");
        assert_eq!(audio.queue.len(), 1);
        assert_eq!(audio.queue[0], "zoom_in");
    }
}
