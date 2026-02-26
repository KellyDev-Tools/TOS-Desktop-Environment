use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use crate::system::pty::PtyHandle;
use crate::system::remote::{RemoteNodeInfo, RemoteStatus};
use crate::system::collaboration::{CollaborationRole, PermissionSet};
use crate::{TosState, CommandHubMode, HierarchyLevel, system::input::SemanticEvent, ConnectionType};
use crate::system::collaboration::Participant;

pub struct IpcDispatcher {
    pub state: Arc<Mutex<TosState>>,
    pub ptys: Arc<Mutex<HashMap<Uuid, PtyHandle>>>,
}

impl IpcDispatcher {
    pub fn new(state: Arc<Mutex<TosState>>, ptys: Arc<Mutex<HashMap<Uuid, PtyHandle>>>) -> Self {
        Self { state, ptys }
    }

    pub fn handle_request(&self, request: &str) {
        let mut state = self.state.lock().expect("Failed to lock state");
        
        if request.starts_with("set_mode:") {
            let mode_str = &request[9..];
            let mode = match mode_str {
                "Command" => CommandHubMode::Command,
                "Directory" => CommandHubMode::Directory,
                "Activity" => CommandHubMode::Activity,
                "Search" => CommandHubMode::Search,
                "Ai" => CommandHubMode::Ai,
                _ => return,
            };
            state.toggle_mode(mode);

                let (hub_id, cwd) = {
                    let viewport = &state.viewports[state.active_viewport_index];
                    let sector_idx = viewport.sector_index;
                    let hub_idx = viewport.hub_index;
                    let hub = &state.sectors[sector_idx].hubs[hub_idx];
                    (hub.id, hub.current_directory.to_string_lossy().to_string())
                };

                drop(state); // release lock before locking ptys
                if let Ok(ptys) = self.ptys.lock() {
                    if let Some(pty) = ptys.get(&hub_id) {
                        // Use the Shell API format: ls -la <path>
                        let ls_cmd = format!("ls -la {}\n", cwd);
                        pty.write(&ls_cmd);
                    }
                }
                return;
        } else if request.starts_with("select_sector:") {
            if let Ok(idx) = request[14..].parse::<usize>() {
                state.select_sector(idx);
            }
        } else if request.starts_with("prompt_submit:") {
            let cmd = request[14..].to_string();
            let (sector_idx, hub_idx) = {
                let viewport = &state.viewports[state.active_viewport_index];
                (viewport.sector_index, viewport.hub_index)
            };
            let hub_id = {
                let hub = &mut state.sectors[sector_idx].hubs[hub_idx];
                hub.prompt.clear(); // Section 13.2: Clear prompt on submission
                hub.confirmation_required = None;
                hub.terminal_output.push(format!("> {}", cmd)); // Immediate local echo
                if hub.terminal_output.len() > 100 { hub.terminal_output.remove(0); }
                hub.id
            };
            self.handle_prompt_submit(&mut state, &cmd);
            
            // §13.2: If handle_prompt_submit didn't intercept (zoom/mode), write to PTY.
            // But we must do it outside the state lock to avoid deadlocks with PtyHandle::poll_all
            drop(state);
            if let Ok(ptys) = self.ptys.lock() {
                if let Some(pty) = ptys.get(&hub_id) {
                    // Check if it was a system command already handled
                    let handled_system = ["zoom", "mode", "focus", "in", "out"].iter().any(|&s| cmd.starts_with(s));
                    if !handled_system {
                         pty.write(&format!("{}\n", cmd));
                    }
                }
            }
            return; // Exit handle_request as we already dropped state and finished logic
        } else if request.starts_with("prompt_input:") {
            let partial = &request[13..];
            state.set_prompt(partial.to_string());
            
            // Generate real-time completions (§13.2)
            let suggestions = state.shell_api.generate_completions(partial, partial.len(), &state);
            let (sector_idx, hub_idx) = {
                let viewport = &state.viewports[state.active_viewport_index];
                (viewport.sector_index, viewport.hub_index)
            };
            state.sectors[sector_idx].hubs[hub_idx].suggestions = suggestions;
        } else if request.starts_with("stage_command:") {
            let cmd = &request[14..];
            state.stage_command(cmd.to_string());
        } else if request.starts_with("focus_app:") {
            if let Ok(id) = Uuid::parse_str(&request[10..]) {
                state.focus_app_by_id(id);
            }
        } else if request == "add_remote_sector" {
            self.handle_add_remote_sector(&mut state);
        } else if request == "toggle_bezel" {
            state.toggle_bezel();
        } else if request == "toggle_portal" {
            state.toggle_portal();
        } else if request == "approve_portal" {
            state.approve_portal();
        } else if request == "deny_portal" {
            state.deny_portal();
        } else if request == "split_viewport" {
            state.handle_semantic_event(SemanticEvent::SplitViewport);
            self.spawn_missing_ptys(&mut state);
        } else if request == "create_sector" {
            state.create_new_sector();
        } else if request == "zoom_in" {
            state.handle_semantic_event(SemanticEvent::ZoomIn);
        } else if request == "zoom_out" {
            state.handle_semantic_event(SemanticEvent::ZoomOut);
        } else if request.starts_with("zoom_to:") {
            let target = &request[8..];
            let api = state.active_viewport_index;
            match target {
                "GlobalOverview" => state.handle_semantic_event(SemanticEvent::OpenGlobalOverview),
                "CommandHub" => {
                    state.current_level = HierarchyLevel::CommandHub;
                    state.viewports[api].current_level = HierarchyLevel::CommandHub;
                }
                "ApplicationFocus" => {
                    state.current_level = HierarchyLevel::ApplicationFocus;
                    state.viewports[api].current_level = HierarchyLevel::ApplicationFocus;
                }
                "DetailInspector" => {
                    state.current_level = HierarchyLevel::DetailInspector;
                    state.viewports[api].current_level = HierarchyLevel::DetailInspector;
                }
                "BufferInspector" => {
                    state.current_level = HierarchyLevel::BufferInspector;
                    state.viewports[api].current_level = HierarchyLevel::BufferInspector;
                }
                _ => tracing::warn!("Unknown zoom target: {}", target),
            }
        } else if request == "toggle_output_mode" {
            let api = state.active_viewport_index;
            let viewport = &state.viewports[api];
            let sector_idx = viewport.sector_index;
            let hub_idx = viewport.hub_index;
            state.sectors[sector_idx].hubs[hub_idx].output_mode_centered = !state.sectors[sector_idx].hubs[hub_idx].output_mode_centered;
        } else if request == "toggle_left_region" {
            let api = state.active_viewport_index;
            let viewport = &state.viewports[api];
            let sector_idx = viewport.sector_index;
            let hub_idx = viewport.hub_index;
            state.sectors[sector_idx].hubs[hub_idx].left_region_visible = !state.sectors[sector_idx].hubs[hub_idx].left_region_visible;
        } else if request == "kill_app" {
            let api = state.active_viewport_index;
            let app_id_str = {
                let viewport = &state.viewports[api];
                viewport.active_app_index.map(|app_idx| {
                    let sector_idx = viewport.sector_index;
                    let hub_idx = viewport.hub_index;
                    state.sectors[sector_idx].hubs[hub_idx].applications[app_idx].id.to_string()
                })
            };
            if let Some(id_str) = app_id_str {
                self.handle_kill_app(&mut state, &id_str);
                state.zoom_out();
            }
        } else if request == "toggle_comms" {
            state.toggle_comms();
        } else if request == "optimize_system" {
            state.performance_alert = false;
            state.fps = 60.0;
            println!("TOS // OPTIMIZING RESOURCES... PRUNING DISTANT SURFACES");
        } else if request == "tactical_reset" {
            state.handle_semantic_event(SemanticEvent::TacticalReset);
        } else if request == "open_settings" {
            println!("TOS // OPENING SECTOR SETTINGS... SYNCING CALIBRATION DATA");
            state.settings_open = true;
        } else if request == "close_settings" {
            state.settings_open = false;
        } else if request.starts_with("follow_participant:") {
            let host_id_str = &request[19..];
            if let Ok(host_id) = Uuid::parse_str(host_id_str) {
                // For mock purposes, find first participant (usually the host/user) to follow the target
                let guest_id = state.sectors[state.viewports[state.active_viewport_index].sector_index].participants[0].id;
                let _ = state.collaboration_manager.start_following(guest_id, host_id);
            }
        } else if request.starts_with("semantic_event:") {
            self.handle_semantic_event(&mut state, &request[15..]);
        } else if request.starts_with("connect_remote:") {
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let sector_id = state.sectors[sector_idx].id;
            
            if state.sandbox_registry.get_level(&sector_id) == Some(crate::containers::sandbox::SandboxLevel::Isolated) {
                println!("TOS // ISOLATION POLICY VIOLATION: Remote connections blocked for this sector.");
                state.play_critical_earcon(crate::system::audio::earcons::EarconEvent::CommandError);
                return;
            }

            let addr = &request[15..];
            self.handle_connect_remote(&mut state, addr);
        } else if request.starts_with("invite_participant:") {
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let sector_id = state.sectors[sector_idx].id;
            
            if state.sandbox_registry.get_level(&sector_id) == Some(crate::containers::sandbox::SandboxLevel::Isolated) {
                println!("TOS // ISOLATION POLICY VIOLATION: External collaboration blocked for this sector.");
                state.play_critical_earcon(crate::system::audio::earcons::EarconEvent::CommandError);
                return;
            }

            let role_str = &request[19..];
            self.handle_invite_participant(&mut state, role_str);
        } else if request.starts_with("save_template:") {
            let name = &request[14..];
            self.handle_save_template(&mut state, name);
        } else if request.starts_with("load_template:") {
            let name = &request[14..];
            self.handle_load_template(&mut state, name);
        } else if request.starts_with("kill_app:") {
            let id_str = &request[9..];
            self.handle_kill_app(&mut state, id_str);
        } else if request.starts_with("kill_sector:") {
            if let Ok(idx) = request[12..].parse::<usize>() {
                if idx < state.sectors.len() {
                    state.sectors.remove(idx);
                }
            }
        } else if request.starts_with("play_audio:") {
            let event_str = &request[11..];
            self.handle_play_audio(&mut state, event_str);
        } else if request.starts_with("send_comms:") {
            let body = &request[11..];
            state.comms_messages.push(crate::CommsMessage {
                from: "USER".to_string(),
                body: body.to_string(),
                timestamp: chrono::Local::now().format("%H:%M").to_string(),
            });
            state.earcon_player.play(crate::system::audio::earcons::EarconEvent::CommandAccepted);
        } else if request.starts_with("signal_app:") {
            let parts: Vec<&str> = request[11..].split(';').collect();
            if parts.len() == 2 {
                let id_str = parts[0];
                let signal_type = parts[1];
                self.handle_signal_app(&mut state, id_str, signal_type);
            }
        } else if request == "terminate_remote_link" {
            self.handle_terminate_link(&mut state);
        } else if request.starts_with("set_stream_quality:") {
            if let Some(val) = request.split(':').nth(1).and_then(|v| v.parse::<u8>().ok()) {
                self.handle_set_stream_quality(&mut state, val);
            }
        } else if request == "collaboration_invite" {
            // Default invite action
            self.handle_invite_participant(&mut state, "Viewer");
        } else if request.starts_with("dir_navigate:") {
            let target = &request[13..];
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            let hub = &mut state.sectors[sector_idx].hubs[hub_idx];

            if target == ".." {
                // Go up one directory
                if let Some(parent) = hub.current_directory.parent() {
                    let new_path = parent.to_path_buf();
                    hub.current_directory = new_path.clone();
                    hub.selected_files.clear();
                    hub.context_menu = None;
                    
                    // Sync with shell
                    let hub_id = hub.id;
                    let path_str = new_path.to_string_lossy().to_string();
                    drop(state);
                    if let Ok(ptys) = self.ptys.lock() {
                        if let Some(pty) = ptys.get(&hub_id) {
                            let cd_cmd = format!("cd {}\n", path_str);
                            pty.write(&cd_cmd);
                            
                            // Request directory listing via Shell API
                            let ls_cmd = format!("LS {}\n", path_str);
                            pty.write(&ls_cmd);
                        }
                    }
                    return;
                }
            } else {
                // Navigate into subdirectory
                let new_path = hub.current_directory.join(target);
                if new_path.is_dir() {
                    hub.current_directory = new_path.clone();
                    hub.selected_files.clear();
                    hub.context_menu = None;

                    // Sync with shell
                    let hub_id = hub.id;
                    let path_str = new_path.to_string_lossy().to_string();
                    drop(state);
                    if let Ok(ptys) = self.ptys.lock() {
                        if let Some(pty) = ptys.get(&hub_id) {
                            let cd_cmd = format!("cd {}\n", path_str);
                            pty.write(&cd_cmd);
                            
                            // Request directory listing via Shell API
                            let ls_cmd = format!("LS {}\n", path_str);
                            pty.write(&ls_cmd);
                        }
                    }
                    return;
                }
            }
        } else if request == "dir_toggle_hidden" {
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            let hub = &mut state.sectors[sector_idx].hubs[hub_idx];
            hub.show_hidden_files = !hub.show_hidden_files;
        } else if request.starts_with("dir_pick_file:") {
            let name = &request[14..].replace("\\'", "'");
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            let hub = &mut state.sectors[sector_idx].hubs[hub_idx];
            
            let full_path = hub.current_directory.join(name).to_string_lossy().to_string();
            if hub.prompt.is_empty() {
                hub.prompt = format!("view {} ", full_path);
            } else {
                if !hub.prompt.ends_with(' ') {
                    hub.prompt.push(' ');
                }
                hub.prompt.push_str(&full_path);
            }
        } else if request.starts_with("dir_toggle_select:") {
            let name = &request[18..].replace("\\'", "'");
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            let hub = &mut state.sectors[sector_idx].hubs[hub_idx];
            if !hub.selected_files.remove(name) {
                hub.selected_files.insert(name.to_string());
            }

            // Sync multi-selection with prompt for batch operations
            if !hub.selected_files.is_empty() {
                let paths: Vec<String> = hub.selected_files.iter()
                    .map(|f| hub.current_directory.join(f).to_string_lossy().to_string())
                    .collect();
                
                let cmd = if hub.prompt.is_empty() { "view" } else {
                    hub.prompt.split_whitespace().next().unwrap_or("view")
                };
                hub.prompt = format!("{} {}", cmd, paths.join(" "));
            }
        } else if request.starts_with("dir_context:") {
            let parts: Vec<&str> = request[12..].split(';').collect();
            if parts.len() >= 3 {
                let target = parts[0].replace("\\'", "'");
                let x = parts[1].parse::<i32>().unwrap_or(0);
                let y = parts[2].parse::<i32>().unwrap_or(0);
                let sector_idx = state.viewports[state.active_viewport_index].sector_index;
                let hub_idx = state.viewports[state.active_viewport_index].hub_index;
                let hub = &mut state.sectors[sector_idx].hubs[hub_idx];
                hub.context_menu = Some(crate::ContextMenu {
                    target: target.to_string(),
                    x, y,
                    actions: vec!["OPEN".to_string(), "COPY".to_string(), "RENAME".to_string(), "DELETE".to_string()],
                });
            }
        } else if request == "dir_close_context" {
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            state.sectors[sector_idx].hubs[hub_idx].context_menu = None;
        } else if request == "dir_clear_select" {
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            let hub = &mut state.sectors[sector_idx].hubs[hub_idx];
            hub.selected_files.clear();
            hub.context_menu = None;
        } else if request.starts_with("app_toggle_select:") {
            let id_str = &request[18..];
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            let sector = &mut state.sectors[sector_idx];
            let hub = &mut sector.hubs[hub_idx];
            
            if !hub.selected_files.remove(id_str) {
                hub.selected_files.insert(id_str.to_string());
            }

            // Sync multi-selection with prompt for batch operations
            if !hub.selected_files.is_empty() {
                let mut pids = Vec::new();
                for app_id_str in &hub.selected_files {
                    if let Ok(uuid) = uuid::Uuid::parse_str(app_id_str) {
                        if let Some(app) = hub.applications.iter().find(|a| a.id == uuid) {
                            if let Some(pid) = app.pid {
                                pids.push(pid.to_string());
                            } else {
                                // Fallback for dummy apps or non-started
                                pids.push(format!("[{}]", app.title));
                            }
                        }
                    }
                }
                
                let current_cmd = hub.prompt.split_whitespace().next().unwrap_or("");
                let cmd = if current_cmd.is_empty() || current_cmd == "view" { "manage" } else { current_cmd };
                
                hub.prompt = format!("{} {}", cmd, pids.join(" "));
            } else {
                hub.prompt = String::new();
            }
        } else if request == "app_batch_kill" {
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            let selected: Vec<String> = state.sectors[sector_idx].hubs[hub_idx].selected_files.iter().cloned().collect();
            
            for id in selected {
                self.handle_kill_app(&mut state, &id);
            }
            
            // Clear selection
            let sector = &mut state.sectors[sector_idx];
            let hub = &mut sector.hubs[hub_idx];
            hub.selected_files.clear();
            hub.prompt.clear();
        } else if request.starts_with("app_batch_signal:") {
            let signal = &request[17..];
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            let selected: Vec<String> = state.sectors[sector_idx].hubs[hub_idx].selected_files.iter().cloned().collect();

            for id in selected {
                self.handle_signal_app(&mut state, &id, signal);
            }
            
            // Clear selection
            let sector = &mut state.sectors[sector_idx];
            let hub = &mut sector.hubs[hub_idx];
            hub.selected_files.clear();
            hub.prompt.clear();
        } else if request == "dir_action_copy" {
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            let hub = &mut state.sectors[sector_idx].hubs[hub_idx];
            if !hub.selected_files.is_empty() {
                let files: Vec<String> = hub.selected_files.iter().cloned().collect();
                hub.prompt = format!("cp {} ", files.join(" "));
            }
        } else if request == "dir_action_paste" {
             let sector_idx = state.viewports[state.active_viewport_index].sector_index;
             let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            let hub = &mut state.sectors[sector_idx].hubs[hub_idx];
            // Placeholder for clipboard
            hub.prompt = format!("cp $CLIPBOARD .");
        } else if request == "dir_action_delete" {
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            let hub = &mut state.sectors[sector_idx].hubs[hub_idx];
             if !hub.selected_files.is_empty() {
                let files: Vec<String> = hub.selected_files.iter().cloned().collect();
                hub.prompt = format!("rm {} ", files.join(" "));
            }
        } else if request.starts_with("update_confirmation_progress:") {
            let parts: Vec<&str> = request[29..].split(':').collect();
            if parts.len() == 2 {
                if let (Ok(id), Ok(progress)) = (Uuid::parse_str(parts[0]), parts[1].parse::<f32>()) {
                    if let Some(complete) = state.security.update_progress(id, progress) {
                        if complete {
                            self.execute_confirmed_command(&mut state, id);
                        }
                    }
                }
            }
            return;
        } else if request.starts_with("confirm_command:") {
            if let Ok(id) = Uuid::parse_str(&request[16..]) {
                self.execute_confirmed_command(&mut state, id);
            }
            return;
        } else if request.starts_with("cancel_confirmation:") {
            if let Ok(id) = Uuid::parse_str(&request[20..]) {
                state.security.active_sessions.remove(&id);
            }
        } else if request.starts_with("increment_hold:") {
            if let Ok(id) = Uuid::parse_str(&request[15..]) {
                if let Some(session) = state.security.active_sessions.get_mut(&id) {
                    session.progress += 0.05; // 5% per 100ms
                }
                if let Some(session) = state.security.active_sessions.get(&id) {
                    if session.progress >= 1.0 {
                        self.execute_confirmed_command(&mut state, id);
                    }
                }
            }
            return;
        } else if request.starts_with("reset_hold:") {
            if let Ok(id) = Uuid::parse_str(&request[11..]) {
                if let Some(session) = state.security.active_sessions.get_mut(&id) {
                    session.progress = 0.0;
                }
            }
            return;
        } else if request.starts_with("update_setting:") {
            let parts: Vec<&str> = request[15..].split(':').collect();
            if parts.len() >= 2 {
                let key = parts[0];
                let val = parts[1].parse::<f32>().unwrap_or(0.0);
                
                let sector_idx = state.viewports[state.active_viewport_index].sector_index;
                let hub_idx = state.viewports[state.active_viewport_index].hub_index;
                let app_idx = state.viewports[state.active_viewport_index].active_app_index;
                
                if let Some(idx) = app_idx {
                    let app = &mut state.sectors[sector_idx].hubs[hub_idx].applications[idx];
                    app.settings.insert(key.to_string(), val);
                }
            }
        } else if request.starts_with("marketplace_search:") {
            let query = request[19..].to_string();
            let state_arc = self.state.clone();
            
            tokio::spawn(async move {
                let marketplace = {
                    let state = state_arc.lock().expect("Lock failed");
                    state.marketplace.clone()
                };
                let results = marketplace.search(&query).await;
                
                match results {
                    Ok(packages) => {
                        let mut state = state_arc.lock().expect("Lock failed");
                        let (sector_index, hub_index) = {
                            let viewport = &state.viewports[state.active_viewport_index];
                            (viewport.sector_index, viewport.hub_index)
                        };
                        let hub = &mut state.sectors[sector_index].hubs[hub_index];
                        hub.terminal_output.push(format!("MARKETPLACE // SEARCH RESULTS FOR '{}':", query));
                        for pkg in packages {
                            hub.terminal_output.push(format!(" - {} v{} ({})", pkg.name, pkg.version, pkg.package_type));
                        }
                    }
                    Err(e) => tracing::error!("Marketplace search failed: {}", e),
                }
            });
        } else if request.starts_with("marketplace_install:") {
            let name = request[20..].to_string();
            let state_arc = self.state.clone();
            
            tokio::spawn(async move {
                let request = crate::marketplace::InstallRequest {
                    package_name: name.clone(),
                    version_constraint: "latest".to_string(),
                    repository: None,
                    auto_accept: true,
                    skip_signature_check: false,
                };
                
                let marketplace = {
                    let state = state_arc.lock().expect("Lock failed");
                    state.marketplace.clone()
                };
                let results = marketplace.install(request).await;
                
                let mut state = state_arc.lock().expect("Lock failed");
                let (sector_index, hub_index) = {
                    let viewport = &state.viewports[state.active_viewport_index];
                    (viewport.sector_index, viewport.hub_index)
                };
                let hub = &mut state.sectors[sector_index].hubs[hub_index];
                
                match results {
                    Ok(res) => {
                        hub.terminal_output.push(format!("MARKETPLACE // INSTALLED: {} v{}", res.package.name, res.package.version));
                        hub.terminal_output.push(format!("PATH // {}", res.install_path.display()));
                        state.earcon_player.play(crate::system::audio::earcons::EarconEvent::CommandAccepted);
                    }
                    Err(e) => {
                        hub.terminal_output.push(format!("MARKETPLACE // ERROR: {}", e));
                        state.earcon_player.play(crate::system::audio::earcons::EarconEvent::TacticalAlert);
                    }
                }
            });
        } else if request.starts_with("marketplace_add_repo:") {
            let parts: Vec<&str> = request[21..].split('|').collect();
            if parts.len() >= 2 {
                let name = parts[0];
                let url = parts[1];
                state.marketplace.add_repository(crate::marketplace::RepositoryConfig {
                    name: name.to_string(),
                    url: url.to_string(),
                    enabled: true,
                    priority: 5,
                    auth_token: None,
                });
                let (sector_index, hub_index) = {
                    let viewport = &state.viewports[state.active_viewport_index];
                    (viewport.sector_index, viewport.hub_index)
                };
                state.sectors[sector_index].hubs[hub_index].terminal_output.push(
                    format!("MARKETPLACE // REPOSITORY ADDED: {} ({})", name, url)
                );
            }
        } else if request == "marketplace_discover" {
            let state_arc = self.state.clone();
            
            tokio::spawn(async move {
                let marketplace = {
                    let state = state_arc.lock().expect("Lock failed");
                    state.marketplace.clone()
                };
                let results = marketplace.discover_repositories().await;
                
                let mut state = state_arc.lock().expect("Lock failed");
                let (sector_index, hub_index) = {
                    let viewport = &state.viewports[state.active_viewport_index];
                    (viewport.sector_index, viewport.hub_index)
                };
                let hub = &mut state.sectors[sector_index].hubs[hub_index];
                
                match results {
                    Ok(repos) => {
                        hub.terminal_output.push("MARKETPLACE // DISCOVERED REPOSITORIES:".to_string());
                        for repo in repos {
                            hub.terminal_output.push(format!(" - {} ({})", repo.name, repo.url));
                        }
                        state.earcon_player.play(crate::system::audio::earcons::EarconEvent::CommandAccepted);
                    }
                    Err(e) => {
                        hub.terminal_output.push(format!("MARKETPLACE // DISCOVERY ERROR: {}", e));
                        state.earcon_player.play(crate::system::audio::earcons::EarconEvent::TacticalAlert);
                    }
                }
            });
        } else if request == "ui_ready" {
            state.force_redraw = true;
        } else {
            // Legacy/Direct zoom fallback
            match request {
                "zoom_in" => state.zoom_in(),
                "zoom_out" => state.zoom_out(),
                _ => println!("Unknown IPC request: {}", request),
            }
        }
        
        // Auto-save on every state modification
        state.save();
    }

