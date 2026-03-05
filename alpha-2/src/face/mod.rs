use crate::brain::ipc_handler::IpcHandler;
use crate::common::{TosState, HierarchyLevel};
use crate::platform::Renderer;
use std::sync::{Arc, Mutex};

pub struct Face {
    brain_ipc: Arc<IpcHandler>,
    state: Arc<Mutex<TosState>>,
    _last_rendered_level: HierarchyLevel,
    renderer: Option<Box<dyn Renderer + Send>>,
    /// Cached native surface handle — created once, reused every frame.
    surface_handle: Option<crate::platform::SurfaceHandle>,
}

impl Face {
    pub fn new(state: Arc<Mutex<TosState>>, ipc: Arc<IpcHandler>) -> Self {
        let initial_level = {
            let s = state.lock().unwrap();
            s.current_level
        };

        Self {
            brain_ipc: ipc,
            state,
            _last_rendered_level: initial_level,
            renderer: None,
            surface_handle: None,
        }
    }

    pub fn with_renderer(mut self, renderer: Box<dyn Renderer + Send>) -> Self {
        self.renderer = Some(renderer);
        self
    }

    /// Synchronize system state and trigger rendering.
    pub fn render(&mut self) {
        let frame = self.render_to_string();
        // Clear screen, hide cursor, print frame, restore cursor.
        print!("\x1B[?25l\x1B[2J\x1B[H");
        print!("{}", frame);
        print!("\x1B[?25h");

        // Native Surface Synchronization
        if let Some(renderer) = &mut self.renderer {
            let handle = match self.surface_handle {
                Some(h) => h,
                None => {
                    let config = crate::platform::SurfaceConfig { width: 1920, height: 1080 };
                    let h = renderer.create_surface(config);
                    self.surface_handle = Some(h);
                    h
                }
            };

            struct NativeFrame;
            impl crate::platform::SurfaceContent for NativeFrame {
                fn pixel_data(&self) -> &[u8] {
                    &[0u8; 100]
                }
            }

            renderer.update_surface(handle, &NativeFrame);
            renderer.composite();
            tracing::debug!("Native Linux Face: Syncing frame buffer to Wayland SHM");
        }
    }

    /// Render the current frame to a String (testable — no ANSI clear/cursor control).
    pub fn render_to_string(&self) -> String {
        let state = self.state.lock().unwrap();
        let mut out = String::new();

        out.push_str("[TOS DISPLAY ENGINE] Syncing State... [OK]\n\n");

        match state.current_level {
            HierarchyLevel::GlobalOverview => self.render_level1_to(&state, &mut out),
            HierarchyLevel::CommandHub => self.render_level2_to(&state, &mut out),
            _ => {
                out.push_str(&format!("+{:->82}+\n", ""));
                out.push_str(&format!("| {:^80} |\n", format!("{:?} VIEW", state.current_level)));
                out.push_str("| [PLACEHOLDER - ALPHA 2 PROTOTYPE]                                                |\n");
                out.push_str(&format!("+{:->82}+\n", ""));
            }
        }

        // Tactical Mini-Map
        self.render_minimap_to(&state, &mut out);

        // System Footer
        let time = chrono::Local::now().format("%H:%M:%S");
        let sector_name = state.sectors.get(state.active_sector_index)
            .map(|s| s.name.as_str()).unwrap_or("NONE");
        out.push_str(&format!(
            "\n[ {} ] SECTOR: {} | LEVEL: {:?} | BRAIN: ACTIVE\n",
            time, sector_name, state.current_level
        ));

        out
    }

    // --- String-buffer render variants (for render_to_string / testing) ---

    fn render_level1_to(&self, state: &TosState, out: &mut String) {
        use std::fmt::Write;
        writeln!(out, "[LEVEL 1: GLOBAL OVERVIEW]\n").unwrap();
        writeln!(out, "+----------------------------------------------------------------------------------+").unwrap();
        writeln!(out, "| SECTOR TILES                                                                     |").unwrap();
        writeln!(out, "+--------------------------------------------------------------+-------------------+").unwrap();
        for (i, sector) in state.sectors.iter().enumerate() {
            let active_mark = if i == state.active_sector_index { ">>" } else { "  " };
            writeln!(out, "| {:<2} [ {:<2} ] {:<52} | HUBS: {:<7} |", active_mark, i, sector.name, sector.hubs.len()).unwrap();
        }
        writeln!(out, "+--------------------------------------------------------------+-------------------+").unwrap();

        writeln!(out, "\n[SYSTEM OUTPUT AREA (BRAIN LOG)]").unwrap();
        writeln!(out, "+----------------------------------------------------------------------------------+").unwrap();
        let start = state.system_log.len().saturating_sub(5);
        for line in &state.system_log[start..] {
            writeln!(out, "| {} [P{}] {:<69}  |", line.timestamp.format("%H:%M"), line.priority, line.text).unwrap();
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

    /// Forward bezel button events to the Brain IPC dispatcher.
    pub fn send_event(&self, event: &str) {
        tracing::info!("Face Event -> Brain: {}", event);
        self.brain_ipc.handle_request(event);
    }
}

pub struct MockFace(pub Face);

impl MockFace {
    pub fn simulate_bezel_zoom_in(&self) {
        println!("(Face) User clicked ZOOM IN bezel button");
        self.0.send_event("zoom_in:");
    }

    pub fn simulate_prompt_submit(&self, cmd: &str) {
        println!("(Face) User submitted prompt: {}", cmd);
        self.0.send_event(&format!("prompt_submit:{}", cmd));
    }

    pub fn simulate_ai_submit(&self, query: &str) {
        println!("(Face) User submitted AI query: {}", query);
        self.0.send_event(&format!("ai_submit:{}", query));
    }

    pub fn simulate_ai_accept(&self) {
        println!("(Face) User accepted AI suggestion");
        self.0.send_event("ai_suggestion_accept:");
    }
}
