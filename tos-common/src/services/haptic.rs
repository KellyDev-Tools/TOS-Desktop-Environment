use std::thread;
use std::time::Duration;
#[cfg(target_os = "android")]
use std::fs;

pub struct HapticService {
    sender: std::sync::mpsc::Sender<String>,
}

impl HapticService {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<String>();

        thread::spawn(move || {
            while let Ok(cue) = rx.recv() {
                // Define haptic patterns as a sequence of (vibrate_duration, pause_after) in ms
                let pattern: Vec<(u64, u64)> = match cue.as_str() {
                    "click" => vec![(20, 0)],
                    "success" => vec![(30, 50), (40, 0)],
                    "error" => vec![(50, 50), (50, 50), (100, 0)],
                    "long_press" => vec![(80, 0)],
                    _ => vec![(50, 0)],
                };

                tracing::debug!("[HAPTIC TRIGGER] Cue: {}, Pattern: {:?}", cue, pattern);

                for (duration, pause) in pattern {
                    // Write directly to the sysfs vibrator node on Android
                    #[cfg(target_os = "android")]
                    {
                        let _ = fs::write("/sys/class/timed_output/vibrator/enable", format!("{}", duration));
                    }

                    // Simulated or fallback physical delay
                    thread::sleep(Duration::from_millis(duration));

                    if pause > 0 {
                        thread::sleep(Duration::from_millis(pause));
                    }
                }
            }
        });

        Self { sender: tx }
    }

    /// Trigger a tactical haptic burst.
    pub fn trigger_haptic(&self, cue: &str) {
        let _ = self.sender.send(cue.to_string());
    }
}
