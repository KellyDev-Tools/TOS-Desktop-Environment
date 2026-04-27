use rodio::source::{SineWave, Source};
use rodio::{OutputStream, Sink};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy)]
pub enum AudioLayer {
    Ambient,
    Tactical,
    Voice,
}

enum AudioCommand {
    PlayEarcon(String),
    PlaySpatialEarcon(String, [f32; 3]),
    PlayAmbient(String), // For now, still synthetic
    StopAmbient,
    PlayVoice(String),   // Placeholder for TTS
    SetVolume(AudioLayer, f32),
    LoadModule(String),
}

pub struct AudioService {
    sender: Sender<AudioCommand>,
    module_manager: Arc<Mutex<Option<Arc<crate::brain::module_manager::ModuleManager>>>>,
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
            let spatial_sink = rodio::SpatialSink::try_new(
                &stream_handle,
                [0.0, 0.0, 0.0],
                [-0.05, 0.0, 0.0],
                [0.05, 0.0, 0.0],
            ).ok();

            // Default volumes
            if let Some(ref s) = tactical_sink { s.set_volume(0.15); }
            if let Some(ref s) = ambient_sink { s.set_volume(0.05); }
            if let Some(ref s) = voice_sink { s.set_volume(0.3); }
            if let Some(ref s) = spatial_sink { s.set_volume(0.2); }

            let mut earcon_assets: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
            let mut ambient_assets: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    AudioCommand::LoadModule(manifest_json) => {
                        if let Ok(manifest) = serde_json::from_str::<crate::services::marketplace::ModuleManifest>(&manifest_json) {
                            if let Some(audio) = manifest.audio {
                                // In a real implementation, we would load these from the module directory
                                // For v0.1, we'll look in ~/.config/tos/modules/audio/<id>/assets/
                                let mut base = dirs::home_dir().unwrap_or_default();
                                base.push(".config/tos/modules/audio");
                                base.push(&manifest.id);
                                base.push("assets");

                                // Load earcons mapping
                                let e_path = base.join("earcons.json");
                                if let Ok(content) = std::fs::read_to_string(e_path) {
                                    if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, String>>(&content) {
                                        for (name, file) in map {
                                            if let Ok(bytes) = std::fs::read(base.join(file)) {
                                                earcon_assets.insert(name, bytes);
                                            }
                                        }
                                    }
                                }
                                
                                // Load ambient mapping
                                let a_path = base.join("ambient.json");
                                if let Ok(content) = std::fs::read_to_string(a_path) {
                                    if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, String>>(&content) {
                                        for (name, file) in map {
                                            if let Ok(bytes) = std::fs::read(base.join(file)) {
                                                ambient_assets.insert(name, bytes);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    AudioCommand::PlayEarcon(name) => {
                        if let Some(ref sink) = tactical_sink {
                            if let Some(bytes) = earcon_assets.get(&name) {
                                let cursor = std::io::Cursor::new(bytes.clone());
                                if let Ok(source) = rodio::Decoder::new(cursor) {
                                    sink.append(source);
                                    continue;
                                }
                            }
                            
                            let (freq, dur) = Self::get_earcon_params(&name);
                            let source = SineWave::new(freq)
                                .take_duration(Duration::from_millis(dur))
                                .amplify(1.0);
                            sink.append(source);
                        }
                    }
                    AudioCommand::PlaySpatialEarcon(name, pos) => {
                        if let Some(ref sink) = spatial_sink {
                            sink.set_emitter_position(pos);
                            
                            if let Some(bytes) = earcon_assets.get(&name) {
                                let cursor = std::io::Cursor::new(bytes.clone());
                                if let Ok(source) = rodio::Decoder::new(cursor) {
                                    sink.append(source);
                                    continue;
                                }
                            }
                            
                            let (freq, dur) = Self::get_earcon_params(&name);
                            let source = SineWave::new(freq)
                                .take_duration(Duration::from_millis(dur))
                                .amplify(1.0);
                            sink.append(source);
                        }
                    }
                    AudioCommand::PlayAmbient(name) => {
                        if let Some(ref sink) = ambient_sink {
                            sink.stop();
                            
                            if let Some(bytes) = ambient_assets.get(&name) {
                                let cursor = std::io::Cursor::new(bytes.clone());
                                if let Ok(source) = rodio::Decoder::new(cursor) {
                                    sink.append(source.repeat_infinite());
                                    continue;
                                }
                            }

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

        (Self { 
            sender: tx,
            module_manager: Arc::new(Mutex::new(None)),
        }, init_warning)
    }

    pub fn set_module_manager(&self, mm: Arc<crate::brain::module_manager::ModuleManager>) {
        *self.module_manager.lock().unwrap() = Some(mm);
    }

    pub fn load_audio_module(&self, id: &str) -> anyhow::Result<()> {
        let mm_lock = self.module_manager.lock().unwrap();
        let mm = mm_lock.as_ref().ok_or_else(|| anyhow::anyhow!("ModuleManager not set"))?;
        let manifest = mm.get_manifest(id).ok_or_else(|| anyhow::anyhow!("Module not found"))?;
        
        if manifest.module_type != "audio" {
            return Err(anyhow::anyhow!("Module is not an audio bundle"));
        }

        let json = serde_json::to_string(manifest)?;
        let _ = self.sender.send(AudioCommand::LoadModule(json));
        Ok(())
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

    /// Trigger a spatial tactical earcon.
    pub fn play_spatial_earcon(&self, name: &str, x: f32, y: f32, z: f32) {
        tracing::debug!("[AUDIO] Spatial earcon: {} at [{}, {}, {}]", name, x, y, z);
        let _ = self.sender.send(AudioCommand::PlaySpatialEarcon(name.to_string(), [x, y, z]));
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
