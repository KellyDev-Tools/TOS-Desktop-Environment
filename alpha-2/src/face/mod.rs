use crate::brain::ipc_handler::IpcHandler;
use crate::common::{TosState, HierarchyLevel};
use std::sync::{Arc, Mutex};

pub struct Face {
    brain_ipc: Arc<IpcHandler>,
    state: Arc<Mutex<TosState>>,
    _last_rendered_level: HierarchyLevel,
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
        }
    }

    /// §19.1 & §16.2: State Sync & Rendering
    pub fn render(&mut self) {
        let state = self.state.lock().unwrap();
        
        println!("\n[FACE RENDER] Level: {:?}", state.current_level);
        
        match state.current_level {
            HierarchyLevel::GlobalOverview => self.render_level1(&state),
            HierarchyLevel::CommandHub => self.render_level2(&state),
            _ => println!("(Level {:?} UI placeholder)", state.current_level),
        }
    }

    fn render_level1(&self, state: &TosState) {
        println!("--- GLOBAL OVERVIEW (L1) ---");
        println!("Sectors: {}", state.sectors.len());
        for (i, sector) in state.sectors.iter().enumerate() {
            let active_mark = if i == state.active_sector_index { "*" } else { " " };
            println!("  [{}{}] ID: {} | Hubs: {}", active_mark, i, sector.name, sector.hubs.len());
        }
        println!("---------------------------");
    }

    fn render_level2(&self, state: &TosState) {
        if let Some(sector) = state.sectors.get(state.active_sector_index) {
            let hub = &sector.hubs[sector.active_hub_index];
            println!("--- COMMAND HUB (L2): {} ---", sector.name);
            println!("Mode: {:?} | CWD: {}", hub.mode, hub.current_directory.display());
            println!("Output (Last 5 lines):");
            let start = hub.terminal_output.len().saturating_sub(5);
            for line in &hub.terminal_output[start..] {
                println!("[P{}] {}", line.priority, line.text);
            }
            println!("Prompt: > {}", hub.prompt);
            println!("----------------------------");
        }
    }

    /// §28.2: Bezel IPC Bridge
    pub fn send_event(&self, event: &str) {
        tracing::info!("Face Event -> Brain: {}", event);
        self.brain_ipc.handle_request(event);
    }
}

/// Mock Face for testing Level Transitions
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
}
