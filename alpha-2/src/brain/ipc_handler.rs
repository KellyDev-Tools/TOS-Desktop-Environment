use crate::common::{TosState, CommandHubMode, HierarchyLevel};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use std::path::PathBuf;
use std::time::Instant;

pub struct IpcHandler {
    state: Arc<Mutex<TosState>>,
    shell: Arc<Mutex<crate::brain::shell::ShellApi>>,
    services: Arc<crate::services::ServiceManager>,
}

impl IpcHandler {
    pub fn new(state: Arc<Mutex<TosState>>, shell: Arc<Mutex<crate::brain::shell::ShellApi>>, services: Arc<crate::services::ServiceManager>) -> Self {
        Self { state, shell, services }
    }

    /// Standardized Message Format: prefix:payload;payload...
    pub fn handle_request(&self, request: &str) -> String {
        let start = Instant::now();
        let parts: Vec<&str> = request.splitn(2, ':').collect();
        if parts.len() < 2 {
            tracing::warn!("Malformed IPC request: {}", request);
            return "ERROR: Malformed request".to_string();
        }

        let prefix = parts[0];
        let payload = parts[1];
        let args: Vec<&str> = payload.split(';').collect();

        let result = match prefix {
            "get_state" => self.handle_get_state(),
            "set_mode" => self.handle_set_mode(args.get(0).copied()),
            "zoom_in" => self.handle_zoom_in(),
            "zoom_out" => self.handle_zoom_out(),
            "zoom_to" => self.handle_zoom_to(args.get(0).copied()),
            "set_setting" => self.handle_set_setting(args.get(0).copied(), args.get(1).copied()),
            "set_sector_setting" => self.handle_set_sector_setting(args.get(0).copied(), args.get(1).copied(), args.get(2).copied()),
            "sector_create" => self.handle_sector_create(args.get(0).copied()),
            "sector_clone" => self.handle_sector_clone(args.get(0).copied()),
            "sector_close" => self.handle_sector_close(args.get(0).copied()),
            "sector_freeze" => self.handle_sector_freeze(args.get(0).copied()),
            "search" => self.handle_search(payload),
            "prompt_submit" => self.handle_prompt_submit(payload), 
            "update_confirmation_progress" => self.handle_update_confirmation_progress(args.get(0).copied(), args.get(1).copied()),
            "ai_submit" => self.handle_ai_submit(payload),
            "ai_suggestion_accept" => self.handle_ai_suggestion_accept(),
            "ai_stage_command" => self.handle_ai_stage_command(payload),
            "system_log_append" => self.handle_system_log_append(args.get(0).copied(), args.get(1).copied()),
            _ => "ERROR: Unknown prefix".to_string(),
        };

        let duration = start.elapsed();
        if duration.as_millis() > 16 {
            tracing::warn!("IPC LATENCY WARNING: {} took {:?}", prefix, duration);
        }
        
        format!("{} ({:?})", result, duration)
    }

    fn is_dangerous(&self, command: &str) -> bool {
        let cmd = command.trim().to_lowercase();
        // Simple list for dangerous command detection
        cmd.starts_with("rm -rf") || 
        cmd.starts_with("format") || 
        cmd.starts_with("mkfs") || 
        cmd.contains("> /dev/sd")
    }

