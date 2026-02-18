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
            let addr = &request[15..];
            self.handle_connect_remote(&mut state, addr);
        } else if request.starts_with("invite_participant:") {
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
                        }
                    }
                }
            }
        } else if request == "dir_toggle_hidden" {
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            let hub = &mut state.sectors[sector_idx].hubs[hub_idx];
            hub.show_hidden_files = !hub.show_hidden_files;
        } else if request.starts_with("dir_toggle_select:") {
            let name = &request[18..].replace("\\'", "'");
            let sector_idx = state.viewports[state.active_viewport_index].sector_index;
            let hub_idx = state.viewports[state.active_viewport_index].hub_index;
            let hub = &mut state.sectors[sector_idx].hubs[hub_idx];
            if !hub.selected_files.remove(name) {
                hub.selected_files.insert(name.to_string());
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
                        self.ptys.lock().unwrap().insert(hub_id, pty);
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
        if let Ok(id) = Uuid::parse_str(id_str) {
            for sector in &mut state.sectors {
                for hub in &mut sector.hubs {
                    if let Some(pos) = hub.applications.iter().position(|a| a.id == id) {
                        let app = hub.applications.remove(pos);
                        println!("TOS // KILLED APP: {}", app.title);
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
        });
        
        let hub_idx = sector.hubs.len() - 1;

        if let Some(fish) = state.shell_registry.get("fish") {
            if let Some(pty) = fish.spawn(".") {
                self.ptys.lock().unwrap().insert(new_hub_id, pty);
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
