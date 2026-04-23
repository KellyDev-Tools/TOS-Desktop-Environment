use rodio::source::{SineWave, Source};
use rodio::{OutputStream, Sink};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{Sender, channel};

#[derive(Debug, Clone, Copy)]
pub enum AudioLayer {
    Ambient,
    Tactical,
    Voice,
}

enum AudioCommand {
    PlayEarcon(String),
    PlayAmbient(String), // For now, still synthetic
    StopAmbient,
    PlayVoice(String),   // Placeholder for TTS
    SetVolume(AudioLayer, f32),
}

pub struct AudioService {
    sender: Sender<AudioCommand>,
}

impl AudioService {
    pub fn new() -> (Self, Option<String>) {
        let (tx, rx) = channel::<AudioCommand>();
        let (warn_tx, warn_rx) = channel::<String>();

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

            // Sinks for each layer
            let tactical_sink = Sink::try_new(&stream_handle).ok();
            let ambient_sink = Sink::try_new(&stream_handle).ok();
            let voice_sink = Sink::try_new(&stream_handle).ok();

            // Default volumes
            if let Some(ref s) = tactical_sink { s.set_volume(0.15); }
            if let Some(ref s) = ambient_sink { s.set_volume(0.05); }
            if let Some(ref s) = voice_sink { s.set_volume(0.3); }

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    AudioCommand::PlayEarcon(name) => {
                        if let Some(ref sink) = tactical_sink {
                            let (freq, dur) = Self::get_earcon_params(&name);
                            let source = SineWave::new(freq)
                                .take_duration(Duration::from_millis(dur))
                                .amplify(1.0); // Volume controlled by sink
                            sink.append(source);
                        }
                    }
                    AudioCommand::PlayAmbient(name) => {
                        if let Some(ref sink) = ambient_sink {
                            sink.stop(); // Stop current ambient
                            // For now, synthetic "hum"
                            let freq = if name == "low_power" { 60.0 } else { 120.0 };
                            let source = SineWave::new(freq)
                                .amplify(0.5)
                                .repeat_infinite();
                            sink.append(source);
                        }
                    }
                    AudioCommand::StopAmbient => {
                        if let Some(ref sink) = ambient_sink {
                            sink.stop();
                        }
                    }
                    AudioCommand::PlayVoice(_text) => {
                        if let Some(ref sink) = voice_sink {
                            // Placeholder: play a "voice-like" chirping for now
                            let source = SineWave::new(880.0)
                                .take_duration(Duration::from_millis(100))
                                .amplify(1.0);
                            sink.append(source);
                        }
                    }
                    AudioCommand::SetVolume(layer, volume) => {
                        let target_sink = match layer {
                            AudioLayer::Tactical => &tactical_sink,
                            AudioLayer::Ambient => &ambient_sink,
                            AudioLayer::Voice => &voice_sink,
                        };
                        if let Some(sink) = target_sink {
                            sink.set_volume(volume);
                        }
                    }
                }
            }
        });

        // Give the thread a moment to report back
        let init_warning = warn_rx.recv_timeout(Duration::from_millis(100)).ok();

        (Self { sender: tx }, init_warning)
    }

    fn get_earcon_params(name: &str) -> (f32, u64) {
        match name {
            "system_ready" => (440.0, 400),
            "priority_mid_alert" => (660.0, 300),
            "priority_critical_alert" => (880.0, 500),
            "bezel_tap" => (1200.0, 50),
            "modal_open" => (550.0, 150),
            "modal_close" => (400.0, 150),
            "data_commit" => (950.0, 100),
            "nav_switch" => (700.0, 80),
            _ => (300.0, 200),
        }
    }

    /// Trigger a tactical earcon.
    pub fn play_earcon(&self, name: &str) {
        tracing::debug!("[AUDIO] Tactical earcon: {}", name);
        let _ = self.sender.send(AudioCommand::PlayEarcon(name.to_string()));
    }

    /// Start ambient background audio.
    pub fn play_ambient(&self, name: &str) {
        tracing::debug!("[AUDIO] Ambient start: {}", name);
        let _ = self.sender.send(AudioCommand::PlayAmbient(name.to_string()));
    }

    /// Stop ambient background audio.
    pub fn stop_ambient(&self) {
        tracing::debug!("[AUDIO] Ambient stop");
        let _ = self.sender.send(AudioCommand::StopAmbient);
    }

    /// Play a voice response.
    pub fn play_voice(&self, text: &str) {
        tracing::debug!("[AUDIO] Voice play: {}", text);
        let _ = self.sender.send(AudioCommand::PlayVoice(text.to_string()));
    }

    /// Set volume for a specific layer.
    pub fn set_volume(&self, layer: AudioLayer, volume: f32) {
        tracing::debug!("[AUDIO] Volume set: {:?} -> {}", layer, volume);
        let _ = self.sender.send(AudioCommand::SetVolume(layer, volume));
    }
}