    fn handle_prompt_submit(&self, state: &mut TosState, cmd_full: &str) {
        println!("Prompt Submitted: {}", cmd_full);

        // Handle Search and AI modes
        let viewport = &state.viewports[state.active_viewport_index];
        let sector_idx = viewport.sector_index;
        let hub_idx = viewport.hub_index;
        let mode = state.sectors[sector_idx].hubs[hub_idx].mode;

        if mode == CommandHubMode::Search {
             state.perform_search(cmd_full);
             return;
        } else if mode == CommandHubMode::Ai {
             // In AI mode, prompt is usually updated live, but if submitted via IPC ensure it's set
             state.sectors[sector_idx].hubs[hub_idx].prompt = cmd_full.to_string();
             state.handle_semantic_event(SemanticEvent::AiSubmit);
             return;
        }

        let parts: Vec<&str> = cmd_full.split_whitespace().collect();
        if parts.is_empty() { return; }

        match parts[0] {
            "zoom" => {
                if parts.get(1) == Some(&"in") { state.zoom_in(); }
                else if parts.get(1) == Some(&"out") { state.zoom_out(); }
            }
            "enable-deep-inspection" => {
                if state.security.config.allow_deep_inspection {
                    if state.security.enable_deep_inspection("host") {
                        println!("TOS // DEEP INSPECTION ENABLED");
                        state.earcon_player.play(crate::system::audio::earcons::EarconEvent::CommandAccepted);
                    }
                } else {
                    println!("TOS // DEEP INSPECTION DISABLED BY POLICY");
                    state.play_critical_earcon(crate::system::audio::earcons::EarconEvent::CommandError);
                }
            }
            "disable-deep-inspection" => {
                state.security.disable_deep_inspection("host");
                println!("TOS // DEEP INSPECTION DISABLED");
                state.earcon_player.play(crate::system::audio::earcons::EarconEvent::CommandAccepted);
            }
            "in" => state.zoom_in(),
            "out" => state.zoom_out(),
            "mode" => {
                match parts.get(1) {
                    Some(&"command") => state.toggle_mode(CommandHubMode::Command),
                    Some(&"directory") | Some(&"dir") => state.toggle_mode(CommandHubMode::Directory),
                    Some(&"activity") | Some(&"apps") => state.toggle_mode(CommandHubMode::Activity),
                    Some(&"search") => state.toggle_mode(CommandHubMode::Search),
                    Some(&"ai") => state.toggle_mode(CommandHubMode::Ai),
                    _ => {}
                }
            }
            "focus" => {
                if let Some(target) = parts.get(1) {
                    let viewport = &state.viewports[state.active_viewport_index];
                    let sector = &mut state.sectors[viewport.sector_index];
                    let hub = &mut sector.hubs[viewport.hub_index];
                    if let Some(pos) = hub.applications.iter().position(|a| a.title.to_uppercase() == target.to_uppercase()) {
                        hub.active_app_index = Some(pos);
                        state.current_level = HierarchyLevel::ApplicationFocus;
                        state.viewports[state.active_viewport_index].current_level = HierarchyLevel::ApplicationFocus;
                        state.viewports[state.active_viewport_index].active_app_index = Some(pos);
                    }
                }
            }
            _ => {
                let viewport = &state.viewports[state.active_viewport_index];
                let sector = &mut state.sectors[viewport.sector_index];
                let hub = &mut sector.hubs[viewport.hub_index];

                // Use SecurityManager for dangerous command detection
                if let Some((risk, pattern)) = state.security.check_command(cmd_full) {
                    if hub.confirmation_required.is_none() {
                        println!("!! DANGEROUS COMMAND DETECTED ({}): {}", pattern.name, cmd_full);
                        hub.confirmation_required = Some(format!("{} (Risk: {:?})", pattern.message, risk));
                        
                        // Start a confirmation session
                        state.security.start_confirmation(cmd_full, "host", sector.id);
                        return;
                    }
                }

                hub.confirmation_required = None;
                // PTY write is now handled in handle_request after dropping state lock
                tracing::info!("Consolidated tactical command: {}", cmd_full);
            }
        }
    }

