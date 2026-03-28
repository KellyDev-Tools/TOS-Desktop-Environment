//! Android Face — the "Representative Layer" for Android devices.
//!
//! Handles input capture (touch, gestures) and visual rendering.
//! Integrates with the Brain via IPC channel for state synchronization.

use std::sync::{Arc, Mutex};
pub use tos_common as common; // Re-export for convenience or use directly
use tos_common::state::{TosState, HierarchyLevel};
use crate::input::AndroidInput;
use crate::services::AndroidServices;

use crate::api;

/// Event types for Android Face
#[derive(Debug, Clone)]
pub enum Event {
    TouchDown { x: f32, y: f32, pointer_id: i32 },
    TouchUp { pointer_id: i32 },
    TouchMove { x: f32, y: f32, pointer_id: i32 },
    Key { key_code: i32, action: i32 },
    ConfigurationChanged,
    LowMemory,
    Resumed,
    Paused,
    ScreenOn,
    ScreenOff,
}

/// Android Face — main entry point for the Android platform layer.
pub struct AndroidFace {
    /// IPC sender for communicating with the Brain
    pub brain_ipc: Arc<tokio::sync::mpsc::Sender<String>>,
    /// Shared state reference
    pub state: Arc<Mutex<TosState>>,
    /// Window surface (from NDK)
    pub window: Option<api::Window>,
    /// Touch and gesture input handler
    pub input: AndroidInput,
    /// System services (file, clipboard, notifications)
    pub services: AndroidServices,
    /// Activity state
    pub activity_state: api::ActivityState,
    /// Last processed event
    pub last_event: Option<Event>,
}

impl Default for AndroidFace {
    fn default() -> Self {
        Self {
            brain_ipc: Arc::new(tokio::sync::mpsc::channel(128).0),
            state: Arc::new(Mutex::new(TosState::default())),
            window: None,
            input: AndroidInput::default(),
            services: AndroidServices::default(),
            activity_state: api::ActivityState::default(),
            last_event: None,
        }
    }
}

impl AndroidFace {
    /// Create a new Android Face connected to a Brain IPC channel.
    pub fn new(
        brain_ipc: Arc<tokio::sync::mpsc::Sender<String>>,
        state: Arc<Mutex<TosState>>,
    ) -> Self {
        Self {
            brain_ipc,
            state,
            window: None,
            input: AndroidInput::default(),
            services: AndroidServices::default(),
            activity_state: api::ActivityState::default(),
            last_event: None,
        }
    }

    /// Attach an existing window surface.
    pub fn with_window(mut self, window: api::Window) -> Self {
        self.window = Some(window);
        self
    }

    // -----------------------------------------------------------------------
    // Command Processing
    // -----------------------------------------------------------------------

    /// Process a batch of commands from the Android activity main loop.
    pub fn process_commands(&mut self, commands: Vec<api::Command>) {
        for cmd in commands {
            match cmd {
                api::Command::Lifecycle(lc) => self.handle_lifecycle(lc),
                api::Command::InputEvent(ev) => self.handle_input_event(ev),
                api::Command::WindowCreated(w) => {
                    tracing::info!("Android Face: Window created");
                    self.activity_state.window = Some(w);
                }
                api::Command::WindowDestroyed => {
                    tracing::info!("Android Face: Window destroyed");
                    self.activity_state.window = None;
                }
                api::Command::ConfigChanged(config) => {
                    tracing::info!("Android Face: Config changed");
                    self.activity_state.config = Some(config);
                    self.last_event = Some(Event::ConfigurationChanged);
                }
                api::Command::Finish => {
                    tracing::info!("Android Face: Finish requested");
                }
            }
        }
    }

    fn handle_lifecycle(&mut self, lifecycle: api::Lifecycle) {
        match lifecycle {
            api::Lifecycle::Resumed => {
                tracing::info!("Android Face: App resumed");
                self.last_event = Some(Event::Resumed);
            }
            api::Lifecycle::Paused => {
                tracing::info!("Android Face: App paused");
                self.last_event = Some(Event::Paused);
            }
            api::Lifecycle::ConfigChanged => {
                tracing::info!("Android Face: Configuration changed");
                self.last_event = Some(Event::ConfigurationChanged);
            }
            api::Lifecycle::LowMemory => {
                tracing::warn!("Android Face: Low memory warning");
                self.last_event = Some(Event::LowMemory);
            }
            api::Lifecycle::ScreenOn => {
                tracing::info!("Android Face: Screen on");
                self.last_event = Some(Event::ScreenOn);
            }
            api::Lifecycle::ScreenOff => {
                tracing::info!("Android Face: Screen off");
                self.last_event = Some(Event::ScreenOff);
            }
        }
    }

