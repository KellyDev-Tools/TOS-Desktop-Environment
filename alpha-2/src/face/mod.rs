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
                println!("+----------------------------------------------------------------------------------+");
                println!("| {:^80} |", format!("{:?} VIEW", state.current_level));
                println!("| [PLACEHOLDER - ALPHA 2 PROTOTYPE]                                                |");
                println!("+----------------------------------------------------------------------------------+");
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
        println!("+----------------------------------------------------------------------------------+");
        println!("| SECTOR TILES                                                                     |");
        println!("+--------------------------------------------------------------+-------------------+");
        for (i, sector) in state.sectors.iter().enumerate() {
            let active_mark = if i == state.active_sector_index { ">>" } else { "  " };
            // The format string needs to account for the fixed parts and the variable parts
            // Total width inside the frame is 80.
            // Fixed parts: "| {:<2} [ {} ] {:<X} | HUBS: {:<Y} |"
            // Let's calculate X and Y.
            // active_mark: 2 chars
            // i: max 2 chars (e.g., 99) -> "[ 99 ]" is 6 chars
            // " | HUBS: " is 9 chars
            // hubs.len(): max 3 chars (e.g., 999)
            // So, 2 + 1 + 6 + 1 + X + 1 + 9 + 3 + 1 = 24 + X
            // Total width for the two columns is 62 and 17.
            // First column: 2 + 1 + 6 + 1 + X = 10 + X. So 10 + X = 62 -> X = 52.
            // Second column: 9 + Y + 1 = 10 + Y. So 10 + Y = 17 -> Y = 7.
            println!("| {:<2} [ {:<2} ] {:<52} | HUBS: {:<7} |", active_mark, i, sector.name, sector.hubs.len());
        }
        println!("+--------------------------------------------------------------+-------------------+");
        
        println!("\n\x1B[1;36m[SYSTEM OUTPUT AREA (BRAIN LOG)]\x1B[0m");
        println!("+----------------------------------------------------------------------------------+");
        let start = state.system_log.len().saturating_sub(5);
        for line in &state.system_log[start..] {
            let prio_color = match line.priority {
                3 => "\x1B[1;31m", // High
                2 => "\x1B[1;33m", // Mid
                _ => "\x1B[0m",    // Low
            };
            // Fixed parts: "| {}{} [P{}] {:<X} \x1B[0m |"
            // timestamp: 5 chars (HH:MM)
            // priority: 1 char (P1)
            // prio_color: 0 chars (ANSI escape codes don't count)
            // Total width inside frame is 80.
            // 5 + 1 + 4 + X + 1 = 11 + X. So 11 + X = 80 -> X = 69.
            println!("| {}{} [P{}] {:<69} \x1B[0m |", prio_color, line.timestamp.format("%H:%M"), line.priority, line.text);
        }
        println!("+----------------------------------------------------------------------------------+");
    }

    fn render_level2(&self, state: &TosState) {
        if let Some(sector) = state.sectors.get(state.active_sector_index) {
            let hub = &sector.hubs[sector.active_hub_index];
            println!("\x1B[1;35m[LEVEL 2: COMMAND HUB - {}]\x1B[0m\n", sector.name.to_uppercase());
            
            println!("MODE:  \x1B[1;32m{:?}\x1B[0m", hub.mode);
            println!("DIR:   \x1B[1;34m{}\x1B[0m", hub.current_directory.display());
            println!("\nOUTPUT:");
            println!("+----------------------------------------------------------------------------------+");
            let start = hub.terminal_output.len().saturating_sub(10);
            for line in &hub.terminal_output[start..] {
                // Extended width to 80 chars
                // The content inside the frame is 80 characters wide.
                // If the text is longer than 80, we need to truncate it and add "..."
                // So, if text.len() > 80, truncate to 77 and add "..."
                let text = if line.text.len() > 80 {
                    format!("{}...", &line.text[..77])
                } else {
                    line.text.clone()
                };
                println!("| {:<80} |", text);
            }
            println!("+----------------------------------------------------------------------------------+");
            println!("\x1B[1;36mPROMPT:\x1B[0m > {}", hub.prompt);

            if let Some(staged) = &hub.staged_command {
                println!("\n\x1B[1;33m[AI STAGED COMMAND §12]\x1B[0m");
                println!("PROPOSAL: \x1B[1;32m{}\x1B[0m", staged);
                if let Some(exp) = &hub.ai_explanation {
                    println!("RATIONALE: {}", exp);
                }
                println!("\x1B[1;36m(Submit 'ai_suggestion_accept:' to promote this command)\x1B[0m");
            }
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

    pub fn simulate_ai_submit(&self, query: &str) {
        println!("(Face) User submitted AI query: {}", query);
        self.0.send_event(&format!("ai_submit:{}", query));
    }

    pub fn simulate_ai_accept(&self) {
        println!("(Face) User accepted AI suggestion");
        self.0.send_event("ai_suggestion_accept:");
    }
}
