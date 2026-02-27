use crate::common::{TosState, CommandHubMode, HierarchyLevel};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use std::path::PathBuf;
use std::time::Instant;

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
        let start = Instant::now();
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
            "zoom_in" => self.handle_zoom_in(),
            "zoom_out" => self.handle_zoom_out(),
            "zoom_to" => self.handle_zoom_to(args.get(0).copied()),
            "set_setting" => self.handle_set_setting(args.get(0).copied(), args.get(1).copied()),
            "sector_create" => self.handle_sector_create(args.get(0).copied()),
            "sector_clone" => self.handle_sector_clone(args.get(0).copied()),
            "sector_close" => self.handle_sector_close(args.get(0).copied()),
            "sector_freeze" => self.handle_sector_freeze(args.get(0).copied()),
            "search" => self.handle_search(payload),
            "prompt_submit" => self.handle_prompt_submit(payload), // Entire payload is the command
            "update_confirmation_progress" => self.handle_update_confirmation_progress(args.get(0).copied(), args.get(1).copied()),
            _ => tracing::warn!("Unknown IPC prefix: {}", prefix),
        }

        let duration = start.elapsed();
        if duration.as_millis() > 16 {
            tracing::warn!("IPC LATENCY WARNING: {} took {:?}", prefix, duration);
        } else {
            tracing::debug!("IPC handled: {} ({:?})", prefix, duration);
        }
    }

    fn is_dangerous(&self, command: &str) -> bool {
        let cmd = command.trim().to_lowercase();
        // Simple list for Alpha-2 prototype §17.3
        cmd.starts_with("rm -rf") || 
        cmd.starts_with("format") || 
        cmd.starts_with("mkfs") || 
        cmd.contains("> /dev/sd")
    }

    fn handle_update_confirmation_progress(&self, id_str: Option<&str>, val_str: Option<&str>) {
        if let (Some(id_s), Some(val_s)) = (id_str, val_str) {
            if let (Ok(id), Ok(val)) = (Uuid::parse_str(id_s), val_s.parse::<f32>()) {
                let mut state = self.state.lock().unwrap();
                if let Some(conf) = &mut state.pending_confirmation {
                    if conf.id == id {
                        conf.progress = val;
                        // If progress reached 100%, execute and clear
                        if val >= 1.0 {
                            let original = conf.original_request.clone();
                            state.pending_confirmation = None;
                            drop(state); // Drop before writing to shell
                            self.execute_final_command(&original);
                        }
                    }
                }
            }
        }
    }

    fn execute_final_command(&self, command: &str) {
        let mut shell = self.shell.lock().unwrap();
        let _ = shell.write(&format!("{}\n", command));
    }

    fn handle_prompt_submit(&self, command: &str) {
        // §17.3: Dangerous Command Handling
        if self.is_dangerous(command) {
            let mut state = self.state.lock().unwrap();
            state.pending_confirmation = Some(crate::common::ConfirmationRequest {
                id: Uuid::new_v4(),
                original_request: command.to_string(),
                message: format!("DANGEROUS COMMAND DETECTED: '{}'. Drag to confirm.", command),
                progress: 0.0,
            });
            tracing::warn!("Intercepted dangerous command: {}", command);
            return;
        }

        // §28.1: Prompt Interception Layer (sniffing for ls/cd)
        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        let cmd_lower = command.to_lowercase();

        if cmd_lower.starts_with("ls") {
            // §27.6: Resolve target path and switch to Directory mode
            if let Some(sector) = state.sectors.get_mut(idx) {
                if let Some(hub) = sector.hubs.get_mut(sector.active_hub_index) {
                    hub.mode = CommandHubMode::Directory;
                }
            }
            crate::brain::sector::SectorManager::refresh_directory_listing(&mut state);
        } else if cmd_lower.starts_with("cd ") {
            // §27.6: Resolve target path and update current_directory
            let new_path_str = &command[3..].trim();
            if let Some(sector) = state.sectors.get_mut(idx) {
                if let Some(hub) = sector.hubs.get_mut(sector.active_hub_index) {
                    let mut new_path = hub.current_directory.clone();
                    new_path.push(new_path_str);
                    // Minimal validation: if it's absolute, use it
                    if PathBuf::from(new_path_str).is_absolute() {
                        hub.current_directory = PathBuf::from(new_path_str);
                    } else {
                        hub.current_directory = new_path;
                    }
                }
            }
        }
        
        // Final submission to PTY
        let mut shell = self.shell.lock().unwrap();
        let _ = shell.write(&format!("{}\n", command));
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
                if mode == CommandHubMode::Directory {
                    crate::brain::sector::SectorManager::refresh_directory_listing(&mut state);
                } else if mode == CommandHubMode::Activity {
                    crate::brain::sector::SectorManager::refresh_activity_listing(&mut state);
                }
            }
        }
    }

    fn handle_zoom_in(&self) {
        let mut state = self.state.lock().unwrap();
        crate::brain::hierarchy::HierarchyManager::zoom_in(&mut state);
    }

    fn handle_zoom_out(&self) {
        let mut state = self.state.lock().unwrap();
        crate::brain::hierarchy::HierarchyManager::zoom_out(&mut state);
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
        crate::brain::hierarchy::HierarchyManager::set_level(&mut state, level);
    }

    fn handle_set_setting(&self, key: Option<&str>, val: Option<&str>) {
        if let (Some(k), Some(v)) = (key, val) {
            let mut state = self.state.lock().unwrap();
            state.settings.insert(k.to_string(), v.to_string());
        }
    }

    fn handle_sector_create(&self, name: Option<&str>) {
        let mut state = self.state.lock().unwrap();
        crate::brain::sector::SectorManager::create_sector(
            &mut state, 
            name.unwrap_or("New Sector").to_string()
        );
    }

    fn handle_sector_clone(&self, id_str: Option<&str>) {
        if let Some(id_str) = id_str {
            if let Ok(id) = Uuid::parse_str(id_str) {
                let mut state = self.state.lock().unwrap();
                crate::brain::sector::SectorManager::clone_sector(&mut state, id);
            }
        }
    }

    fn handle_sector_close(&self, id_str: Option<&str>) {
        if let Some(id_str) = id_str {
            if let Ok(id) = Uuid::parse_str(id_str) {
                let mut state = self.state.lock().unwrap();
                crate::brain::sector::SectorManager::close_sector(&mut state, id);
            }
        }
    }

    fn handle_sector_freeze(&self, id_str: Option<&str>) {
        if let Some(id_str) = id_str {
            if let Ok(id) = Uuid::parse_str(id_str) {
                let mut state = self.state.lock().unwrap();
                crate::brain::sector::SectorManager::toggle_freeze(&mut state, id);
            }
        }
    }

    fn handle_search(&self, query: &str) {
        let mut state = self.state.lock().unwrap();
        crate::brain::sector::SectorManager::perform_search(&mut state, query);
    }
}
