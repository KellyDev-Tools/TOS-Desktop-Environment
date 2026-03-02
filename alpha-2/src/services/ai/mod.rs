use std::sync::{Arc, Mutex};
use crate::common::TosState;
use crate::common::ipc_dispatcher::IpcDispatcher;
use serde_json::json;

pub struct AiService {
    ipc: Arc<Mutex<Option<Arc<dyn IpcDispatcher>>>>,
}

impl AiService {
    pub fn new() -> Self {
        Self { ipc: Arc::new(Mutex::new(None)) }
    }

    pub fn set_ipc(&self, ipc: Arc<dyn IpcDispatcher>) {
        let mut lock = self.ipc.lock().unwrap();
        *lock = Some(ipc);
    }

    /// Process natural language query and stage a command for user review.
    pub async fn query(&self, prompt: &str) -> anyhow::Result<()> {
        let ipc = {
            let lock = self.ipc.lock().unwrap();
            lock.clone()
        };
        let ipc = match ipc {
            Some(i) => i,
            None => return Err(anyhow::anyhow!("IPC dispatcher not set for AiService")),
        };
        let state_json = ipc.dispatch("get_state:");
        let clean_json = if let Some(idx) = state_json.rfind(" (") {
            &state_json[..idx]
        } else {
            &state_json
        };

        let state: TosState = match serde_json::from_str(clean_json) {
            Ok(s) => s,
            Err(e) => {
                println!("[AiService] Failed to parse state JSON: {}", e);
                return Err(e.into());
            }
        };
        
        let (current_dir, sector_names) = {
            let s_idx = state.active_sector_index;
            let hub_idx = state.sectors[s_idx].active_hub_index;
            let dir = state.sectors[s_idx].hubs[hub_idx].current_directory.display().to_string();
            let sectors = state.sectors.iter().map(|s| s.name.clone()).collect::<Vec<_>>().join(", ");
            (dir, sectors)
        };

        // Contextual Awareness - Mock logic using gathered system context
        let (command, explanation) = match prompt.to_lowercase().as_str() {
            p if p.contains("where") && p.contains("am") && p.contains("i") => {
                ("pwd".to_string(), format!("You are currently in sector {} at path {}.", sector_names, current_dir))
            },
            p if (p.contains("list") || p.contains("show")) && p.contains("files") => {
                ("ls -la".to_string(), "I've staged a command to list all files in long format, including hidden ones.".to_string())
            },
            p if p.contains("search") || p.contains("find") => {
                // Natural Language Search transition: Route to sector indexing
                let term = prompt.split_whitespace().last().unwrap_or("everything");
                let _ = ipc.dispatch(&format!("search:{}", term));
                ("zoom_to:CommandHub".to_string(), format!("Found matches for '{}'. Zooming to Command Hub results.", term))
            },
            _ => {
                (format!("echo 'AI suggest: {}'", prompt), "I've translated your request into a staged echo command for review.".to_string())
            }
        };

        let payload = json!({
            "command": command,
            "explanation": explanation
        });
        
        let _ = ipc.dispatch(&format!("ai_stage_command:{}", payload.to_string()));
        
        Ok(())
    }
}