    fn handle_update_confirmation_progress(&self, id_str: Option<&str>, val_str: Option<&str>) -> String {
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
                            return format!("CONFIRMED: {}", original);
                        }
                        return format!("PROGRESS: {}", val);
                    }
                }
            }
        }
        "ERROR: Invalid confirmation update".to_string()
    }

    fn execute_final_command(&self, command: &str) {
        let mut shell = self.shell.lock().unwrap();
        let _ = shell.write(&format!("{}\n", command));
    }

    fn handle_prompt_submit(&self, command: &str) -> String {
        // Dangerous Command Handling: Intercept and request confirmation
        if self.is_dangerous(command) {
            self.services.logger.audit_log("SessionUser", "EXECUTE_DANGEROUS", command);
            let mut state = self.state.lock().unwrap();
            let id = Uuid::new_v4();
            state.pending_confirmation = Some(crate::common::ConfirmationRequest {
                id,
                original_request: command.to_string(),
                message: format!("DANGEROUS COMMAND DETECTED: '{}'. Drag to confirm.", command),
                progress: 0.0,
            });
            tracing::warn!("Intercepted dangerous command: {}", command);
            return format!("INTERCEPTED: {}", id);
        }

        // Prompt Interception Layer (sniffing for ls/cd)
        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        let cmd_lower = command.to_lowercase();

        if cmd_lower.starts_with("ls") {
            // Resolve target path and switch to Directory mode
            if let Some(sector) = state.sectors.get_mut(idx) {
                if let Some(hub) = sector.hubs.get_mut(sector.active_hub_index) {
                    hub.mode = CommandHubMode::Directory;
                }
            }
            crate::brain::sector::SectorManager::refresh_directory_listing(&mut state);
        } else if cmd_lower.starts_with("cd ") {
            // Resolve target path and update current_directory
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
        "SUBMITTED".to_string()
    }

    fn handle_set_mode(&self, mode_str: Option<&str>) -> String {
        // Context-Aware Mode Transitions
        let mode = match mode_str {
            Some("command") => CommandHubMode::Command,
            Some("directory") => CommandHubMode::Directory,
            Some("activity") => CommandHubMode::Activity,
            Some("search") => CommandHubMode::Search,
            Some("ai") => CommandHubMode::Ai,
            _ => return "ERROR: Unknown mode".to_string(),
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
                return format!("MODE_SET: {:?}", mode);
            }
        }
        "ERROR: Hub not found".to_string()
    }

    fn handle_zoom_in(&self) -> String {
        let mut state = self.state.lock().unwrap();
        crate::brain::hierarchy::HierarchyManager::zoom_in(&mut state);
        "ZOOMED_IN".to_string()
    }

    fn handle_zoom_out(&self) -> String {
        let mut state = self.state.lock().unwrap();
        crate::brain::hierarchy::HierarchyManager::zoom_out(&mut state);
        "ZOOMED_OUT".to_string()
    }

    fn handle_zoom_to(&self, level_str: Option<&str>) -> String {
        // Multi-Level Navigation
        let level = match level_str {
            Some("GlobalOverview") => HierarchyLevel::GlobalOverview,
            Some("CommandHub") => HierarchyLevel::CommandHub,
            Some("ApplicationFocus") => HierarchyLevel::ApplicationFocus,
            Some("DetailView") => HierarchyLevel::DetailView,
            Some("BufferView") => HierarchyLevel::BufferView,
            _ => return "ERROR: Unknown level".to_string(),
        };

        let mut state = self.state.lock().unwrap();
        crate::brain::hierarchy::HierarchyManager::set_level(&mut state, level);
        format!("ZOOMED_TO: {:?}", level)
    }

    fn handle_set_setting(&self, key: Option<&str>, val: Option<&str>) -> String {
        if let (Some(k), Some(v)) = (key, val) {
            let mut state = self.state.lock().unwrap();
            state.settings.global.insert(k.to_string(), v.to_string());
            // Implicit save - in production this would debounce via daemon
            let _ = self.services.settings.save(&state.settings);
            return format!("SETTING_UPDATE: {}={}", k, v);
        }
        "ERROR: Invalid setting args".to_string()
    }

    fn handle_set_sector_setting(&self, sector_id: Option<&str>, key: Option<&str>, val: Option<&str>) -> String {
        if let (Some(sec), Some(k), Some(v)) = (sector_id, key, val) {
            let mut state = self.state.lock().unwrap();
            let entry = state.settings.sectors.entry(sec.to_string()).or_insert_with(std::collections::HashMap::new);
            entry.insert(k.to_string(), v.to_string());
            let _ = self.services.settings.save(&state.settings);
            return format!("SECTOR_SETTING_UPDATE: [{}] {}={}", sec, k, v);
        }
        "ERROR: Invalid sector setting args".to_string()
    }

    fn handle_sector_create(&self, name: Option<&str>) -> String {
        // Dynamic Sector Allocation
        let mut state = self.state.lock().unwrap();
        let name = name.unwrap_or("New Sector");
        crate::brain::sector::SectorManager::create_sector(
            &mut state, 
            name.to_string()
        );
        format!("SECTOR_CREATED: {}", name)
    }

    fn handle_sector_clone(&self, id_str: Option<&str>) -> String {
        if let Some(id_str) = id_str {
            if let Ok(id) = Uuid::parse_str(id_str) {
                let mut state = self.state.lock().unwrap();
                crate::brain::sector::SectorManager::clone_sector(&mut state, id);
                return format!("SECTOR_CLONED: {}", id);
            }
        }
        "ERROR: Invalid sector ID for clone".to_string()
    }

    fn handle_sector_close(&self, id_str: Option<&str>) -> String {
        if let Some(id_str) = id_str {
            if let Ok(id) = Uuid::parse_str(id_str) {
                let mut state = self.state.lock().unwrap();
                crate::brain::sector::SectorManager::close_sector(&mut state, id);
                return format!("SECTOR_CLOSED: {}", id);
            }
        }
        "ERROR: Invalid sector ID for close".to_string()
    }

    fn handle_sector_freeze(&self, id_str: Option<&str>) -> String {
        if let Some(id_str) = id_str {
            if let Ok(id) = Uuid::parse_str(id_str) {
                let mut state = self.state.lock().unwrap();
                crate::brain::sector::SectorManager::toggle_freeze(&mut state, id);
                return format!("SECTOR_FREEZE_TOGGLED: {}", id);
            }
        }
        "ERROR: Invalid sector ID for freeze".to_string()
    }

    fn handle_search(&self, query: &str) -> String {
        let mut state = self.state.lock().unwrap();
        crate::brain::sector::SectorManager::perform_search(&mut state, query);
        
        // Add globally indexed file matches
        let indexed_hits = self.services.search.query(query);
        if !indexed_hits.is_empty() {
            let idx = state.active_sector_index;
            if let Some(sector) = state.sectors.get_mut(idx) {
                let hub = &mut sector.hubs[sector.active_hub_index];
                
                let matches: Vec<String> = indexed_hits.iter()
                    .map(|h| format!("{} [{}]", h.path, if h.is_dir { "DIR" } else { "FILE" }))
                    .collect();
                    
                if let Some(ref mut results) = hub.search_results {
                    results.insert(0, crate::common::SearchResult {
                        source_sector: "Global FS Index".to_string(),
                        matches,
                    });
                } else {
                    hub.search_results = Some(vec![crate::common::SearchResult {
                        source_sector: "Global FS Index".to_string(),
                        matches,
                    }]);
                }
            }
        }
        
        format!("SEARCH_PERFORMED: {}", query)
    }

    fn handle_ai_submit(&self, query: &str) -> String {
        let ai = self.services.ai.clone();
        let query_owned = query.to_string();
        tokio::spawn(async move {
            let _ = ai.query(&query_owned).await;
        });
        "AI_PROCESSING".to_string()
    }

    fn handle_ai_suggestion_accept(&self) -> String {
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
        "AI_SUGGESTION_ACCEPTED".to_string()
    }

    fn handle_ai_stage_command(&self, payload: &str) -> String {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(payload) {
            let mut state = self.state.lock().unwrap();
            let s_idx = state.active_sector_index;
            if let Some(sector) = state.sectors.get_mut(s_idx) {
                let h_idx = sector.active_hub_index;
                if let Some(hub) = sector.hubs.get_mut(h_idx) {
                    if let Some(cmd) = parsed.get("command").and_then(|v| v.as_str()) {
                        hub.staged_command = Some(cmd.to_string());
                    }
                    if let Some(expl) = parsed.get("explanation").and_then(|v| v.as_str()) {
                        hub.ai_explanation = Some(expl.to_string());
                    }
                }
            }
            return "AI_COMMAND_STAGED".to_string();
        }
        "ERROR: Invalid JSON for ai_stage_command".to_string()
    }

    fn handle_system_log_append(&self, priority_str: Option<&str>, text_str: Option<&str>) -> String {
        if let (Some(priority), Some(text)) = (priority_str, text_str) {
            let mut state = self.state.lock().unwrap();
            let priority = priority.parse::<u8>().unwrap_or(1);
            let limit = 1000;
            
            state.system_log.push(crate::common::TerminalLine {
                text: text.to_string(),
                priority,
                timestamp: chrono::Local::now(),
            });
            
            if state.system_log.len() > limit {
                let to_drain = state.system_log.len() - limit;
                state.system_log.drain(0..to_drain);
            }
            return "LOGGED".to_string();
        }
        "ERROR: Invalid arguments for log".to_string()
    }

    fn handle_get_state(&self) -> String {
        let state = self.state.lock().unwrap();
        serde_json::to_string(&*state).unwrap_or_else(|_| "ERROR: Serialization failed".to_string())
    }
}

impl crate::common::ipc_dispatcher::IpcDispatcher for IpcHandler {
    fn dispatch(&self, request: &str) -> String {
        self.handle_request(request)
    }
}