    fn handle_add_remote_sector(&self, state: &mut TosState) {
        // Use RemoteManager to register and connect to a mock node
        let node_id = Uuid::new_v4();
        let info = RemoteNodeInfo {
            id: node_id,
            hostname: "REMOTE-NODE-01".to_string(),
            address: "10.0.4.15".to_string(),
            os_type: "TOS".to_string(),
            version: "1.0.0".to_string(),
            status: RemoteStatus::Online,
        };

        state.remote_manager.register_node(info);
        if let Ok(_) = state.remote_manager.connect(node_id, ConnectionType::SSH) {
            if let Some(sector) = state.remote_manager.create_remote_sector(node_id) {
                let hub_id = sector.hubs[0].id;
                let host = sector.host.clone();
                state.add_sector(sector);

                // Spawn a shell for the remote (Real SSH)
                if let Some(_ssh) = state.shell_registry.get("ssh") {
                    // We need a way to pass the host to the ssh provider.
                    // For now, we'll use a hack or update the trait.
                    // Actually, if we have the SshShellProvider, we can try to cast it
                    // or just use a generic command if we update the trait.
                    
                    // Simple approach: look for 'ssh' in registry and use it.
                    // Since we can't easily downcast Box<dyn ShellProvider>, 
                    // let's just use PtyHandle directly here if it's SSH.
                    
                    let pty = if host != "LOCAL" {
                        crate::system::pty::PtyHandle::spawn_with_args("/usr/bin/ssh", &["-t", &host], ".")
                    } else {
                        state.shell_registry.get("fish").and_then(|f| f.spawn("."))
                    };

                    if let Some(pty) = pty {
                        let pid = pty.child_pid;
                        self.ptys.lock().unwrap().insert(hub_id, pty);
                        
                        // Register shell as an application for monitoring
                        // Find the sector we just added
                        if let Some(s) = state.sectors.iter_mut().find(|s| s.hubs.iter().any(|h| h.id == hub_id)) {
                            if let Some(h) = s.hubs.iter_mut().find(|h| h.id == hub_id) {
                                h.applications.push(crate::Application {
                                    id: Uuid::new_v4(),
                                    title: "ssh".to_string(),
                                    app_class: "Shell".to_string(),
                                    is_minimized: false,
                                    pid: Some(pid),
                                    icon: Some("📡".to_string()),
                                    is_dummy: false,
                                    settings: std::collections::HashMap::new(),
                                    thumbnail: None,
                                    decoration_policy: crate::DecorationPolicy::Native,
                                    bezel_actions: std::vec::Vec::new(),
                                });
                                h.active_app_index = Some(0);
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_connect_remote(&self, state: &mut TosState, address: &str) {
        // Attempt to link to a specific address
        let node_id = Uuid::new_v4();
        let info = RemoteNodeInfo {
            id: node_id,
            hostname: format!("REMOTE-{}", address),
            address: address.to_string(),
            os_type: "TOS".to_string(),
            version: "1.0.0".to_string(),
            status: RemoteStatus::Online,
        };

        state.remote_manager.register_node(info);
        if let Ok(_) = state.remote_manager.connect(node_id, ConnectionType::TOSNative) {
            if let Some(sector) = state.remote_manager.create_remote_sector(node_id) {
                state.add_sector(sector);
                println!("TOS // ESTABLISHED NATIVE LINK TO {}", address);
            }
        }
    }

    fn handle_invite_participant(&self, state: &mut TosState, role_str: &str) {
        let role = match role_str {
            "CoOwner" => CollaborationRole::CoOwner,
            "Operator" => CollaborationRole::Operator,
            "Viewer" => CollaborationRole::Viewer,
            _ => CollaborationRole::Viewer,
        };

        let sector_idx = state.viewports[state.active_viewport_index].sector_index;
        let sector_id = state.sectors[sector_idx].id;

        let token = state.collaboration_manager.create_invitation(sector_id, role);
        println!("TOS // COLLABORATION INVITE GENERATED: {}", token);
        println!("TOS // INVITATION CREATED FOR {:?}: {}", role, token);
        
        // Mock: Automatically add the participant for demo purposes
        let p_id = Uuid::new_v4();
        state.sectors[sector_idx].participants.push(Participant {
            id: p_id,
            name: format!("Guest-{}", &token[..4]),
            color: "#00ffcc".to_string(),
            avatar_url: None,
            role: role,
            cursor_position: None,
            following_host_id: None,
        });
        
        state.collaboration_manager.sessions.insert(p_id, PermissionSet::for_role(role));
    }

    fn handle_terminate_link(&self, state: &mut TosState) {
        let sector_index = state.viewports[state.active_viewport_index].sector_index;
        let sector_id = state.sectors[sector_index].id;
        
        // Remove active link
        state.remote_manager.disconnect(sector_id);
        
        // Remove sector if remote
        if state.sectors[sector_index].connection_type != ConnectionType::Local {
            state.sectors.remove(sector_index);
            state.viewports.retain(|v| v.sector_index != sector_index);
            // Re-index remaining viewports
            for v in &mut state.viewports {
                if v.sector_index > sector_index {
                    v.sector_index -= 1;
                }
            }
            state.active_viewport_index = 0;
            state.current_level = crate::HierarchyLevel::GlobalOverview;
        }
    }

    fn handle_set_stream_quality(&self, state: &mut TosState, quality: u8) {
        let sector_index = state.viewports[state.active_viewport_index].sector_index;
        let sector_id = state.sectors[sector_index].id;
        
        if let Some(conn) = state.remote_manager.active_connections.get_mut(&sector_id) {
            conn.stream_quality = quality;
            println!("TOS // REMOTE STREAM QUALITY SET TO {}%", quality);
            state.earcon_player.play(crate::system::audio::earcons::EarconEvent::CommandAccepted);
        }
    }

    fn handle_save_template(&self, state: &mut TosState, name: &str) {
        let viewport = &state.viewports[state.active_viewport_index];
        let sector = &state.sectors[viewport.sector_index];
        
        println!("TOS // EXPORTING SECTOR {} AS TEMPLATE: {}", sector.name, name);
        // Mocking the export result for the UI
        state.sectors[viewport.sector_index].hubs[viewport.hub_index].terminal_output.push(
            format!("SYSTEM // TEMPLATE EXPORTED: {}.tos-template", name)
        );
    }

    fn handle_load_template(&self, state: &mut TosState, name: &str) {
        println!("TOS // LOADING TEMPLATE: {}", name);
        // Mocking the load result
        state.sectors[state.viewports[state.active_viewport_index].sector_index].name = format!("TEMPLATE: {}", name);
    }

    fn execute_confirmed_command(&self, state: &mut TosState, session_id: Uuid) {
        if let Some(session) = state.security.active_sessions.remove(&session_id) {
            let cmd = session.command;
            let hub_id = {
                let viewport = &state.viewports[state.active_viewport_index];
                state.sectors[viewport.sector_index].hubs[viewport.hub_index].id
            };
            
            println!("TOS // COMMAND CONFIRMED: {}", cmd);
            state.earcon_player.play(crate::system::audio::earcons::EarconEvent::CommandAccepted);
            
            if let Ok(ptys) = self.ptys.lock() {
                if let Some(pty) = ptys.get(&hub_id) {
                    pty.write(&format!("{}\n", cmd));
                }
            }
        }
    }

    fn handle_kill_app(&self, state: &mut TosState, id_str: &str) {
        self.handle_signal_app(state, id_str, "KILL");
    }

    fn handle_signal_app(&self, state: &mut TosState, id_str: &str, signal_type: &str) {
        if let Ok(id) = Uuid::parse_str(id_str) {
            for sector in &mut state.sectors {
                for hub in &mut sector.hubs {
                    if let Some(pos) = hub.applications.iter().position(|a| a.id == id) {
                        let app = &hub.applications[pos];
                        if let Some(pid) = app.pid {
                            let sig = match signal_type {
                                "INT" => libc::SIGINT,
                                "TERM" => libc::SIGTERM,
                                "KILL" => libc::SIGKILL,
                                _ => libc::SIGTERM,
                            };
                            crate::system::proc::send_signal(pid, sig);
                            
                            if signal_type == "KILL" {
                                hub.applications.remove(pos);
                            }
                        } else {
                            // Dummy app just gets removed
                            if signal_type == "KILL" {
                                hub.applications.remove(pos);
                            }
                        }
                        return;
                    }
                }
            }
        }
    }

    fn handle_play_audio(&self, state: &mut TosState, event_str: &str) {
        use crate::system::audio::AudioEvent;
        let event = match event_str {
            "AmbientHum" => AudioEvent::AmbientHum,
            "AlertBeep" => AudioEvent::AlertBeep,
            "SectorTransition" => AudioEvent::SectorTransition,
            "DataTransfer" => AudioEvent::DataTransfer,
            _ => AudioEvent::AlertBeep,
        };
        state.audio_manager.play_event(event);
    }

    fn spawn_missing_ptys(&self, state: &mut TosState) {
        let mut ptys = self.ptys.lock().expect("Failed to lock ptys");
        let shell_name = "fish"; // Default
        
        for sector in &state.sectors {
            for hub in &sector.hubs {
                if !ptys.contains_key(&hub.id) {
                    if let Some(shell) = state.shell_registry.get(shell_name) {
                        if let Some(pty) = shell.spawn(hub.current_directory.to_str().unwrap_or(".")) {
                            let _pid = pty.child_pid;
                            ptys.insert(hub.id, pty);
                            
                            // Register shell as an application for monitoring
                            // We need to re-find the hub since we can't easily mutably borrow sector here while holding state
                            // Actually we already have &mut state, but we are in a loop.
                        }
                    }
                }
            }
        }
    }

    fn handle_semantic_event(&self, state: &mut TosState, event_name: &str) {
        match event_name {
            "ZoomIn" => state.handle_semantic_event(SemanticEvent::ZoomIn),
            "ZoomOut" => state.handle_semantic_event(SemanticEvent::ZoomOut),
            "CycleMode" => state.handle_semantic_event(SemanticEvent::CycleMode),
            "ToggleBezel" => state.handle_semantic_event(SemanticEvent::ToggleBezel),
            "TacticalReset" => state.handle_semantic_event(SemanticEvent::TacticalReset),
            "SystemReset" => state.handle_semantic_event(SemanticEvent::SystemReset),
            "OpenGlobalOverview" => state.handle_semantic_event(SemanticEvent::OpenGlobalOverview),
            "SplitViewport" => {
                state.handle_semantic_event(SemanticEvent::SplitViewport);
                self.spawn_missing_ptys(state);
            }
            "CloseViewport" => state.handle_semantic_event(SemanticEvent::CloseViewport),
            "StopOperation" => state.handle_semantic_event(SemanticEvent::StopOperation),
            "ToggleMiniMap" => state.handle_semantic_event(SemanticEvent::ToggleMiniMap),
            "ToggleComms" => state.handle_semantic_event(SemanticEvent::ToggleComms),
            "VoiceCommandStart" => {
                tracing::info!("VOICE COMMAND INITIATED");
                state.stage_command("LISTENING...".to_string());
            }
            _ => tracing::warn!("Unknown semantic event from IPC: {}", event_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TosState, CommandHub, CommandHubMode, Sector, Viewport, HierarchyLevel, ConnectionType};
    use uuid::Uuid;
    use std::collections::{HashSet, HashMap};
    use std::sync::{Arc, Mutex};

    // Helper to create a minimal state for testing
    fn create_test_state() -> TosState {
        let hub = CommandHub {
            id: Uuid::new_v4(),
            mode: CommandHubMode::Directory,
            prompt: String::new(),
            applications: Vec::new(),
            active_app_index: None,
            terminal_output: Vec::new(),
            confirmation_required: None,
            current_directory: std::path::PathBuf::from("/home/test"),
            show_hidden_files: false,
            selected_files: HashSet::new(),
            context_menu: None,
            shell_listing: None,
            suggestions: vec![],
            output_mode_centered: false,
            left_region_visible: true,
        };

        let sector = Sector {
            id: Uuid::new_v4(),
            name: "TEST_SECTOR".to_string(),
            hubs: vec![hub],
            active_hub_index: 0, 
            color: "blue".to_string(),
            settings: std::collections::HashMap::new(),
            host: "localhost".to_string(),
            connection_type: ConnectionType::Local,
            participants: Vec::new(),
            portal_active: false,
            portal_url: None,
            description: "Test".to_string(),
            icon: "T".to_string(),
            sector_type_name: "development".to_string(),
        };

        let viewport = Viewport {
            id: Uuid::new_v4(),
            sector_index: 0,
            hub_index: 0,
            current_level: HierarchyLevel::CommandHub,
            active_app_index: None,
            bezel_expanded: false,
        };

        let mut state = TosState::new();
        state.sectors = vec![sector];
        state.viewports = vec![viewport];
        state.active_viewport_index = 0;
        state
    }

    #[test]
    fn test_dir_pick_file_appends() {
        let mut state = create_test_state();
        state.sectors[0].hubs[0].current_directory = std::path::PathBuf::from("/home/test");
        state.sectors[0].hubs[0].prompt = "view".to_string(); // Initial prompt
        
        let state_arc = Arc::new(Mutex::new(state));
        let ptys = Arc::new(Mutex::new(HashMap::new()));
        let dispatcher = IpcDispatcher::new(state_arc.clone(), ptys);

        // Action: Pick file 'file.txt'
        dispatcher.handle_request("dir_pick_file:file.txt");

        // Assert: Prompt should be 'view /home/test/file.txt'
        {
            let state = state_arc.lock().unwrap();
            assert_eq!(state.sectors[0].hubs[0].prompt, "view /home/test/file.txt");
        }

        // Action: Pick another file 'file2.log'
        dispatcher.handle_request("dir_pick_file:file2.log");

        // Assert: Prompt should be 'view /home/test/file.txt /home/test/file2.log'
        {
            let state = state_arc.lock().unwrap();
            assert_eq!(state.sectors[0].hubs[0].prompt, "view /home/test/file.txt /home/test/file2.log");
        }
    }

    #[test]
    fn test_dir_action_copy_stages_command() {
        let mut state = create_test_state();
        state.sectors[0].hubs[0].current_directory = std::path::PathBuf::from("/home/test");
        state.sectors[0].hubs[0].selected_files.insert("file1.txt".to_string());
        state.sectors[0].hubs[0].selected_files.insert("file2.txt".to_string());
        
        let state_arc = Arc::new(Mutex::new(state));
        let ptys = Arc::new(Mutex::new(HashMap::new()));
        let dispatcher = IpcDispatcher::new(state_arc.clone(), ptys);

        // Action: Copy
        dispatcher.handle_request("dir_action_copy");

        // Assert: Prompt should contain 'cp' and both files
        {
            let state = state_arc.lock().unwrap();
            let prompt = &state.sectors[0].hubs[0].prompt;
            assert!(prompt.starts_with("cp "));
            assert!(prompt.contains("file1.txt"));
            assert!(prompt.contains("file2.txt"));
        }
    }
}
