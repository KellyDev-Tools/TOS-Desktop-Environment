use crate::common::{TosState, CommandHubMode, HierarchyLevel};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct IpcHandler {
    state: Arc<Mutex<TosState>>,
    shell: Arc<Mutex<crate::brain::shell::ShellApi>>,
}

impl IpcHandler {
    pub fn new(state: Arc<Mutex<TosState>>, shell: Arc<Mutex<crate::brain::shell::ShellApi>>) -> Self {
        Self { state, shell }
    }

    /// §3.3.1: Standardized Message Format: prefix:payload;payload...
    pub fn handle_request(&self, request: &str) {
        let parts: Vec<&str> = request.splitn(2, ':').collect();
        if parts.len() < 2 {
            tracing::warn!("Malformed IPC request: {}", request);
            return;
        }

        let prefix = parts[0];
        let payload = parts[1];
        let args: Vec<&str> = payload.split(';').collect();

        match prefix {
            "set_mode" => self.handle_set_mode(args.get(0).copied()),
            "zoom_to" => self.handle_zoom_to(args.get(0).copied()),
            "set_setting" => self.handle_set_setting(args.get(0).copied(), args.get(1).copied()),
            "sector_close" => self.handle_sector_close(args.get(0).copied()),
            "prompt_submit" => self.handle_prompt_submit(payload), // Entire payload is the command
            _ => tracing::warn!("Unknown IPC prefix: {}", prefix),
        }
    }

    fn handle_set_mode(&self, mode_str: Option<&str>) {
        let mode = match mode_str {
            Some("command") => CommandHubMode::Command,
            Some("directory") => CommandHubMode::Directory,
            Some("activity") => CommandHubMode::Activity,
            Some("search") => CommandHubMode::Search,
            Some("ai") => CommandHubMode::Ai,
            _ => return,
        };

        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let hub_idx = sector.active_hub_index;
            if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                hub.mode = mode;
            }
        }
    }

    fn handle_zoom_to(&self, level_str: Option<&str>) {
        let level = match level_str {
            Some("GlobalOverview") => HierarchyLevel::GlobalOverview,
            Some("CommandHub") => HierarchyLevel::CommandHub,
            Some("ApplicationFocus") => HierarchyLevel::ApplicationFocus,
            Some("DetailView") => HierarchyLevel::DetailView,
            Some("BufferView") => HierarchyLevel::BufferView,
            _ => return,
        };

        let mut state = self.state.lock().unwrap();
        state.current_level = level;
    }

    fn handle_set_setting(&self, key: Option<&str>, val: Option<&str>) {
        if let (Some(k), Some(v)) = (key, val) {
            let mut state = self.state.lock().unwrap();
            state.settings.insert(k.to_string(), v.to_string());
        }
    }

    fn handle_sector_close(&self, id_str: Option<&str>) {
        if let Some(id_str) = id_str {
            if let Ok(id) = Uuid::parse_str(id_str) {
                let mut state = self.state.lock().unwrap();
                state.sectors.retain(|s| s.id != id);
            }
        }
    }

    fn handle_prompt_submit(&self, command: &str) {
        // §28.1: Prompt Interception Layer (sniffing for ls/cd)
        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        let cmd_lower = command.to_lowercase();

        if cmd_lower.starts_with("ls") {
            // Logic to switch to Directory mode
            if let Some(sector) = state.sectors.get_mut(idx) {
                let hub_idx = sector.active_hub_index;
                if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                    hub.mode = CommandHubMode::Directory;
                }
            }
        }
        
        // Final submission to PTY
        let mut shell = self.shell.lock().unwrap();
        let _ = shell.write(&format!("{}\n", command));
    }
}
