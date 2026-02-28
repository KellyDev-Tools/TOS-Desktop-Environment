use std::sync::{Arc, Mutex};
use crate::common::TosState;

pub struct AiService {
    state: Arc<Mutex<TosState>>,
}

impl AiService {
    pub fn new(state: Arc<Mutex<TosState>>) -> Self {
        Self { state }
    }

    /// §12: Process natural language query and stage a command
    pub async fn query(&self, prompt: &str) -> anyhow::Result<()> {
        let (current_dir, sector_names) = {
            let state = self.state.lock().unwrap();
            let s_idx = state.active_sector_index;
            let hub_idx = state.sectors[s_idx].active_hub_index;
            let dir = state.sectors[s_idx].hubs[hub_idx].current_directory.display().to_string();
            let sectors = state.sectors.iter().map(|s| s.name.clone()).collect::<Vec<_>>().join(", ");
            (dir, sectors)
        };

        // §12: Contextual Awareness - Mock logic using gathered context
        let (command, explanation) = match prompt.to_lowercase().as_str() {
            p if p.contains("where") && p.contains("am") && p.contains("i") => {
                ("pwd".to_string(), format!("You are currently in sector {} at path {}.", sector_names, current_dir))
            },
            p if (p.contains("list") || p.contains("show")) && p.contains("files") => {
                ("ls -la".to_string(), "I've staged a command to list all files in long format, including hidden ones.".to_string())
            },
            p if p.contains("search") || p.contains("find") => {
                // §12: Natural Language Search transition
                let term = prompt.split_whitespace().last().unwrap_or("everything");
                let mut state = self.state.lock().unwrap();
                crate::brain::sector::SectorManager::perform_search(&mut state, term);
                ("zoom_to:CommandHub".to_string(), format!("Found matches for '{}'. Zooming to Command Hub results.", term))
            },
            _ => {
                (format!("echo 'AI suggest: {}'", prompt), "I've translated your request into a staged echo command for review.".to_string())
            }
        };

        let mut state = self.state.lock().unwrap();
        let s_idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(s_idx) {
            let h_idx = sector.active_hub_index;
            if let Some(hub) = sector.hubs.get_mut(h_idx) {
                hub.staged_command = Some(command);
                hub.ai_explanation = Some(explanation);
            }
        }
        
        Ok(())
    }

    /// §12: Accept the staged command and promote it to the prompt or execute
    pub fn accept_suggestion(&self) -> anyhow::Result<()> {
        let mut state = self.state.lock().unwrap();
        let s_idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(s_idx) {
            let h_idx = sector.active_hub_index;
            if let Some(hub) = sector.hubs.get_mut(h_idx) {
                if let Some(cmd) = hub.staged_command.take() {
                    hub.prompt = cmd;
                    hub.ai_explanation = None;
                }
            }
        }
        Ok(())
    }
}

