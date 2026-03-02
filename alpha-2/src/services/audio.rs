use std::thread;
use std::time::Duration;
use rodio::{OutputStream, Sink};
use rodio::source::{SineWave, Source};

pub struct AudioService {
    sender: std::sync::mpsc::Sender<String>,
}

impl AudioService {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        
        thread::spawn(move || {
            // Attempt to initialize the default audio output
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("Failed to initialize audio stream: {}", e);
                    return;
                }
            };
            
            while let Ok(name) = rx.recv() {
                // For Alpha-2, we generate a synthetic earcon based on the name
                // To keep the pipeline alive while sounds play
                if let Ok(sink) = Sink::try_new(&stream_handle) {
                    let freq = match name.as_str() {
                        "system_ready" => 440.0,
                        "priority_mid_alert" => 660.0,
                        "priority_critical_alert" => 880.0,
                        _ => 300.0,
                    };
                    
                    let source = SineWave::new(freq)
                        .take_duration(Duration::from_millis(200))
                        .amplify(0.20);
                        
                    sink.append(source);
                    sink.detach(); // Let it play out asynchronously
                }
            }
        });

        Self { sender: tx }
    }

    /// Trigger a specific system earcon (audio notification).
    pub fn play_earcon(&self, name: &str) {
        println!("[EARCON TRIGGER] Playing cue: {}", name);
        let _ = self.sender.send(name.to_string());
    }
}
