use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use crate::system::pty::PtyHandle;
use crate::system::remote::{RemoteNodeInfo, RemoteStatus};
use crate::system::collaboration::{CollaborationRole, PermissionSet};
use crate::{TosState, CommandHubMode, HierarchyLevel, system::input::SemanticEvent, CommandHub, Viewport, ConnectionType, Participant};

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
                _ => return,
            };
            state.toggle_mode(mode);
        } else if request.starts_with("select_sector:") {
            if let Ok(idx) = request[14..].parse::<usize>() {
                state.select_sector(idx);
            }
        } else if request.starts_with("prompt_submit:") {
            self.handle_prompt_submit(&mut state, &request[14..]);
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
            self.handle_split_viewport(&mut state);
        } else if request == "zoom_in" {
            state.handle_semantic_event(SemanticEvent::ZoomIn);
        } else if request == "zoom_out" {
            state.handle_semantic_event(SemanticEvent::ZoomOut);
        } else if request == "optimize_system" {
            state.performance_alert = false;
            state.fps = 60.0;
            println!("TOS // OPTIMIZING RESOURCES... PRUNING DISTANT SURFACES");
        } else if request == "tactical_reset" {
            state.handle_semantic_event(SemanticEvent::TacticalReset);
        } else if request.starts_with("semantic_event:") {
            self.handle_semantic_event(&mut state, &request[15..]);
        } else if request.starts_with("connect_remote:") {
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let sector_id = state.sectors[sector_idx].id;
            
            if state.sandbox_registry.get_level(&sector_id) == Some(crate::containers::sandbox::SandboxLevel::Isolated) {
                println!("TOS // ISOLATION POLICY VIOLATION: Remote connections blocked for this sector.");
                state.earcon_player.play(crate::system::audio::earcons::EarconEvent::CommandError);
                return;
            }

            let addr = &request[15..];
            self.handle_connect_remote(&mut state, addr);
        } else if request.starts_with("invite_participant:") {
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let sector_id = state.sectors[sector_idx].id;
            
            if state.sandbox_registry.get_level(&sector_id) == Some(crate::containers::sandbox::SandboxLevel::Isolated) {
                println!("TOS // ISOLATION POLICY VIOLATION: External collaboration blocked for this sector.");
                state.earcon_player.play(crate::system::audio::earcons::EarconEvent::CommandError);
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
                    if let Ok(ptys) = self.ptys.lock() {
                        if let Some(pty) = ptys.get(&hub_id) {
                            let cd_cmd = format!("cd {}\n", new_path.to_string_lossy());
                            pty.write(&cd_cmd);
                            
                            // Request directory listing via Shell API
                            let ls_cmd = format!("LS {}\n", new_path.to_string_lossy());
                            pty.write(&ls_cmd);
                        }
                    }
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
                    if let Ok(ptys) = self.ptys.lock() {
                        if let Some(pty) = ptys.get(&hub_id) {
                            let cd_cmd = format!("cd {}\n", new_path.to_string_lossy());
                            pty.write(&cd_cmd);
                            
                            // Request directory listing via Shell API
                            let ls_cmd = format!("LS {}\n", new_path.to_string_lossy());
                            pty.write(&ls_cmd);
                        }
                    }
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
        } else {
            // Legacy/Direct zoom fallback
            match request {
                "zoom_in" => state.zoom_in(),
                "zoom_out" => state.zoom_out(),
                _ => tracing::warn!("Unknown IPC request: {}", request),
            }
        }
        
        // Auto-save on every state modification
        state.save();
    }

    fn handle_prompt_submit(&self, state: &mut TosState, cmd_full: &str) {
        println!("Prompt Submitted: {}", cmd_full);
        let parts: Vec<&str> = cmd_full.split_whitespace().collect();
        if parts.is_empty() { return; }

        match parts[0] {
            "zoom" => {
                if parts.get(1) == Some(&"in") { state.zoom_in(); }
                else if parts.get(1) == Some(&"out") { state.zoom_out(); }
            }
            "in" => state.zoom_in(),
            "out" => state.zoom_out(),
            "mode" => {
                match parts.get(1) {
                    Some(&"command") => state.toggle_mode(CommandHubMode::Command),
                    Some(&"directory") | Some(&"dir") => state.toggle_mode(CommandHubMode::Directory),
                    Some(&"activity") | Some(&"apps") => state.toggle_mode(CommandHubMode::Activity),
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
                if let Some(pty) = self.ptys.lock().unwrap().get(&hub.id) {
                    pty.write(&format!("{}\n", cmd_full));
                }
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
        println!("TOS // INVITATION CREATED FOR {:?}: {}", role, token);
        
        // Mock: Automatically add the participant for demo purposes
        let p_id = Uuid::new_v4();
        state.sectors[sector_idx].participants.push(Participant {
            name: format!("Guest-{}", &token[..4]),
            color: "#00ffcc".to_string(),
            role: role.as_str().to_string(),
        });
        
        state.collaboration_manager.sessions.insert(p_id, PermissionSet::for_role(role));
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

    fn handle_split_viewport(&self, state: &mut TosState) {
        let sector_idx = state.viewports[state.active_viewport_index].sector_index;
        let new_hub_id = Uuid::new_v4();
        
        let sector = &mut state.sectors[sector_idx];
        sector.hubs.push(CommandHub {
            id: new_hub_id,
            mode: CommandHubMode::Command,
            prompt: String::new(),
            applications: Vec::new(),
            active_app_index: None,
            terminal_output: Vec::new(),
            confirmation_required: None,
            current_directory: dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/")),
            show_hidden_files: false,
            selected_files: std::collections::HashSet::new(),
            context_menu: None,
            shell_listing: None,
        });
        
        let hub_idx = sector.hubs.len() - 1;

        if let Some(fish) = state.shell_registry.get("fish") {
            if let Some(pty) = fish.spawn(".") {
                let pid = pty.child_pid;
                self.ptys.lock().unwrap().insert(new_hub_id, pty);
                
                // Register shell as an application for monitoring
                sector.hubs[hub_idx].applications.push(crate::Application {
                    id: Uuid::new_v4(),
                    title: "fish".to_string(),
                    app_class: "Shell".to_string(),
                    is_minimized: false,
                    pid: Some(pid),
                    icon: Some("⌨️".to_string()),
                    is_dummy: false,
                    settings: std::collections::HashMap::new(),
                });
                sector.hubs[hub_idx].active_app_index = Some(0);
            }
        }

        state.viewports.push(Viewport {
            id: Uuid::new_v4(),
            sector_index: sector_idx,
            hub_index: hub_idx,
            current_level: HierarchyLevel::CommandHub,
            active_app_index: None,
            bezel_expanded: false,
        });
        state.current_level = HierarchyLevel::SplitView;
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
        };

        let sector = Sector {
            id: Uuid::new_v4(),
            name: "TEST_SECTOR".to_string(),
            hubs: vec![hub],
            active_hub_index: 0, 
            color: "blue".to_string(),
            host: "localhost".to_string(),
            connection_type: ConnectionType::Local,
            participants: Vec::new(),
            portal_active: false,
            portal_url: None,
            description: "Test".to_string(),
            icon: "T".to_string(),
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
