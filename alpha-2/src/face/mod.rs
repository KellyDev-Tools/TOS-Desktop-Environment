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
        
        // Simulating header
        println!("\x1B[1;36m[TOS DISPLAY ENGINE]\x1B[0m Syncing State... [\x1B[1;32mOK\x1B[0m]\n");
        
        match state.current_level {
            HierarchyLevel::GlobalOverview => self.render_level1(&state),
            HierarchyLevel::CommandHub => self.render_level2(&state),
            _ => {
                println!("+-----------------------------------------------+");
                println!("| {:^45} |", format!("{:?} VIEW", state.current_level));
                println!("| [PLACEHOLDER - ALPHA 2 PROTOTYPE]              |");
                println!("+-----------------------------------------------+");
            }
        }

        // §22: Tactical Mini-Map
        self.render_minimap(&state);

        // System Footer
        let time = chrono::Local::now().format("%H:%M:%S");
        let sector_name = state.sectors.get(state.active_sector_index)
            .map(|s| s.name.as_str()).unwrap_or("NONE");
        println!("\n\x1B[1;34m[ {} ]\x1B[0m SECTOR: \x1B[1;33m{}\x1B[0m | LEVEL: {:?} | BRAIN: \x1B[1;32mACTIVE\x1B[0m", time, sector_name, state.current_level);
    }

    fn render_level1(&self, state: &TosState) {
        println!("\x1B[1;35m[LEVEL 1: GLOBAL OVERVIEW]\x1B[0m\n");
        println!("+-----------------------------------------------+");
        println!("| SECTOR TILES                                  |");
        println!("+----------------------------+------------------+");
        for (i, sector) in state.sectors.iter().enumerate() {
            let active_mark = if i == state.active_sector_index { ">>" } else { "  " };
            println!("| {:<2} [ {} ] {:<18} | HUBS: {:<3} |", active_mark, i, sector.name, sector.hubs.len());
        }
        println!("+----------------------------+------------------+");
        
        println!("\n\x1B[1;36m[SYSTEM OUTPUT AREA (BRAIN LOG)]\x1B[0m");
        println!("+-----------------------------------------------+");
        let start = state.system_log.len().saturating_sub(5);
        for line in &state.system_log[start..] {
            let prio_color = match line.priority {
                3 => "\x1B[1;31m", // High
                2 => "\x1B[1;33m", // Mid
                _ => "\x1B[0m",    // Low
            };
            println!("| {}{} [P{}] {:<35} \x1B[0m |", prio_color, line.timestamp.format("%H:%M"), line.priority, line.text);
        }
        println!("+-----------------------------------------------+");
    }

    fn render_level2(&self, state: &TosState) {
        if let Some(sector) = state.sectors.get(state.active_sector_index) {
            let hub = &sector.hubs[sector.active_hub_index];
            println!("\x1B[1;35m[LEVEL 2: COMMAND HUB - {}]\x1B[0m\n", sector.name.to_uppercase());
            
            println!("MODE:  \x1B[1;32m{:?}\x1B[0m", hub.mode);
            println!("DIR:   \x1B[1;34m{}\x1B[0m", hub.current_directory.display());
            println!("\nOUTPUT:");
            println!("+-----------------------------------------------+");
            let start = hub.terminal_output.len().saturating_sub(10);
            for line in &hub.terminal_output[start..] {
                let text = if line.text.len() > 43 {
                    format!("{}...", &line.text[..40])
                } else {
                    line.text.clone()
                };
                println!("| {:<45} |", text);
            }
            println!("+-----------------------------------------------+");
            println!("\x1B[1;36mPROMPT:\x1B[0m > {}", hub.prompt);
        }
    }

    fn render_minimap(&self, state: &TosState) {
        println!("\n\x1B[1;33m[TACTICAL MINI-MAP §22]\x1B[0m");
        for (i, sector) in state.sectors.iter().enumerate() {
            let active = i == state.active_sector_index;
            let color = if active { "\x1B[1;32m" } else { "\x1B[0m" };
            print!("{}S{} ", color, i);
            for (j, _hub) in sector.hubs.iter().enumerate() {
                let hub_active = active && j == sector.active_hub_index;
                let hub_color = if hub_active { "\x1B[1;36m" } else { "\x1B[0m" };
                print!(" {}.{} ", hub_color, j);
            }
            println!("\x1B[0m");
        }
    }

    /// §28.2: Bezel IPC Bridge
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
}

