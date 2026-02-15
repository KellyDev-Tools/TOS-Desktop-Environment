use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use crate::system::pty::PtyHandle;
use crate::{TosState, CommandHubMode, HierarchyLevel, system::input::SemanticEvent, Sector, CommandHub, Application, Viewport, ConnectionType};

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
                let is_dangerous = cmd_full.contains("rm -rf") || cmd_full.contains(":(){ :|:& };:");
                let viewport = &state.viewports[state.active_viewport_index];
                let sector = &mut state.sectors[viewport.sector_index];
                let hub = &mut sector.hubs[viewport.hub_index];

                if is_dangerous && hub.confirmation_required.is_none() {
                    println!("!! DANGEROUS COMMAND DETECTED: {}", cmd_full);
                    hub.confirmation_required = Some(cmd_full.to_string());
                    return;
                }

                hub.confirmation_required = None;
                if let Some(pty) = self.ptys.lock().unwrap().get(&hub.id) {
                    pty.write(&format!("{}\n", cmd_full));
                }
            }
        }
    }

    fn handle_add_remote_sector(&self, state: &mut TosState) {
        let hub_id = Uuid::new_v4();
        let new_sector = Sector {
            id: Uuid::new_v4(),
            name: "Command Remote".to_string(),
            color: "#cc6666".to_string(),
            hubs: vec![CommandHub {
                id: hub_id,
                mode: CommandHubMode::Command,
                prompt: String::new(),
                applications: vec![Application {
                    id: Uuid::new_v4(),
                    title: "Remote Shell".to_string(),
                    app_class: "tos.remote".to_string(),
                    is_minimized: false,
                }],
                active_app_index: Some(0),
                terminal_output: Vec::new(),
                confirmation_required: None,
            }],
            active_hub_index: 0,
            host: "10.0.4.15".to_string(),
            connection_type: ConnectionType::SSH,
            participants: Vec::new(),
            portal_active: false,
            portal_url: None,
        };
        state.add_sector(new_sector);
        
        if let Some(pty) = PtyHandle::spawn("/usr/bin/fish", ".") {
            self.ptys.lock().unwrap().insert(hub_id, pty);
        }
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
        });
        
        let hub_idx = sector.hubs.len() - 1;

        if let Some(pty) = PtyHandle::spawn("/usr/bin/fish", ".") {
            self.ptys.lock().unwrap().insert(new_hub_id, pty);
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
            "OpenGlobalOverview" => state.handle_semantic_event(SemanticEvent::OpenGlobalOverview),
            "VoiceCommandStart" => {
                tracing::info!("VOICE COMMAND INITIATED");
                state.stage_command("LISTENING...".to_string());
            }
            _ => tracing::warn!("Unknown semantic event from IPC: {}", event_name),
        }
    }
}