    fn handle_input_event(&mut self, event: api::InputEvent) {
        match event {
            api::InputEvent::Touch(motion) => {
                match motion.action {
                    api::TouchAction::Down => {
                        self.last_event = Some(Event::TouchDown {
                            x: motion.x,
                            y: motion.y,
                            pointer_id: motion.pointer_id,
                        });
                    }
                    api::TouchAction::Up => {
                        self.last_event = Some(Event::TouchUp {
                            pointer_id: motion.pointer_id,
                        });
                    }
                    api::TouchAction::Move => {
                        self.last_event = Some(Event::TouchMove {
                            x: motion.x,
                            y: motion.y,
                            pointer_id: motion.pointer_id,
                        });
                    }
                }
            }
            api::InputEvent::Key { key_code, action } => {
                self.last_event = Some(Event::Key { key_code, action });
            }
        }
    }

    // -----------------------------------------------------------------------
    // Rendering
    // -----------------------------------------------------------------------

    /// Render the current frame.
    pub fn render(&mut self) {
        self.sync_state();
        let _frame = self.render_to_string();

        if self.window.is_some() {
            // In production: submit frame via EGL / Vulkan
            tracing::debug!("Android Face: Frame buffered for rendering");
        }
    }

    /// Sync state from Brain IPC.
    fn sync_state(&mut self) {
        // Currently state is shared via Arc<Mutex<TosState>>
        // Future: receive incremental state diffs via IPC channel
    }

    /// Render the current state to a text frame.
    pub fn render_to_string(&self) -> String {
        use std::fmt::Write;
        let state = self.state.lock().unwrap();
        let mut out = String::new();

        writeln!(out, "[TOS ANDROID FACE] Syncing State... [OK]\n").unwrap();

        match state.current_level {
            HierarchyLevel::GlobalOverview => self.render_level1_to(&state, &mut out),
            HierarchyLevel::CommandHub => self.render_level2_to(&state, &mut out),
            HierarchyLevel::Marketplace => {
                writeln!(out, "+----------------------------------------------------------------------------------+").unwrap();
                writeln!(out, "| [MARKETPLACE COMPRISING ALL KNOWN MODULES]                                       |").unwrap();
                writeln!(out, "| Use the Web Interface to browse, search, and install new behaviors.              |").unwrap();
                writeln!(out, "+----------------------------------------------------------------------------------+").unwrap();
            }
            _ => {
                writeln!(out, "+{:-^82}+", "").unwrap();
                writeln!(out, "| {:^80} |", format!("{:?} VIEW", state.current_level)).unwrap();
                writeln!(out, "| [PLACEHOLDER - ANDROID ALPHA 2 PROTOTYPE]                                        |").unwrap();
                writeln!(out, "+{:-^82}+", "").unwrap();
            }
        }

        // Tactical Mini-Map
        self.render_minimap_to(&state, &mut out);

        // System footer
        let sector_name = state.sectors.get(state.active_sector_index)
            .map(|s| s.name.as_str())
            .unwrap_or("NONE");
        writeln!(
            out,
            "\n[ {} ] SECTOR: {} | LEVEL: {:?} | BRAIN: ACTIVE",
            chrono::Local::now().format("%H:%M:%S"),
            sector_name,
            state.current_level
        ).unwrap();

        // Android-specific footer
        writeln!(
            out,
            "\n[ ANDROID ] Device: {} | SDK: {}",
            self.get_device_info(),
            self.get_sdk_version()
        ).unwrap();

        out
    }

