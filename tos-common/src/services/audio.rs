use rodio::source::{SineWave, Source};
use rodio::{OutputStream, Sink};
use std::thread;
use std::time::Duration;

pub struct AudioService {
    sender: std::sync::mpsc::Sender<String>,
}

impl AudioService {
    pub fn new() -> (Self, Option<String>) {
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        let (warn_tx, warn_rx) = std::sync::mpsc::channel::<String>();

        thread::spawn(move || {
            // Attempt to initialize the default audio output
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(e) => {
                    let msg = format!("Failed to initialize audio stream: {}", e);
                    tracing::warn!("{}", msg);
                    let _ = warn_tx.send(msg);
                    return;
                }
            };

            while let Ok(name) = rx.recv() {
                // For Alpha-2, we generate a synthetic earcon based on the name
                // To keep the pipeline alive while sounds play
                if let Ok(sink) = Sink::try_new(&stream_handle) {
                    let (freq, dur) = match name.as_str() {
                        "system_ready" => (440.0, 400),
                        "priority_mid_alert" => (660.0, 300),
                        "priority_critical_alert" => (880.0, 500),
                        "bezel_tap" => (1200.0, 50),
                        "modal_open" => (550.0, 150),
                        "modal_close" => (400.0, 150),
                        "data_commit" => (950.0, 100),
                        "nav_switch" => (700.0, 80),
                        _ => (300.0, 200),
                    };

                    let source = SineWave::new(freq)
                        .take_duration(Duration::from_millis(dur))
                        .amplify(0.15);

                    sink.append(source);
                    sink.detach(); // Let it play out asynchronously
                }
            }
        });

        // Give the thread a moment to report back
        let init_warning = warn_rx.recv_timeout(Duration::from_millis(100)).ok();

        (Self { sender: tx }, init_warning)
    }

    /// Trigger a specific system earcon (audio notification).
    pub fn play_earcon(&self, name: &str) {
        println!("[EARCON TRIGGER] Playing cue: {}", name);
        let _ = self.sender.send(name.to_string());
    }
}