    fn render_level1_to(&self, state: &TosState, out: &mut String) {
        use std::fmt::Write;
        writeln!(out, "[LEVEL 1: GLOBAL OVERVIEW]\n").unwrap();
        writeln!(out, "+----------------------------------------------------------------------------------+").unwrap();
        writeln!(out, "| SECTOR TILES                                                                     |").unwrap();
        writeln!(out, "+--------------------------------------------------------------+-------------------+").unwrap();
        for (i, sector) in state.sectors.iter().enumerate() {
            let active_mark = if i == state.active_sector_index { ">>" } else { "  " };
            writeln!(out, "| {:<2} [ {:<2} ] {:<52} | HUBS: {:<7} |",
                active_mark, i, sector.name, sector.hubs.len()).unwrap();
        }
        writeln!(out, "+--------------------------------------------------------------+-------------------+").unwrap();

        writeln!(out, "\n[SYSTEM OUTPUT AREA (BRAIN LOG)]").unwrap();
        writeln!(out, "+----------------------------------------------------------------------------------+").unwrap();
        let start = state.system_log.len().saturating_sub(5);
        for line in &state.system_log[start..] {
            writeln!(out, "| {} [P{}] {:<69}  |",
                line.timestamp.format("%H:%M"), line.priority, line.text).unwrap();
        }
        writeln!(out, "+----------------------------------------------------------------------------------+").unwrap();
    }

    fn render_level2_to(&self, state: &TosState, out: &mut String) {
        use std::fmt::Write;
        if let Some(sector) = state.sectors.get(state.active_sector_index) {
            let hub = &sector.hubs[sector.active_hub_index];
            writeln!(out, "[LEVEL 2: COMMAND HUB - {}]\n", sector.name.to_uppercase()).unwrap();
            writeln!(out, "MODE:  {:?}", hub.mode).unwrap();
            writeln!(out, "DIR:   {}", hub.current_directory.display()).unwrap();
            writeln!(out, "\nOUTPUT:").unwrap();
            writeln!(out, "+----------------------------------------------------------------------------------+").unwrap();
            let start = hub.terminal_output.len().saturating_sub(10);
            for line in &hub.terminal_output[start..] {
                let text = if line.text.len() > 80 {
                    format!("{}...", &line.text[..77])
                } else {
                    line.text.clone()
                };
                writeln!(out, "| {:<80} |", text).unwrap();
            }
            writeln!(out, "+----------------------------------------------------------------------------------+").unwrap();
            writeln!(out, "PROMPT: > {}", hub.prompt).unwrap();

            if let Some(staged) = &hub.staged_command {
                writeln!(out, "\n[AI STAGED COMMAND]").unwrap();
                writeln!(out, "PROPOSAL: {}", staged).unwrap();
                if let Some(exp) = &hub.ai_explanation {
                    writeln!(out, "RATIONALE: {}", exp).unwrap();
                }
            }
        }
    }

    fn render_minimap_to(&self, state: &TosState, out: &mut String) {
        use std::fmt::Write;
        writeln!(out, "\n[TACTICAL MINI-MAP]").unwrap();
        for (i, sector) in state.sectors.iter().enumerate() {
            write!(out, "S{} ", i).unwrap();
            for (j, _hub) in sector.hubs.iter().enumerate() {
                write!(out, " .{} ", j).unwrap();
            }
            writeln!(out).unwrap();
        }
    }

    // -----------------------------------------------------------------------
    // IPC Events
    // -----------------------------------------------------------------------

    /// Forward a named event to the Brain IPC dispatcher.
    pub fn send_event(&self, event: &str) {
        tracing::info!("Android Face Event -> Brain: {}", event);
        if let Err(e) = self.brain_ipc.clone().try_send(event.to_string()) {
            tracing::error!("Failed to send event to Brain: {}", e);
        }
    }

    /// Send a touch coordinate to the Brain.
    pub fn send_touch_event(&self, x: f32, y: f32) {
        self.send_event(&format!("touch:{}:{}", x, y));
    }

    /// Send a gesture name to the Brain.
    pub fn send_gesture_event(&self, gesture: &str) {
        self.send_event(&format!("gesture:{}", gesture));
    }

    // -----------------------------------------------------------------------
    // Device Info (stubs — real impl uses android.os.Build)
    // -----------------------------------------------------------------------

    fn get_device_info(&self) -> &str {
        "Android Device"
    }

    fn get_sdk_version(&self) -> &str {
        "34"
    }
}
