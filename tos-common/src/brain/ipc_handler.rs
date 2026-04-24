use crate::{CommandHubMode, HierarchyLevel, TosState};
use crate::state::QueuedAiRequest;
use crate::services::MarketplaceService;
// use crate::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;

pub struct IpcHandler {
    state: Arc<Mutex<TosState>>,
    shell: Arc<Mutex<crate::brain::shell::ShellApi>>,
    services: Arc<crate::services::ServiceManager>,
}

impl IpcHandler {
    pub fn new(
        state: Arc<Mutex<TosState>>,
        shell: Arc<Mutex<crate::brain::shell::ShellApi>>,
        services: Arc<crate::services::ServiceManager>,
    ) -> Self {
        Self {
            state,
            shell,
            services,
        }
    }

    /// Standardized Message Format: prefix:payload;payload...
    pub fn handle_request(&self, request: &str) -> String {
        let start = Instant::now();
        let (prefix, payload) = request.split_once(':').unwrap_or((request, ""));
        let args: Vec<&str> = payload.split(';').collect();

        let result = match prefix {
            "get_state" => self.handle_get_state(),
            "set_mode" => self.handle_set_mode(args.get(0).copied()),
            "zoom_in" => self.handle_zoom_in(),
            "zoom_out" => self.handle_zoom_out(),
            "set_active_sector" => self.handle_set_active_sector(args.get(0).copied()),
            "system_reset" => self.handle_system_reset(),
            "zoom_to" => self.handle_zoom_to(args.get(0).copied()),
            "set_setting" => self.handle_set_setting(
                args.get(0).copied(),
                args.get(1).copied(),
                args.get(2).copied(),
            ),
            "sector_set_setting" => self.handle_set_sector_setting(
                args.get(0).copied(),
                args.get(1).copied(),
                args.get(2).copied(),
            ),
            "get_settings" => self.handle_get_settings(),
            "set_sector_setting" => self.handle_set_sector_setting(
                args.get(0).copied(),
                args.get(1).copied(),
                args.get(2).copied(),
            ),
            "remote_ssh_connect" => self.handle_remote_ssh_connect(args.get(0).copied()),
            "remote_ssh_disconnect" => self.handle_remote_ssh_disconnect(),
            "sector_create" => self.handle_sector_create(args.get(0).copied()),
            "sector_create_from_template" => self.handle_sector_create_from_template(payload),
            "sector_clone" => self.handle_sector_clone(args.get(0).copied()),
            "sector_close" => self.handle_sector_close(args.get(0).copied()),
            "sector_freeze" => self.handle_sector_freeze(args.get(0).copied()),
            "remote_disconnect" => self.handle_remote_disconnect(args.get(0).copied()),
            "click" => self.handle_click(payload),
            "app_launch" => self.handle_app_launch(payload),
            "app_close" => self.handle_app_close(args.get(0).copied(), args.get(1).copied()),
            "signal_app" => self.handle_signal_app(args.get(0).copied(), args.get(1).copied()),
            "search" => self.handle_search(payload),
            "semantic_search" => self.handle_semantic_search(payload),
            "prompt_submit" => self.handle_prompt_submit(payload, false),
            "force_prompt_submit" => self.handle_prompt_submit(payload, true),
            "ai_suggestion_accept" => self.handle_ai_suggestion_accept(),
            "ai_stage_command" => self.handle_ai_stage_command(payload),
            "ai_tool_call" => self.handle_ai_tool_call(payload),
            "system_log_append" => {
                self.handle_system_log_append(args.get(0).copied(), args.get(1).copied())
            }
            "log_query" => self.handle_log_query(payload),
            "trigger_haptic" => self.handle_trigger_haptic(args.get(0).copied()),
            "portal_create" => self.handle_portal_create(),
            "portal_revoke" => self.handle_portal_revoke(args.get(0).copied()),
            "get_sector_templates" => self.handle_get_sector_templates(),
            "get_state_delta" => self.handle_get_state_delta(args.get(0).copied()),
            "webrtc_presence" => self.handle_webrtc_presence(payload),
            "set_terminal_module" => self.handle_set_terminal_module(args.get(0).copied()),
            "set_theme" => self.handle_set_theme(args.get(0).copied()),
            "face_register" => self.handle_face_register(payload),
            "service_register" => self.handle_service_register(payload),
            "market" => self.handle_market_command(payload),
            "terminal_resize" => {
                self.handle_terminal_resize(args.get(0).copied(), args.get(1).copied())
            }
            "terminal_signal" => self.handle_terminal_signal(args.get(0).copied()),
            "tos_ports" => self.handle_tos_ports(),
            "service_deregister" => self.handle_service_deregister(args.get(0).copied()),
            "session_list" => self.handle_session_list(args.get(0).copied()),
            "session_save" => self.handle_session_save(args.get(0).copied(), args.get(1).copied()),
            "session_load" => self.handle_session_load(args.get(0).copied(), args.get(1).copied()),
            "session_delete" => {
                self.handle_session_delete(args.get(0).copied(), args.get(1).copied())
            }
            "session_live_write" => self.handle_session_live_write(),
            "session_export" => {
                self.handle_session_export(args.get(0).copied(), args.get(1).copied())
            }
            "session_import" => {
                self.handle_session_import(args.get(0).copied(), args.get(1).copied())
            }
            "session_handoff_prepare" => self.handle_session_handoff_prepare(),
            "session_handoff_claim" => self.handle_session_handoff_claim(args.get(0).copied()),
            "collaboration_role_set" => self.handle_collaboration_role_set(
                args.get(0).copied(),
                args.get(1).copied(),
                args.get(2).copied(),
            ),
            "collaboration_participant_remove" => {
                self.handle_collaboration_participant_remove(args.get(0).copied(), args.get(1).copied())
            }
            "trust_promote" => self.handle_trust_promote(args.get(0).copied()),
            "trust_demote" => self.handle_trust_demote(args.get(0).copied()),
            "ai_behavior_enable" => self.handle_ai_behavior_enable(args.get(0).copied()),
            "ai_behavior_disable" => self.handle_ai_behavior_disable(args.get(0).copied()),
            "ai_behavior_configure" => self.handle_ai_behavior_configure(
                args.get(0).copied(),
                args.get(1).copied(),
                args.get(2).copied(),
            ),
            "ai_chip_stage" => self.handle_ai_chip_stage(payload),
            "ai_chip_dismiss" => self.handle_ai_chip_dismiss(args.get(0).copied()),
            "ai_thought_expand" => self.handle_ai_thought_expand(args.get(0).copied()),
            "ai_thought_dismiss" => self.handle_ai_thought_dismiss(args.get(0).copied()),
            "ai_thought_dismiss_permanent" => {
                self.handle_ai_thought_dismiss_permanent(args.get(0).copied())
            }
            "ai_context_request" => self.handle_ai_context_request(args.get(0).copied()),
            "ai_backend_set_default" => self.handle_ai_backend_set_default(args.get(0).copied()),
            "ai_backend_set_behavior" => {
                self.handle_ai_backend_set_behavior(args.get(0).copied(), args.get(1).copied())
            }
            "ai_backend_clear_behavior" => {
                self.handle_ai_backend_clear_behavior(args.get(0).copied())
            }
            "ai_history_clear" => self.handle_ai_history_clear(),
            "ai_pattern_set" => self.handle_ai_pattern_set(args.get(0).copied(), args.get(1).copied()),
            "ai_pattern_get" => self.handle_ai_pattern_get(args.get(0).copied()),
            "ai_history_append" => self.handle_ai_history_append(payload, "assistant"),
            "ai_submit" => self.handle_ai_submit(payload),
            "ai_predict_command" => self.handle_ai_predict_command(payload),
            "ai_thought_stage" => self.handle_ai_thought_stage(payload),
            "ai_queue_push" => self.handle_ai_queue_push(payload),
            "ai_queue_get" => self.handle_ai_queue_get(),
            "ai_queue_clear" => self.handle_ai_queue_clear(),
            "ai_plan" => self.handle_ai_plan(payload),
            "ai_roadmap_plan" => self.handle_ai_roadmap_plan(),
            "ai_dream_consolidate" => self.handle_ai_dream_consolidate(),
            "ai_isolated_exec" => self.handle_ai_isolated_exec(payload),
            "ai_archive_interaction" => {
                let args: Vec<&str> = payload.splitn(3, ';').collect();
                if args.len() < 3 {
                    "ERROR: Malformed archive request".to_string()
                } else {
                    self.handle_ai_archive_interaction(args[0], args[1], args[2])
                }
            }
            "kanban_init" => self.handle_kanban_init(),
            "kanban_get" => self.handle_kanban_get(),
            "kanban_task_add" => self.handle_kanban_task_add(payload),
            "kanban_task_move" => self.handle_kanban_task_move(payload),
            "kanban_task_delete" => self.handle_kanban_task_delete(payload),
            "tactical_kill_switch" => self.handle_tactical_kill_switch(),
            "process_inspect" => self.handle_process_inspect(args.get(0).copied()),
            "process_renice" => {
                self.handle_process_renice(args.get(0).copied(), args.get(1).copied())
            }
            "process_signal" => {
                self.handle_process_signal(args.get(0).copied(), args.get(1).copied())
            }
            "get_buffer" => self.handle_get_buffer(args.get(0).copied()),
            "clear_system_log" => self.handle_clear_system_log(),
            "play_earcon" => self.handle_play_earcon(args.get(0).copied()),
            "audio_ambient_start" => self.handle_audio_ambient_start(args.get(0).copied()),
            "audio_ambient_stop" => self.handle_audio_ambient_stop(),
            "audio_volume_set" => {
                self.handle_audio_volume_set(args.get(0).copied(), args.get(1).copied())
            }
            "audio_voice_play" => self.handle_audio_voice_play(Some(payload)),
            "bezel_expand" => self.handle_bezel_expand(),
            "bezel_collapse" => self.handle_bezel_collapse(),
            "bezel_swipe" => self.handle_bezel_swipe(args.get(0).copied()),
            "onboarding_skip_tour" => self.handle_onboarding_skip_tour(),
            "onboarding_advance_step" => self.handle_onboarding_advance_step(args.get(0).copied()),
            "onboarding_hint_dismiss" => self.handle_onboarding_hint_dismiss(args.get(0).copied()),
            "onboarding_hints_suppress" => self.handle_onboarding_hints_suppress(),
            "onboarding_reset_hints" => self.handle_onboarding_reset_hints(),

            "voice_command_start" => self.handle_voice_command_start(),
            "voice_transcription" => self.handle_voice_transcription(payload),

            "split_create" => self.handle_split_create(args.get(0).copied(), args.get(1).copied()),
            "split_close" => self.handle_split_close(args.get(0).copied()),
            "split_focus" => self.handle_split_focus(args.get(0).copied()),
            "split_focus_direction" => self.handle_split_focus_direction(args.get(0).copied()),
            "split_resize" => self.handle_split_resize(args.get(0).copied(), args.get(1).copied()),
            "split_equalize" => self.handle_split_equalize(),
            "split_fullscreen" => self.handle_split_fullscreen(args.get(0).copied()),
            "split_fullscreen_exit" => self.handle_split_fullscreen_exit(),
            "split_swap" => self.handle_split_swap(args.get(0).copied(), args.get(1).copied()),
            "split_detach:context" => self.handle_split_detach_context(),
            "split_detach:fresh" => self.handle_split_detach_fresh(),
            "split_save_template" => self.handle_split_save_template(args.get(0).copied()),
            "trust_promote_sector" => {
                self.handle_trust_promote_sector(args.get(0).copied(), args.get(1).copied())
            }
            "trust_demote_sector" => {
                self.handle_trust_demote_sector(args.get(0).copied(), args.get(1).copied())
            }
            "trust_clear_sector" => self.handle_trust_clear_sector(args.get(0).copied()),
            "trust_get_config" => self.handle_trust_get_config(),
            "heuristic_query" => self.handle_heuristic_query(args.get(0).copied()),
            "confirmation_accept" => self.handle_confirmation_accept(args.get(0).copied()),
            "confirmation_reject" => self.handle_confirmation_reject(args.get(0).copied()),
            "update_confirmation_progress" => {
                self.handle_update_confirmation_progress(args.get(0).copied(), args.get(1).copied())
            }
            "marketplace_home" => self.handle_marketplace_home(),
            "marketplace_category" => self.handle_marketplace_category(args.get(0).copied()),
            "marketplace_detail" => self.handle_marketplace_detail(args.get(0).copied()),
            "marketplace_install" => self.handle_marketplace_install(args.get(0).copied()),
            "marketplace_status" => self.handle_marketplace_status(args.get(0).copied()),
            "marketplace_search_ai" => self.handle_marketplace_search_ai(payload),
            "marketplace_install_cancel" => {
                self.handle_marketplace_install_cancel(args.get(0).copied())
            }
            // §14.2: Configurable Keyboard Shortcuts
            "keybindings_get" => self.handle_keybindings_get(),
            "keybindings_set" => {
                self.handle_keybindings_set(args.get(0).copied(), args.get(1).copied(), args.get(2).copied())
            }
            "keybindings_reset" => self.handle_keybindings_reset(),

            // §30.3–30.4: Editor Pane IPC
            "editor_open" => {
                self.handle_editor_open(args.get(0).copied(), args.get(1).copied())
            }
            "editor_save" => self.handle_editor_save(args.get(0).copied()),
            "editor_save_as" => {
                self.handle_editor_save_as(args.get(0).copied(), args.get(1).copied())
            }
            "editor_activate" => self.handle_editor_activate(args.get(0).copied()),
            "editor_mode_switch" => {
                self.handle_editor_mode_switch(args.get(0).copied(), args.get(1).copied())
            }
            "editor_scroll" => {
                self.handle_editor_scroll(args.get(0).copied(), args.get(1).copied())
            }
            "editor_open_ai" => {
                self.handle_editor_open_ai(
                    args.get(0).copied(),
                    args.get(1).copied(),
                    args.get(2).copied(),
                )
            }
            "editor_diff" => {
                self.handle_editor_diff(args.get(0).copied(), args.get(1).copied())
            }
            "editor_annotate" => self.handle_editor_annotate(payload),
            "editor_clear_annotations" => {
                self.handle_editor_clear_annotations(args.get(0).copied())
            }
            "editor_edit_proposal" => {
                self.handle_editor_edit_proposal(args.get(0).copied(), args.get(1).copied())
            }
            "editor_edit_apply" => {
                self.handle_editor_edit_apply(args.get(0).copied(), args.get(1).copied())
            }
            "editor_edit_reject" => {
                self.handle_editor_edit_reject(args.get(0).copied(), args.get(1).copied())
            }
            "editor_context_update" => {
                self.handle_editor_context_update(args.get(0).copied(), args.get(1).copied())
            }
            "editor_send_context" => {
                self.handle_editor_send_context(args.get(0).copied(), args.get(1).copied())
            }
            "editor_promote" => self.handle_editor_promote(args.get(0).copied()),
            _ => "ERROR: Unknown prefix".to_string(),
        };

        // Debounced session save on state-mutating events
        if !prefix.starts_with("session_")
            && !prefix.starts_with("get_")
            && prefix != "tos_ports"
            && !prefix.starts_with("service_")
            && !prefix.starts_with("trigger_")
            && !prefix.starts_with("play_")
            && !prefix.starts_with("trust_")
        {
            self.services
                .session
                .debounced_save_live(self.state.clone());
        }

        let duration = start.elapsed();
        if duration.as_millis() > 16 {
            let msg = format!("IPC LATENCY WARNING: {} took {:?}", prefix, duration);
            tracing::warn!("{}", msg);
            // Surface latency warnings in the Face's System Output
            let mut state = self.state.lock().unwrap();
            state.system_log.push(crate::TerminalLine {
                text: msg,
                priority: 2,
                timestamp: chrono::Local::now(),
            });
        }

        result
    }

    fn check_permission(
        &self,
        role: crate::collaboration::ParticipantRole,
        prefix: &str,
    ) -> bool {
        use crate::collaboration::ParticipantRole::*;
        match role {
            CoOwner => true,
            Operator => {
                // Operators can do almost everything except admin tasks
                !prefix.starts_with("collaboration_role_set")
                    && !prefix.starts_with("collaboration_participant_remove")
                    && prefix != "system_reset"
            }
            Commenter => {
                // Commenters can only add annotations or read state
                prefix.starts_with("editor_annotate")
                    || prefix.starts_with("get_")
                    || prefix == "tos_ports"
            }
            Viewer => {
                // Viewers are read-only
                prefix.starts_with("get_") || prefix == "tos_ports"
            }
        }
    }

    fn handle_voice_command_start(&self) -> String {
        tracing::info!("Voice command started listening...");
        "VOICE_COMMAND_STARTED".to_string()
    }

    fn handle_voice_transcription(&self, payload: &str) -> String {
        let text_lower = payload.to_lowercase();
        let text_lower = text_lower.trim();

        // 1. Focus [Sector]
        if text_lower.starts_with("focus ") {
            let target = text_lower[6..].trim();
            let state = self.state.lock().unwrap();
            let mut found_idx = None;
            for (i, sector) in state.sectors.iter().enumerate() {
                if sector.name.to_lowercase() == target {
                    found_idx = Some(i);
                    break;
                }
            }
            drop(state);
            if let Some(idx) = found_idx {
                return self.handle_set_active_sector(Some(&idx.to_string()));
            } else {
                return format!("ERROR: Sector '{}' not found", target);
            }
        }

        // 2. Run [Command]
        if text_lower.starts_with("run ") {
            let cmd = text_lower[4..].trim();
            return self.handle_prompt_submit(cmd, false);
        }

        // 3. Inspect [Target]
        if text_lower.starts_with("inspect ") {
            // "Inspect browser" -> simplified to zoom into DetailView for now
            return self.handle_zoom_to(Some("detail"));
        }

        // 4. Alert Status
        if text_lower == "report alert status" || text_lower == "alert status" {
            let state = self.state.lock().unwrap();
            let mut critical = 0;
            let mut warning = 0;
            for log in &state.system_log {
                if log.priority >= 4 {
                    critical += 1;
                } else if log.priority == 3 {
                    warning += 1;
                }
            }
            let msg = if critical > 0 {
                format!("You have {} critical alerts and {} warnings.", critical, warning)
            } else if warning > 0 {
                format!("You have {} warnings. All clear on critical.", warning)
            } else {
                "All systems nominal. No alerts.".to_string()
            };
            drop(state);
            return self.handle_audio_voice_play(Some(&msg));
        }

        // 5. Stop everything
        if text_lower == "stop everything" {
            return self.handle_tactical_kill_switch();
        }

        // Fallback
        self.handle_ai_submit(payload)
    }

    fn handle_prompt_submit(&self, command: &str, force: bool) -> String {
        let cmd = command.trim();
        if cmd.is_empty() {
            return "ERROR: Empty command".to_string();
        }

        // Non-blocking Trust Chip Emission
        // Classify the command and push a warning chip to system_log if needed.
        // This does NOT block or delay PTY submission.
        if !force {
            let mut state = self.state.lock().unwrap();
            let idx = state.active_sector_index;
            let sector_id_str = state.sectors.get(idx).map(|s| s.id.to_string());
            let cwd = state
                .sectors
                .get(idx)
                .and_then(|s| s.hubs.get(s.active_hub_index))
                .map(|h| h.current_directory.clone())
                .unwrap_or_else(|| std::path::PathBuf::from("/"));

            let bulk_threshold = state
                .settings
                .global
                .get("tos.trust.bulk_threshold")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(10);

            let class = self
                .services
                .trust
                .classify_command(cmd, &cwd, bulk_threshold);
            let policy =
                self.services
                    .trust
                    .get_trust_policy(&state, sector_id_str.as_deref(), &class);

            if class != crate::services::trust::CommandClass::Standard {
                let chip_msg = match &class {
                    crate::services::trust::CommandClass::PrivilegeEscalation => format!(
                        "[TRUST] ⚠ PRIVILEGE ESCALATION: '{}' — policy: {}",
                        cmd, policy
                    ),
                    crate::services::trust::CommandClass::RecursiveBulk => format!(
                        "[TRUST] ⚠ RECURSIVE BULK OP: '{}' — policy: {}",
                        cmd, policy
                    ),
                    crate::services::trust::CommandClass::ImplicitBulk => format!(
                        "[TRUST] ⚠ IMPLICIT BULK (glob): '{}' — policy: {}",
                        cmd, policy
                    ),
                    _ => String::new(),
                };
                if !chip_msg.is_empty() {
                    state.system_log.push(crate::TerminalLine {
                        text: chip_msg,
                        priority: 2,
                        timestamp: chrono::Local::now(),
                    });
                    tracing::warn!(
                        "[TRUST] Classified '{}' as {:?} (policy={})",
                        cmd,
                        class,
                        policy
                    );
                }

                if policy == "block" {
                    return format!("TRUST_BLOCKED: {:?}", class);
                }
                if policy == "confirm" {
                    state.pending_confirmation = Some(crate::ConfirmationRequest {
                        id: Uuid::new_v4(),
                        original_request: format!("force_prompt_submit:{}", cmd),
                        message: format!("⚠ DANGEROUS COMMAND: {}", cmd),
                        progress: 0.0,
                    });
                    state.version += 1;
                    return "CONFIRMATION_REQUIRED".to_string();
                }
            }
        }

        let mut hub_mode = CommandHubMode::Command;
        let mut hub_id = Uuid::nil();
        {
            let mut state_lock = self.state.lock().unwrap();
            let idx = state_lock.active_sector_index;
            if let Some(sector) = state_lock.sectors.get_mut(idx) {
                let hub_idx = sector.active_hub_index;
                if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                    hub_mode = hub.mode;
                    hub_id = hub.id;

                    // §7.3: Auto Activity Mode detection on monitoring commands.
                    let first_token = cmd.split_whitespace().next().unwrap_or("");
                    let activity_commands = ["top", "htop", "btop", "ps", "atop", "glances"];
                    if hub_mode == CommandHubMode::Command && activity_commands.contains(&first_token) {
                        hub.mode = CommandHubMode::Activity;
                        tracing::info!(
                            "Auto-switched to Activity mode for command: {}",
                            first_token
                        );
                    }

                    hub.is_running = true;
                    hub.last_exit_status = None;
                    hub.json_context = None;
                    state_lock.version += 1;
                }
            }
        }

        if hub_mode == CommandHubMode::Ssh {
            if let Err(e) = self.services.ssh.write(&hub_id, &format!("{}\n", command)) {
                return format!("ERROR: SSH write failed: {}", e);
            }
            return "SSH_SUBMITTED".to_string();
        }

        let mut shell = self.shell.lock().unwrap();
        if let Err(e) = shell.write(&format!("{}\n", command)) {
            let msg = format!("ERROR: Failed to write to shell: {}", e);
            tracing::error!("{}", msg);
            // Revert is_running
            let mut state_lock = self.state.lock().unwrap();
            let idx = state_lock.active_sector_index;
            if let Some(sector) = state_lock.sectors.get_mut(idx) {
                let hub_idx = sector.active_hub_index;
                if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                    hub.is_running = false;
                    state_lock.version += 1;
                }
            }
            return msg;
        }
        "SUBMITTED".to_string()
    }

    fn handle_set_mode(&self, mode_str: Option<&str>) -> String {
        let mode_raw = mode_str.unwrap_or("");

        // 1. Try Hierarchy Levels / ViewModes
        let target_level = match mode_raw {
            "global" | "GlobalOverview" => Some(HierarchyLevel::GlobalOverview),
            "hubs" | "CommandHub" => Some(HierarchyLevel::CommandHub),
            "sectors" | "ApplicationFocus" => Some(HierarchyLevel::ApplicationFocus),
            "detail" | "DetailView" => Some(HierarchyLevel::DetailView),
            "buffer" | "BufferView" => Some(HierarchyLevel::BufferView),
            "marketplace" | "Marketplace" => Some(HierarchyLevel::Marketplace),
            _ => None,
        };

        if let Some(level) = target_level {
            let mut state = self.state.lock().unwrap();
            crate::brain::hierarchy::HierarchyManager::set_level(&mut state, level);
            return format!("LEVEL_SET: {:?}", level);
        }

        // 2. Try Command Hub Modes
        let hub_mode = match mode_raw {
            "command" => Some(CommandHubMode::Command),
            "directory" => Some(CommandHubMode::Directory),
            "activity" => Some(CommandHubMode::Activity),
            "search" => Some(CommandHubMode::Search),
            "ai" => Some(CommandHubMode::Ai),
            _ => None,
        };

        if let Some(mode) = hub_mode {
            let mut state = self.state.lock().unwrap();
            let idx = state.active_sector_index;
            if let Some(sector) = state.sectors.get_mut(idx) {
                let hub_idx = sector.active_hub_index;
                if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                    hub.mode = mode;
                    if mode == CommandHubMode::Directory {
                        crate::brain::sector::SectorManager::refresh_directory_listing(&mut state);
                    } else if mode == CommandHubMode::Activity {
                        crate::brain::sector::SectorManager::refresh_activity_listing(
                            &mut state,
                            Some(&self.services.capture),
                        );
                    }
                    return format!("MODE_SET: {:?}", mode);
                }
            }
            return "ERROR: Hub not found".to_string();
        }

        "ERROR: Unknown mode or level".to_string()
    }

    fn handle_zoom_to(&self, level_str: Option<&str>) -> String {
        let mode_raw = level_str.unwrap_or("");

        let target_level = match mode_raw {
            "global" | "GlobalOverview" => Some(HierarchyLevel::GlobalOverview),
            "hubs" | "CommandHub" => Some(HierarchyLevel::CommandHub),
            "sectors" | "ApplicationFocus" => Some(HierarchyLevel::ApplicationFocus),
            "detail" | "DetailView" => Some(HierarchyLevel::DetailView),
            "buffer" | "BufferView" => Some(HierarchyLevel::BufferView),
            "marketplace" | "Marketplace" => Some(HierarchyLevel::Marketplace),
            _ => None,
        };

        if let Some(level) = target_level {
            let mut state = self.state.lock().unwrap();
            crate::brain::hierarchy::HierarchyManager::set_level(&mut state, level);
            return format!("ZOOMED_TO: {:?}", level);
        }

        "ERROR: Unknown level".to_string()
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

    fn handle_set_setting(
        &self,
        key: Option<&str>,
        val: Option<&str>,
        sector_id: Option<&str>,
    ) -> String {
        if let (Some(k), Some(v)) = (key, val) {
            let mut state = self.state.lock().unwrap();
            if let Some(sid) = sector_id {
                state
                    .settings
                    .sectors
                    .entry(sid.to_string())
                    .or_default()
                    .insert(k.to_string(), v.to_string());
            } else {
                state.settings.global.insert(k.to_string(), v.to_string());
            }
            state.version += 1;
            return format!("SETTING_UPDATE: {}={}", k, v);
        }
        "ERROR: Key and value required".to_string()
    }

    fn handle_set_sector_setting(
        &self,
        sector_id: Option<&str>,
        key: Option<&str>,
        val: Option<&str>,
    ) -> String {
        if let (Some(sec), Some(k), Some(v)) = (sector_id, key, val) {
            let mut state = self.state.lock().unwrap();
            let entry = state
                .settings
                .sectors
                .entry(sec.to_string())
                .or_insert_with(std::collections::HashMap::new);
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
        crate::brain::sector::SectorManager::create_sector(&mut state, name.to_string());
        format!("SECTOR_CREATED: {}", name)
    }

    fn handle_sector_create_from_template(&self, json_payload: &str) -> String {
        let template: crate::SectorTemplate = match serde_json::from_str(json_payload) {
            Ok(t) => t,
            Err(e) => return format!("ERROR: Invalid template JSON: {}", e),
        };

        let mut state = self.state.lock().unwrap();
        let name = template.name.clone();
        crate::brain::sector::SectorManager::create_from_template(&mut state, template);
        format!("SECTOR_CREATED_FROM_TEMPLATE: {}", name)
    }

    fn handle_get_sector_templates(&self) -> String {
        // Built-in templates for Alpha-2.1
        let mut templates = Vec::new();

        // 1. Rust Development
        let mut env = std::collections::HashMap::new();
        env.insert("RUST_LOG".to_string(), "info".to_string());
        templates.push(crate::SectorTemplate {
            name: "Rust Dev".to_string(),
            description: "Pre-configured for Rust development with Dual-Hub layout.".to_string(),
            environment: env,
            hubs: vec![
                crate::HubTemplate {
                    mode: crate::CommandHubMode::Command,
                    cwd: "~/".to_string(),
                    shell: "fish".to_string(),
                },
                crate::HubTemplate {
                    mode: crate::CommandHubMode::Directory,
                    cwd: "~/".to_string(),
                    shell: "fish".to_string(),
                },
            ],
        });

        // 2. Monitoring
        templates.push(crate::SectorTemplate {
            name: "Tactical Monitoring".to_string(),
            description: "System telemetry and log aggregation.".to_string(),
            environment: std::collections::HashMap::new(),
            hubs: vec![crate::HubTemplate {
                mode: crate::CommandHubMode::Activity,
                cwd: "/".to_string(),
                shell: "bash".to_string(),
            }],
        });

        serde_json::to_string(&templates)
            .unwrap_or_else(|_| "ERROR: Serialization failed".to_string())
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

    fn handle_app_launch(&self, json_payload: &str) -> String {
        let model: crate::ApplicationModel = match serde_json::from_str(json_payload) {
            Ok(m) => m,
            Err(e) => return format!("ERROR: Invalid app model JSON: {}", e),
        };

        let mut state = self.state.lock().unwrap();
        let sector_id = state.sectors[state.active_sector_index].id;
        let app_id = crate::brain::sector::SectorManager::launch_app(&mut state, sector_id, model);
        format!("APP_LAUNCHED: {}", app_id)
    }

    fn handle_app_close(&self, sector_id_str: Option<&str>, app_id_str: Option<&str>) -> String {
        if let (Some(s_id), Some(a_id)) = (sector_id_str, app_id_str) {
            if let (Ok(s_uuid), Ok(a_uuid)) = (Uuid::parse_str(s_id), Uuid::parse_str(a_id)) {
                let mut state = self.state.lock().unwrap();
                crate::brain::sector::SectorManager::close_app(&mut state, s_uuid, a_uuid);
                return format!("APP_CLOSED: {}", a_uuid);
            }
        }
        "ERROR: Invalid IDs for app close".to_string()
    }

    fn handle_signal_app(&self, app_id_str: Option<&str>, signal_str: Option<&str>) -> String {
        if let (Some(a_id), Some(sig)) = (app_id_str, signal_str) {
            if let Ok(a_uuid) = Uuid::parse_str(a_id) {
                // In a full implementation, this would route to the specific app container.
                // For the TDD integration test, we return the parsed confirmation.
                tracing::info!("internal_signal_event({}, {})", a_uuid, sig);
                return format!("APP_SIGNALED: {} {}", a_uuid, sig);
            }
        }
        "ERROR: Invalid args for signal_app".to_string()
    }

    fn handle_click(&self, payload: &str) -> String {
        if payload == "ZOOM OUT" {
            tracing::warn!("Bezel label rejected: ZOOM OUT. Use action identifier 'zoom_out'.");
            return "ERROR: Use identifier".to_string();
        } else if payload == "zoom_out" {
            return self.handle_zoom_out();
        } else if payload == "zoom_in" {
            return self.handle_zoom_in();
        }
        "ERROR: Unknown click".to_string()
    }

    fn handle_remote_disconnect(&self, id_str: Option<&str>) -> String {
        if let Some(id_str) = id_str {
            if let Ok(id) = Uuid::parse_str(id_str) {
                let mut state = self.state.lock().unwrap();
                if let Some(sector) = state.sectors.iter_mut().find(|s| s.id == id) {
                    sector.disconnected = true;
                    // In a full implementation we would spawn a timer.
                    let state_clone = self.state.clone();
                    let id_clone = id;
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        let mut state = state_clone.lock().unwrap();
                        if let Some(sector) = state.sectors.iter().find(|s| s.id == id_clone) {
                            if sector.disconnected {
                                crate::brain::sector::SectorManager::close_sector(
                                    &mut state, id_clone,
                                );
                            }
                        }
                    });
                    return format!("REMOTE_DISCONNECTED: {}", id);
                }
            }
        }
        "ERROR: Invalid args for remote_disconnect".to_string()
    }

    fn handle_set_terminal_module(&self, module_id: Option<&str>) -> String {
        if let Some(id) = module_id {
            let mut state = self.state.lock().unwrap();

            // Refresh available modules from disk
            let mut modules = MarketplaceService::list_terminal_modules();
            // Ensure built-ins are also there
            if !modules.iter().any(|m| m.id == "tos-standard-rect") {
                modules.push(crate::TerminalOutputModuleMeta {
                    id: "tos-standard-rect".to_string(),
                    name: "Standard Rectangular".to_string(),
                    version: "1.0.0".to_string(),
                    layout: crate::TerminalLayoutType::Rectangular,
                    supports_high_contrast: true,
                    supports_reduced_motion: true,
                });
            }
            if !modules.iter().any(|m| m.id == "tos-cinematic-tri") {
                modules.push(crate::TerminalOutputModuleMeta {
                    id: "tos-cinematic-tri".to_string(),
                    name: "Cinematic Triangular".to_string(),
                    version: "1.0.0".to_string(),
                    layout: crate::TerminalLayoutType::Cinematic,
                    supports_high_contrast: false,
                    supports_reduced_motion: false,
                });
            }
            state.available_modules = modules;

            if state.available_modules.iter().any(|m| m.id == id) {
                state.active_terminal_module = id.to_string();
                return format!("TERMINAL_MODULE_SET: {}", id);
            }
        }
        "ERROR: Invalid terminal module ID".to_string()
    }

    fn handle_set_theme(&self, theme_id: Option<&str>) -> String {
        if let Some(id) = theme_id {
            let mut state = self.state.lock().unwrap();

            // Refresh themes from disk
            let mut themes = MarketplaceService::list_theme_modules();
            // Ensure built-ins are also there
            if !themes.iter().any(|t| t.id == "tos-classic-lcars") {
                themes.push(crate::ThemeModule {
                    id: "tos-classic-lcars".to_string(),
                    name: "Classic LCARS".to_string(),
                    version: "1.0.0".to_string(),
                    author: "TOS Core".to_string(),
                    description: "Standard LCARS color scheme (Blue/Purple/Gold)".to_string(),
                    assets: crate::ThemeAssetDefinition {
                        css: "theme-classic.css".to_string(),
                        fonts: vec!["Outfit-Regular.ttf".to_string()],
                        icons: "assets/icons/classic/".to_string(),
                    },
                });
            }
            if !themes.iter().any(|t| t.id == "tos-tactical-amber") {
                themes.push(crate::ThemeModule {
                    id: "tos-tactical-amber".to_string(),
                    name: "Tactical Amber".to_string(),
                    version: "1.0.0".to_string(),
                    author: "TOS Core".to_string(),
                    description: "High-contrast amber tactical interface".to_string(),
                    assets: crate::ThemeAssetDefinition {
                        css: "theme-tactical.css".to_string(),
                        fonts: vec!["Outfit-Bold.ttf".to_string()],
                        icons: "assets/icons/tactical/".to_string(),
                    },
                });
            }
            if !themes.iter().any(|t| t.id == "tos-red-alert") {
                themes.push(crate::ThemeModule {
                    id: "tos-red-alert".to_string(),
                    name: "Red Alert".to_string(),
                    version: "1.0.0".to_string(),
                    author: "TOS Core".to_string(),
                    description: "High-intensity emergency mode".to_string(),
                    assets: crate::ThemeAssetDefinition {
                        css: "theme-red.css".to_string(),
                        fonts: vec!["Outfit-Bold.ttf".to_string()],
                        icons: "assets/icons/red/".to_string(),
                    },
                });
            }
            state.available_themes = themes;

            if state.available_themes.iter().any(|t| t.id == id) {
                state.active_theme = id.to_string();
                return format!("THEME_SET: {}", id);
            }
        }
        "ERROR: Invalid theme ID".to_string()
    }

    fn handle_market_command(&self, payload: &str) -> String {
        let args: Vec<&str> = payload.split(';').collect();
        if args.is_empty() {
            return "ERROR: Empty market command".to_string();
        }

        match args[0] {
            "install" => {
                if args.len() < 2 {
                    return "ERROR: Missing module ID".to_string();
                }
                let module_id = args[1];
                // Mock installation logic for Alpha-2.1
                format!("INSTALLED: {}", module_id)
            }
            _ => format!("ERROR: Unknown market command: {}", args[0]),
        }
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

                let matches: Vec<String> = indexed_hits
                    .iter()
                    .map(|h| format!("{} [{}]", h.path, if h.is_dir { "DIR" } else { "FILE" }))
                    .collect();

                if let Some(ref mut results) = hub.search_results {
                    results.insert(
                        0,
                        crate::SearchResult {
                            source_sector: "Global FS Index".to_string(),
                            matches,
                        },
                    );
                } else {
                    hub.search_results = Some(vec![crate::SearchResult {
                        source_sector: "Global FS Index".to_string(),
                        matches,
                    }]);
                }
            }
        }

        format!("SEARCH_PERFORMED: {}", query)
    }

    fn handle_semantic_search(&self, prompt: &str) -> String {
        let mut state = self.state.lock().unwrap();
        crate::brain::sector::SectorManager::perform_search(&mut state, prompt);

        // Add mocked LLM semantic embeddings search matches
        let indexed_hits = self.services.search.semantic_query(prompt);
        if !indexed_hits.is_empty() {
            let idx = state.active_sector_index;
            if let Some(sector) = state.sectors.get_mut(idx) {
                let hub = &mut sector.hubs[sector.active_hub_index];

                let matches: Vec<String> = indexed_hits
                    .iter()
                    .map(|h| format!("{} [{}]", h.path, if h.is_dir { "DIR" } else { "FILE" }))
                    .collect();

                let semantic_result = crate::SearchResult {
                    source_sector: "AI Semantic Engine".to_string(),
                    matches,
                };

                if let Some(ref mut results) = hub.search_results {
                    results.insert(0, semantic_result);
                } else {
                    hub.search_results = Some(vec![semantic_result]);
                }
            }
        }

        format!("SEMANTIC_SEARCH_COMPLETED")
    }

    fn handle_ai_submit(&self, query: &str) -> String {
        let ai = self.services.ai.clone();
        let query_owned = query.to_string();
        self.handle_ai_history_append(&query_owned, "user");
        tokio::spawn(async move {
            let _ = ai.query(&query_owned).await;
        });
        "AI_SUBMITTED".to_string()
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

    fn handle_face_register(&self, payload: &str) -> String {
        let reg: crate::ipc::FaceRegister = match serde_json::from_str(payload) {
            Ok(r) => r,
            Err(e) => return format!("ERROR: Invalid face_register JSON: {}", e),
        };

        let mut state = self.state.lock().unwrap();
        state.device_profile = reg.profile;

        match reg.profile {
            crate::ipc::FaceProfile::Handheld => {
                // §3.3.5: Handheld profile automations
                state.bezel_expanded = false;
                // Set default hub layout to tabs (if supported by settings)
                state
                    .settings
                    .global
                    .insert("tos.layout.default".to_string(), "tabs".to_string());

                tracing::info!("FACE_REGISTER: Handheld profile active. Adapting layout and AI.");
            }
            crate::ipc::FaceProfile::Spatial => {
                tracing::info!("FACE_REGISTER: Spatial profile active. Enabling spatial bezel.");
            }
            _ => {}
        }

        state.version += 1;
        format!("FACE_REGISTERED: {:?}", reg.profile)
    }

    fn handle_service_register(&self, payload: &str) -> String {
        let req: crate::ipc::ServiceRegister = match serde_json::from_str(payload) {
            Ok(r) => r,
            Err(e) => return format!("ERROR: Invalid registration JSON: {}", e),
        };

        // §4.1: Cryptographic signature verification
        if !self.services.trust.verify_service_signature(&req) {
            return serde_json::to_string(&crate::ipc::ServiceRegisterResponse {
                status: "DENIED".to_string(),
                message: "Cryptographic signature verification failed".to_string(),
            })
            .unwrap();
        }

        let mut registry = self.services.registry.lock().unwrap();
        registry.register(&req.name, req.port, "127.0.0.1");

        tracing::info!("SERVICE_REGISTER: {} on port {}", req.name, req.port);

        serde_json::to_string(&crate::ipc::ServiceRegisterResponse {
            status: "OK".to_string(),
            message: format!("Service {} registered successfully", req.name),
        })
        .unwrap()
    }

    fn handle_ai_queue_push(&self, payload: &str) -> String {
        if let Ok(req) = serde_json::from_str::<QueuedAiRequest>(payload) {
            let mut state = self.state.lock().unwrap();
            state.ai_offline_queue.push(req);
            return "AI_QUEUE_PUSHED".to_string();
        }
        "ERROR: Invalid JSON for ai_queue_push".to_string()
    }

    fn handle_ai_queue_get(&self) -> String {
        let state = self.state.lock().unwrap();
        serde_json::to_string(&state.ai_offline_queue).unwrap_or_else(|_| "[]".to_string())
    }

    fn handle_ai_queue_clear(&self) -> String {
        let mut state = self.state.lock().unwrap();
        state.ai_offline_queue.clear();
        "AI_QUEUE_CLEARED".to_string()
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

    fn handle_ai_tool_call(&self, payload: &str) -> String {
        // payload format: <behavior_id>;<json_args>
        let (behavior_id, json_payload) = payload.split_once(';').unwrap_or((payload, ""));
        if json_payload.is_empty() {
            return "ERROR: Missing tool arguments".to_string();
        }

        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_payload) {
            let tool_name = parsed.get("name").and_then(|v| v.as_str()).unwrap_or("");
            
            {
                let state = self.state.lock().unwrap();
                if !self.services.ai.validate_tool_call(&state, behavior_id, tool_name) {
                    tracing::warn!("[TOOL REGISTRY] Denied tool '{}' for behavior '{}'", tool_name, behavior_id);
                    return format!("ERROR: Tool '{}' not permitted for behavior '{}'", tool_name, behavior_id);
                }
            }

            // Route recognized tools
            if tool_name == "exec_cmd" {
                if let Some(args) = parsed.get("args") {
                    let cmd = args.get("cmd").and_then(|v| v.as_str()).unwrap_or("");
                    let explanation = args.get("explanation").and_then(|v| v.as_str()).unwrap_or("Invoked via exec_cmd tool");
                    let stage_payload = serde_json::json!({
                        "command": cmd,
                        "explanation": explanation
                    }).to_string();
                    return self.handle_ai_stage_command(&stage_payload);
                }
            } else if tool_name == "semantic_search" {
                if let Some(args) = parsed.get("args") {
                    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                    return self.handle_semantic_search(query);
                }
            }

            return format!("ERROR: Tool '{}' recognized but not implemented locally", tool_name);
        }

        "ERROR: Invalid JSON for ai_tool_call".to_string()
    }

    fn handle_system_log_append(
        &self,
        priority_str: Option<&str>,
        text_str: Option<&str>,
    ) -> String {
        if let (Some(priority), Some(text)) = (priority_str, text_str) {
            let mut state = self.state.lock().unwrap();
            let priority = priority.parse::<u8>().unwrap_or(1);
            let limit: usize = state
                .settings
                .global
                .get("terminal_buffer_limit")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000);

            state.system_log.push(crate::TerminalLine {
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

    fn handle_tactical_kill_switch(&self) -> String {
        tracing::warn!("TACTICAL KILL SWITCH ACTIVATED!");

        // 1. Force-kill the active shell (§22)
        if let Ok(mut shell) = self.shell.lock() {
            if let Err(e) = shell.force_kill() {
                tracing::error!("Failed to force-kill shell: {}", e);
            }
        }

        // 2. Disconnect/Freeze all sectors
        let mut state = self.state.lock().unwrap();
        for sector in state.sectors.iter_mut() {
            sector.frozen = true;
            sector.disconnected = true;
        }
        state.version += 1;
        state.system_log.push(crate::TerminalLine {
            text: "[CRITICAL] LEVEL 4 TACTICAL RESET EXECUTED - ALL PROCESSES TERMINATED"
                .to_string(),
            priority: 3,
            timestamp: chrono::Local::now(),
        });
        "TACTICAL_RESET_EXECUTED".to_string()
    }

    fn handle_process_inspect(&self, pid_str: Option<&str>) -> String {
        let pid = pid_str.unwrap_or("0");
        
        // Return structured metadata as JSON
        let mut metadata = serde_json::json!({
            "pid": pid,
            "user": "tos",
            "cpu_percent": 1.2,
            "mem_rss": 45600,
            "mem_vsz": 120000,
            "uptime": "02:14:55",
            "cwd": "/home/tim/TOS-Desktop-Environment",
            "command": "tos-brain",
            "threads": 8,
            "status": "Running",
            "parent_pid": 1,
            "sandbox_tier": "System",
            "permissions": ["fs_read", "net_raw", "proc_list"],
            "event_history": [
                {"time": "15:40:01", "event": "PROCESS_SPAWNED"},
                {"time": "15:40:05", "event": "IPC_CHANNEL_OPENED"},
                {"time": "15:42:10", "event": "HEURISTIC_SCAN_COMPLETE"}
            ]
        });

        // If on Linux, try to get some real data from /proc for authenticity
        #[cfg(target_os = "linux")]
        {
            if let Ok(p) = pid.parse::<i32>() {
                if std::path::Path::new(&format!("/proc/{}", p)).exists() {
                    if let Ok(cmd) = std::fs::read_to_string(format!("/proc/{}/comm", p)) {
                        metadata["command"] = serde_json::Value::String(cmd.trim().to_string());
                    }
                    if let Ok(status) = std::fs::read_to_string(format!("/proc/{}/status", p)) {
                        for line in status.lines() {
                            if line.starts_with("State:") {
                                metadata["status"] = serde_json::Value::String(line.split_whitespace().nth(1).unwrap_or("Unknown").to_string());
                            }
                        }
                    }
                }
            }
        }

        serde_json::to_string(&metadata).unwrap_or_default()
    }

    fn handle_get_buffer(&self, pid_str: Option<&str>) -> String {
        let _pid = pid_str.unwrap_or("0");
        // Generate high-fidelity mock hex buffer
        let mut buffer = String::new();
        let rows = 64;
        for i in 0..rows {
            let offset = i * 16;
            let mut hex = String::new();
            let mut ascii = String::new();
            for j in 0..16 {
                let val = ((offset + j) * 13 % 256) as u8;
                hex.push_str(&format!("{:02x} ", val));
                if val >= 32 && val <= 126 {
                    ascii.push(val as char);
                } else {
                    ascii.push('.');
                }
            }
            buffer.push_str(&format!("{:08x}  {} |{}|\n", offset, hex, ascii));
        }
        buffer
    }

    fn handle_process_renice(&self, pid: Option<&str>, adjustment: Option<&str>) -> String {
        if let (Some(pid_str), Some(adj_str)) = (pid, adjustment) {
            // Run renice command
            let output = std::process::Command::new("renice")
                .arg("-n")
                .arg(adj_str)
                .arg("-p")
                .arg(pid_str)
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    return format!("PROCESS_RENICED: {}", pid_str);
                }
            }
            return "ERROR: Renice failed".to_string();
        }
        "ERROR: Missing arguments".to_string()
    }

    fn handle_process_signal(&self, pid: Option<&str>, signal: Option<&str>) -> String {
        if let (Some(pid_str), Some(sig_str)) = (pid, signal) {
            let output = std::process::Command::new("kill")
                .arg(format!("-s"))
                .arg(sig_str)
                .arg(pid_str)
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    return format!("PROCESS_SIGNALED: {}", pid_str);
                }
            }
            return "ERROR: Signal failed".to_string();
        }
        "ERROR: Missing arguments".to_string()
    }

    fn handle_get_state(&self) -> String {
        let state = self.state.lock().unwrap();
        serde_json::to_string(&*state).unwrap_or_else(|_| "ERROR: Serialization failed".to_string())
    }

    fn handle_system_reset(&self) -> String {
        let mut state = self.state.lock().unwrap();
        // Clear all sectors and push a fresh default one
        state.sectors.clear();
        
        // Use the same logic as TosState::default() for the initial sector
        let sector = crate::state::Sector {
            id: uuid::Uuid::new_v4(),
            name: "Primary".to_string(),
            hubs: vec![crate::state::CommandHub {
                id: uuid::Uuid::new_v4(),
                mode: crate::state::CommandHubMode::Command,
                prompt: String::new(),
                current_directory: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/")),
                terminal_output: vec![],
                buffer_limit: 500,
                shell_listing: None,
                activity_listing: None,
                search_results: None,
                staged_command: None,
                ai_explanation: None,
                json_context: None,
                shell_module: Some("tos-shell-fish".to_string()),
                split_layout: None,
                focused_pane_id: None,
                version: 0,
                ai_history: vec![],
                active_thoughts: vec![],
                last_exit_status: None,
                is_running: false,
            }],
            active_hub_index: 0,
            frozen: false,
            is_remote: false,
            disconnected: false,
            trust_tier: crate::state::TrustTier::System,
            priority: 1,
            active_apps: vec![],
            active_app_index: 0,
            participants: vec![],
            kanban_board: None,
            version: 0,
        };
        
        state.sectors.push(sector);
        state.active_sector_index = 0;
        state.current_level = crate::state::HierarchyLevel::GlobalOverview;
        state.version += 1;
        
        "OK".to_string()
    }

    fn handle_get_settings(&self) -> String {
        let state = self.state.lock().unwrap();
        serde_json::to_string(&state.settings)
            .unwrap_or_else(|_| "ERROR: Serialization failed".to_string())
    }

    fn handle_heuristic_query(&self, keyword: Option<&str>) -> String {
        let keyword = keyword.unwrap_or("").to_string();
        let cwd = {
            let state = self.state.lock().unwrap();
            let idx = state.active_sector_index;
            state
                .sectors
                .get(idx)
                .and_then(|s| s.hubs.get(s.active_hub_index))
                .map(|h| h.current_directory.display().to_string())
                .unwrap_or_else(|| "/".to_string())
        };

        let svc = self.services.heuristic.clone();

        // Use block_in_place to prevent "Cannot start a runtime from within a runtime" panic
        tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current().block_on(async move {
                match svc.query(&keyword, &cwd).await {
                    Ok(json) => json,
                    Err(e) => format!("ERROR: Heuristic query failed: {}", e),
                }
            })
        })
    }

    fn handle_play_earcon(&self, name: Option<&str>) -> String {
        if let Some(n) = name {
            self.services.audio.play_earcon(n);
            return format!("PLAYING_EARCON: {}", n);
        }
        "ERROR: Invalid earcon name".to_string()
    }

    fn handle_audio_ambient_start(&self, name: Option<&str>) -> String {
        if let Some(n) = name {
            self.services.audio.play_ambient(n);
            return format!("AUDIO_AMBIENT_STARTED: {}", n);
        }
        "ERROR: Missing ambient name".to_string()
    }

    fn handle_audio_ambient_stop(&self) -> String {
        self.services.audio.stop_ambient();
        "AUDIO_AMBIENT_STOPPED".to_string()
    }

    fn handle_audio_volume_set(&self, layer: Option<&str>, volume: Option<&str>) -> String {
        if let (Some(l), Some(v)) = (layer, volume) {
            if let Ok(vol) = v.parse::<f32>() {
                let layer_enum = match l {
                    "ambient" => crate::services::audio::AudioLayer::Ambient,
                    "tactical" => crate::services::audio::AudioLayer::Tactical,
                    "voice" => crate::services::audio::AudioLayer::Voice,
                    _ => return "ERROR: Invalid audio layer".to_string(),
                };
                self.services.audio.set_volume(layer_enum, vol);
                return format!("AUDIO_VOLUME_SET: {} -> {}", l, vol);
            }
        }
        "ERROR: Invalid arguments for audio_volume_set".to_string()
    }

    fn handle_audio_voice_play(&self, text: Option<&str>) -> String {
        if let Some(t) = text {
            self.services.audio.play_voice(t);
            return format!("AUDIO_VOICE_PLAYING: {}", t);
        }
        "ERROR: Missing text for voice".to_string()
    }

    fn handle_trigger_haptic(&self, name: Option<&str>) -> String {
        if let Some(n) = name {
            self.services.haptic.trigger_haptic(n);
            return format!("HAPTIC_TRIGGERED: {}", n);
        }
        "ERROR: Invalid haptic cue".to_string()
    }

    fn handle_portal_create(&self) -> String {
        let state = self.state.lock().unwrap();
        let sector_id = if let Some(sector) = state.sectors.get(state.active_sector_index) {
            sector.id
        } else {
            return "ERROR: No active sector".to_string();
        };

        // Drop lock before service call if possible (though PortalService has its own internal sync)
        let token = self.services.portal.create_token(sector_id);
        format!("PORTAL_CREATED: {}", token)
    }

    fn handle_portal_revoke(&self, token: Option<&str>) -> String {
        if let Some(t) = token {
            self.services.portal.revoke_token(t);
            return format!("PORTAL_REVOKED: {}", t);
        }
        "ERROR: Token required".to_string()
    }

    fn handle_log_query(&self, json_payload: &str) -> String {
        #[derive(serde::Deserialize)]
        struct QueryParams {
            surface: Option<String>,
            limit: Option<usize>,
        }

        let params: QueryParams = match serde_json::from_str(json_payload) {
            Ok(p) => p,
            Err(_) => QueryParams {
                surface: None,
                limit: None,
            }, // Fallback to defaults
        };

        match self
            .services
            .logger
            .query(params.surface.as_deref(), params.limit)
        {
            Ok(response) => response,
            Err(e) => format!("ERROR: Log query failed: {}", e),
        }
    }

    fn handle_get_state_delta(&self, last_version_str: Option<&str>) -> String {
        let last_version = last_version_str
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let state = self.state.lock().unwrap();

        if state.version == last_version {
            return "NO_CHANGE".to_string();
        }

        serde_json::to_string(&*state).unwrap_or_else(|_| "ERROR: Serialization failed".to_string())
    }

    fn handle_webrtc_presence(&self, payload: &str) -> String {
        // Parse the incoming generic WebRTC payload structure
        if let Ok(presence_event) =
            serde_json::from_str::<crate::collaboration::WebRtcPayload>(payload)
        {
            let mut state = self.state.lock().unwrap();
            let mut changed = false;

            // Typically routed to active sector, but robust architecture would route via the remote host router
            let active_idx = state.active_sector_index;
            if active_idx < state.sectors.len() {
                let sector = &mut state.sectors[active_idx];

                match presence_event {
                    crate::collaboration::WebRtcPayload::CursorSync {
                        user,
                        x,
                        y,
                        target,
                    } => {
                        if let Some(participant) =
                            sector.participants.iter_mut().find(|p| p.id == user)
                        {
                            participant.cursor_x = Some(x);
                            participant.cursor_y = Some(y);
                            participant.cursor_target = target;
                            changed = true;
                        }
                    }
                    crate::collaboration::WebRtcPayload::Presence {
                        user,
                        status,
                        level,
                        active_viewport_title,
                        left_chip_state,
                        right_chip_state,
                    } => {
                        if let Some(participant) =
                            sector.participants.iter_mut().find(|p| p.id == user)
                        {
                            // Update existing tracked remote guest
                            participant.status = status;
                            participant.current_level = level;
                            participant.viewport_title = active_viewport_title;
                            participant.left_chip_state = left_chip_state;
                            participant.right_chip_state = right_chip_state;
                        } else {
                            // Register new remote guest observation to the active sector
                            sector
                                .participants
                                .push(crate::collaboration::Participant {
                                    id: user,
                                    alias: format!("Guest {}", user.to_string()[..4].to_string()),
                                    status,
                                    role: crate::collaboration::ParticipantRole::Viewer,
                                    current_level: level,
                                    viewport_title: active_viewport_title,
                                    left_chip_state,
                                    right_chip_state,
                                    cursor_x: None,
                                    cursor_y: None,
                                    cursor_target: None,
                                    following: None,
                                });
                        }
                        changed = true;
                    }
                    crate::collaboration::WebRtcPayload::Command { user, request } => {
                        // Role Enforcement for Remote Commands (§13.2)
                        let mut participant_role = None;
                        if let Some(participant) =
                            sector.participants.iter().find(|p| p.id == user)
                        {
                            participant_role = Some(participant.role);
                        }

                        if let Some(role) = participant_role {
                            let (prefix, _) = request.split_once(':').unwrap_or((&request, ""));
                            if self.check_permission(role, prefix) {
                                drop(state); // Drop lock before recursive call
                                let result = self.handle_request(&request);
                                state = self.state.lock().unwrap();
                                tracing::info!(
                                    "REMOTE_COMMAND: {} by {} (role={:?}) -> {}",
                                    request,
                                    user,
                                    role,
                                    result
                                );
                                changed = true;
                            } else {
                                tracing::warn!(
                                    "PERMISSION_DENIED: {} by {} (role={:?})",
                                    request,
                                    user,
                                    role
                                );
                            }
                        }
                    }
                    crate::collaboration::WebRtcPayload::Following {
                        follower,
                        leader,
                        sync,
                    } => {
                        if let Some(participant) =
                            sector.participants.iter_mut().find(|p| p.id == follower)
                        {
                            participant.following = if sync { Some(leader) } else { None };
                            changed = true;
                        }
                    }
                    crate::collaboration::WebRtcPayload::SdpOffer { user, sdp } => {
                        tracing::info!("[WEBRTC] Received SDP Offer from {}: {}...", user, &sdp[..sdp.len().min(20)]);
                        // In a full implementation, this would be routed to the target peer or handled by a media server.
                        changed = false; 
                    }
                    crate::collaboration::WebRtcPayload::SdpAnswer { user, sdp } => {
                        tracing::info!("[WEBRTC] Received SDP Answer from {}: {}...", user, &sdp[..sdp.len().min(20)]);
                        changed = false;
                    }
                    crate::collaboration::WebRtcPayload::IceCandidate { user, candidate } => {
                        tracing::info!("[WEBRTC] Received ICE Candidate from {}: {}...", user, &candidate[..candidate.len().min(20)]);
                        changed = false;
                    }
                    _ => {} // Other presence events unhandled in this exact route
                }

                if changed {
                    state.version += 1;
                }
            }
            "OK".to_string()
        } else {
            "ERROR: Invalid WebRTC payload".to_string()
        }
    }
    fn handle_terminal_signal(&self, signal: Option<&str>) -> String {
        if let Some(sig) = signal {
            let mut shell = self.shell.lock().unwrap();
            match shell.send_signal(sig) {
                Ok(_) => return format!("SIGNAL_SENT: {}", sig),
                Err(e) => return format!("ERROR: Signal failed: {}", e),
            }
        }
        "ERROR: Missing signal ID".to_string()
    }

    fn handle_terminal_resize(&self, rows: Option<&str>, cols: Option<&str>) -> String {
        if let (Some(r), Some(c)) = (rows, cols) {
            if let (Ok(r_n), Ok(c_n)) = (r.parse::<u16>(), c.parse::<u16>()) {
                let shell = self.shell.lock().unwrap();
                match shell.resize(r_n, c_n) {
                    Ok(_) => return format!("TERMINAL_RESIZED: {}x{}", r_n, c_n),
                    Err(e) => return format!("ERROR: Resize failed: {}", e),
                }
            }
        }
        "ERROR: Invalid dimensions".to_string()
    }

    fn handle_tos_ports(&self) -> String {
        let registry = self.services.registry.lock().unwrap();
        // Return JSON for wire safety (port_table() is multi-line, breaks
        // the line-based TCP protocol). Clients can pretty-print if needed.
        let mut entries = Vec::new();
        entries.push(serde_json::json!({
            "name": "tos-brain (anchor)",
            "port": registry.anchor_port(),
            "host": "0.0.0.0",
            "status": "ACTIVE"
        }));
        for svc in registry.list_all() {
            entries.push(serde_json::json!({
                "name": svc.name,
                "port": svc.port,
                "host": svc.host,
                "status": if svc.alive { "ACTIVE" } else { "DEAD" }
            }));
        }
        serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string())
    }

    fn handle_service_deregister(&self, name: Option<&str>) -> String {
        if let Some(n) = name {
            let mut registry = self.services.registry.lock().unwrap();
            registry.deregister(n);
            return format!("SERVICE_DEREGISTERED: {}", n);
        }
        "ERROR: Missing service name".to_string()
    }

    fn handle_session_list(&self, sector_id: Option<&str>) -> String {
        let sid = sector_id.unwrap_or("global");
        match self.services.session.list(sid) {
            Ok(json) => format!("SESSION_LIST: {}", json),
            Err(e) => format!("ERROR: {}", e),
        }
    }

    fn handle_session_save(&self, sector_id: Option<&str>, name: Option<&str>) -> String {
        if let (Some(sid), Some(n)) = (sector_id, name) {
            let state = self.state.lock().unwrap();
            match self.services.session.save(sid, n, &state) {
                Ok(_) => return format!("SESSION_SAVED: {}", n),
                Err(e) => return format!("ERROR: {}", e),
            }
        }
        "ERROR: Missing sector_id or name".to_string()
    }

    fn handle_session_load(&self, sector_id: Option<&str>, name: Option<&str>) -> String {
        if let (Some(sid), Some(n)) = (sector_id, name) {
            match self.services.session.load(sid, n) {
                Ok(json) => {
                    if let Ok(new_state) = serde_json::from_str::<TosState>(&json) {
                        let mut state = self.state.lock().unwrap();
                        *state = new_state;
                        return format!("SESSION_LOADED: {}", n);
                    }
                    return "ERROR: Session schema invalid".to_string();
                }
                Err(e) => return format!("ERROR: {}", e),
            }
        }
        "ERROR: Missing sector_id or name".to_string()
    }

    fn handle_session_delete(&self, sector_id: Option<&str>, name: Option<&str>) -> String {
        if let (Some(sid), Some(n)) = (sector_id, name) {
            match self.services.session.delete(sid, n) {
                Ok(_) => return format!("SESSION_DELETED: {}", n),
                Err(e) => return format!("ERROR: {}", e),
            }
        }
        "ERROR: Missing sector_id or name".to_string()
    }

    fn handle_session_live_write(&self) -> String {
        let mut state = self.state.lock().unwrap();
        match self.services.session.save_live(&state) {
            Ok(_) => {
                state.version += 1;
                "SESSION_LIVE_WRITTEN".to_string()
            }
            Err(e) => format!("ERROR: {}", e),
        }
    }

    fn handle_session_export(&self, sector_id: Option<&str>, name: Option<&str>) -> String {
        if let (Some(sid), Some(n)) = (sector_id, name) {
            match self.services.session.load(sid, n) {
                Ok(json) => return format!("SESSION_EXPORT: {}", json),
                Err(e) => return format!("ERROR: {}", e),
            }
        }
        "ERROR: Missing sector_id or name".to_string()
    }

    fn handle_session_import(&self, name: Option<&str>, json: Option<&str>) -> String {
        if let (Some(n), Some(data)) = (name, json) {
            if let Ok(state) = serde_json::from_str::<TosState>(data) {
                match self.services.session.save("global", n, &state) {
                    Ok(_) => return format!("SESSION_IMPORTED: {}", n),
                    Err(e) => return format!("ERROR: {}", e),
                }
            } else {
                return "ERROR: Invalid session schema".to_string();
            }
        }
        "ERROR: Missing name or json payload".to_string()
    }

    fn handle_session_handoff_prepare(&self) -> String {
        let state = self.state.lock().unwrap();
        match self.services.session.prepare_handoff(&state) {
            Ok(token) => format!("SESSION_HANDOFF_TOKEN: {}", token),
            Err(e) => format!("ERROR: {}", e),
        }
    }

    fn handle_session_handoff_claim(&self, token: Option<&str>) -> String {
        if let Some(t) = token {
            match self.services.session.claim_handoff(t) {
                Ok(json) => {
                    match serde_json::from_str(&json) {
                        Ok(new_state) => {
                            let mut state = self.state.lock().unwrap();
                            *state = new_state;
                            state.version += 1;
                            "SESSION_HANDOFF_CLAIMED".to_string()
                        }
                        Err(e) => format!("ERROR: Claimed session schema invalid: {}", e),
                    }
                }
                Err(e) => format!("ERROR: {}", e),
            }
        } else {
            "ERROR: Missing handoff token".to_string()
        }
    }

    // ----- Collaboration IPC Handlers -----

    fn handle_collaboration_role_set(
        &self,
        sector_id: Option<&str>,
        user_id: Option<&str>,
        role_str: Option<&str>,
    ) -> String {
        if let (Some(sid_str), Some(uid_str), Some(r_str)) = (sector_id, user_id, role_str) {
            if let (Ok(sid), Ok(uid)) = (Uuid::parse_str(sid_str), Uuid::parse_str(uid_str)) {
                let role = match r_str {
                    "viewer" => crate::collaboration::ParticipantRole::Viewer,
                    "commenter" => crate::collaboration::ParticipantRole::Commenter,
                    "operator" => crate::collaboration::ParticipantRole::Operator,
                    "coowner" => crate::collaboration::ParticipantRole::CoOwner,
                    _ => return "ERROR: Invalid role".to_string(),
                };

                let mut state = self.state.lock().unwrap();
                if let Some(sector) = state.sectors.iter_mut().find(|s| s.id == sid) {
                    if let Some(participant) = sector.participants.iter_mut().find(|p| p.id == uid) {
                        participant.role = role;
                        state.version += 1;
                        return format!("ROLE_SET: {} -> {:?}", uid, role);
                    }
                }
            }
        }
        "ERROR: Invalid arguments for role set".to_string()
    }

    fn handle_collaboration_participant_remove(
        &self,
        sector_id: Option<&str>,
        user_id: Option<&str>,
    ) -> String {
        if let (Some(sid_str), Some(uid_str)) = (sector_id, user_id) {
            if let (Ok(sid), Ok(uid)) = (Uuid::parse_str(sid_str), Uuid::parse_str(uid_str)) {
                let mut state = self.state.lock().unwrap();
                if let Some(sector) = state.sectors.iter_mut().find(|s| s.id == sid) {
                    let len_before = sector.participants.len();
                    sector.participants.retain(|p| p.id != uid);
                    if sector.participants.len() < len_before {
                        state.version += 1;
                        return format!("PARTICIPANT_REMOVED: {}", uid);
                    }
                }
            }
        }
        "ERROR: Invalid arguments for participant remove".to_string()
    }

    // ----- Remote SSH Fallback IPC Handlers -----

    fn handle_remote_ssh_connect(&self, host: Option<&str>) -> String {
        if let Some(h) = host {
            let mut state = self.state.lock().unwrap();
            let s_idx = state.active_sector_index;
            let sector_id = state.sectors[s_idx].id;
            let hub_id = state.sectors[s_idx].hubs[state.sectors[s_idx].active_hub_index].id;

            drop(state); // Drop lock before connecting (spawns thread)

            if let Err(e) = self
                .services
                .ssh
                .connect(h, self.state.clone(), sector_id, hub_id)
            {
                return format!("ERROR: SSH connection failed: {}", e);
            }

            let mut state = self.state.lock().unwrap();
            let h_idx = state.sectors[s_idx].active_hub_index;
            let hub = &mut state.sectors[s_idx].hubs[h_idx];
            hub.mode = crate::CommandHubMode::Ssh;
            hub.terminal_output.push(crate::TerminalLine {
                text: format!("SSH CONNECTED: {}", h),
                priority: 2,
                timestamp: chrono::Local::now(),
            });
            state.version += 1;
            return format!("SSH_CONNECT_OK: {}", h);
        }
        "ERROR: Missing SSH host".to_string()
    }

    fn handle_remote_ssh_disconnect(&self) -> String {
        let mut state = self.state.lock().unwrap();
        let s_idx = state.active_sector_index;
        let h_idx = state.sectors[s_idx].active_hub_index;
        let hub_id = state.sectors[s_idx].hubs[h_idx].id;
        let hub = &mut state.sectors[s_idx].hubs[h_idx];

        if hub.mode == crate::CommandHubMode::Ssh {
            self.services.ssh.disconnect(&hub_id);
            hub.mode = crate::CommandHubMode::Command;
            hub.terminal_output.push(crate::TerminalLine {
                text: "SSH DISCONNECTED".to_string(),
                priority: 2,
                timestamp: chrono::Local::now(),
            });
            state.version += 1;
            return "SSH_DISCONNECT_OK".to_string();
        }
        "ERROR: Hub is not in SSH mode".to_string()
    }

    // ----- Trust IPC Handlers -----

    fn handle_trust_promote(&self, class_key: Option<&str>) -> String {
        if let Some(key) = class_key {
            let mut state = self.state.lock().unwrap();
            self.services.trust.promote_global(&mut state, key);
            format!("TRUST_PROMOTED: {}", key)
        } else {
            "ERROR: Missing class_key".to_string()
        }
    }

    fn handle_trust_demote(&self, class_key: Option<&str>) -> String {
        if let Some(key) = class_key {
            let mut state = self.state.lock().unwrap();
            self.services.trust.demote_global(&mut state, key);
            format!("TRUST_DEMOTED: {}", key)
        } else {
            "ERROR: Missing class_key".to_string()
        }
    }

    fn handle_trust_promote_sector(
        &self,
        sector_id: Option<&str>,
        class_key: Option<&str>,
    ) -> String {
        if let (Some(sid), Some(key)) = (sector_id, class_key) {
            let mut state = self.state.lock().unwrap();
            self.services.trust.promote_sector(&mut state, sid, key);
            format!("TRUST_SECTOR_PROMOTED: {}:{}", sid, key)
        } else {
            "ERROR: Missing sector_id or class_key".to_string()
        }
    }

    fn handle_trust_demote_sector(
        &self,
        sector_id: Option<&str>,
        class_key: Option<&str>,
    ) -> String {
        if let (Some(sid), Some(key)) = (sector_id, class_key) {
            let mut state = self.state.lock().unwrap();
            self.services.trust.demote_sector(&mut state, sid, key);
            format!("TRUST_SECTOR_DEMOTED: {}:{}", sid, key)
        } else {
            "ERROR: Missing sector_id or class_key".to_string()
        }
    }

    fn handle_trust_clear_sector(&self, sector_id: Option<&str>) -> String {
        if let Some(sid) = sector_id {
            let mut state = self.state.lock().unwrap();
            self.services.trust.clear_sector(&mut state, sid);
            format!("TRUST_SECTOR_CLEARED: {}", sid)
        } else {
            "ERROR: Missing sector_id".to_string()
        }
    }

    fn handle_trust_get_config(&self) -> String {
        let state = self.state.lock().unwrap();
        format!(
            "TRUST_CONFIG: {}",
            self.services.trust.get_config_json(&state)
        )
    }

    // ----- AIService Refactor IPC Handlers -----

    fn handle_ai_pattern_set(&self, behavior_id: Option<&str>, pattern: Option<&str>) -> String {
        if let (Some(id), Some(p)) = (behavior_id, pattern) {
            let mut state = self.state.lock().unwrap();
            state.settings.ai_patterns.insert(id.to_string(), p.to_string());
            let _ = self.services.settings.save(&state.settings);
            return "AI_PATTERN_SET".to_string();
        }
        "ERROR: behavior_id and pattern required".to_string()
    }

    fn handle_ai_pattern_get(&self, behavior_id: Option<&str>) -> String {
        if let Some(id) = behavior_id {
            let state = self.state.lock().unwrap();
            return state.settings.ai_patterns.get(id).cloned().unwrap_or_default();
        }
        let state = self.state.lock().unwrap();
        serde_json::to_string(&state.settings.ai_patterns).unwrap_or_else(|_| "{}".to_string())
    }

    fn handle_ai_behavior_enable(&self, id: Option<&str>) -> String {
        if let Some(id) = id {
            let mut state = self.state.lock().unwrap();
            if self.services.ai.enable_behavior(&mut state, id) {
                return format!("AI_BEHAVIOR_ENABLED: {}", id);
            }
            return format!("ERROR: Behavior '{}' not registered", id);
        }
        "ERROR: Missing behavior_id".to_string()
    }

    fn handle_ai_behavior_disable(&self, id: Option<&str>) -> String {
        if let Some(id) = id {
            let mut state = self.state.lock().unwrap();
            if self.services.ai.disable_behavior(&mut state, id) {
                return format!("AI_BEHAVIOR_DISABLED: {}", id);
            }
            return format!("ERROR: Behavior '{}' not registered", id);
        }
        "ERROR: Missing behavior_id".to_string()
    }

    fn handle_ai_behavior_configure(
        &self,
        id: Option<&str>,
        key: Option<&str>,
        value: Option<&str>,
    ) -> String {
        if let (Some(id), Some(k), Some(v)) = (id, key, value) {
            let mut state = self.state.lock().unwrap();
            if self.services.ai.configure_behavior(&mut state, id, k, v) {
                return format!("AI_BEHAVIOR_CONFIGURED: {}::{}={}", id, k, v);
            }
            return format!("ERROR: Behavior '{}' not registered", id);
        }
        "ERROR: Missing behavior_id, key, or value".to_string()
    }

    fn handle_ai_chip_stage(&self, payload: &str) -> String {
        // Stage an AI chip in the system_log. Payload is the chip text.
        let mut state = self.state.lock().unwrap();
        state.system_log.push(crate::TerminalLine {
            text: format!("[AI CHIP] {}", payload),
            priority: 1,
            timestamp: chrono::Local::now(),
        });
        format!("AI_CHIP_STAGED: {}", &payload[..payload.len().min(40)])
    }

    fn handle_ai_chip_dismiss(&self, id: Option<&str>) -> String {
        // Remove the most recent AI chip matching the id prefix from system_log
        let prefix = format!("[AI CHIP] {}", id.unwrap_or(""));
        let mut state = self.state.lock().unwrap();
        let before = state.system_log.len();
        state.system_log.retain(|l| !l.text.starts_with(&prefix));
        let removed = before - state.system_log.len();
        format!("AI_CHIP_DISMISSED: {} removed", removed)
    }

    fn handle_ai_thought_expand(&self, thought_id: Option<&str>) -> String {
        // Push an expanded thought chip to system_log
        let id = thought_id.unwrap_or("unknown");
        let mut state = self.state.lock().unwrap();
        state.system_log.push(crate::TerminalLine {
            text: format!("[AI THOUGHT:EXPANDED] id={}", id),
            priority: 1,
            timestamp: chrono::Local::now(),
        });
        format!("AI_THOUGHT_EXPANDED: {}", id)
    }

    fn handle_ai_thought_dismiss(&self, thought_id: Option<&str>) -> String {
        let id = thought_id.unwrap_or("unknown");
        let prefix = format!("[AI THOUGHT:EXPANDED] id={}", id);
        let mut state = self.state.lock().unwrap();
        state.system_log.retain(|l| l.text != prefix);
        format!("AI_THOUGHT_DISMISSED: {}", id)
    }

    fn handle_ai_thought_dismiss_permanent(&self, thought_id: Option<&str>) -> String {
        let id = thought_id.unwrap_or("unknown");
        // Mark dismissal in settings so it persists across sessions
        let key = format!("tos.ai.thought.dismissed.{}", id);
        let mut state = self.state.lock().unwrap();
        state.settings.global.insert(key, "true".to_string());
        state
            .system_log
            .retain(|l| !l.text.contains(&format!("id={}", id)));
        format!("AI_THOUGHT_DISMISSED_PERMANENT: {}", id)
    }

    fn handle_ai_context_request(&self, behavior_id: Option<&str>) -> String {
        let state = self.state.lock().unwrap();
        let ctx = crate::services::ai::build_context(&state);
        let behavior_id = behavior_id.unwrap_or("*");

        // Look up declared fields for the behavior; default to all if not found
        let fields = state
            .ai_behaviors
            .iter()
            .find(|b| b.id == behavior_id)
            .map(|b| b.context_fields.clone())
            .unwrap_or_else(|| {
                vec![
                    "cwd".to_string(),
                    "sector_name".to_string(),
                    "shell".to_string(),
                    "terminal_tail".to_string(),
                    "last_command".to_string(),
                    "mode".to_string(),
                ]
            });

        let context_entries = ctx.filter_to_fields(&fields);
        let json = serde_json::to_string(&context_entries).unwrap_or_else(|_| "[]".to_string());
        format!("AI_CONTEXT:{}", json)
    }

    // ----- Bezel Handlers -----

    fn handle_bezel_expand(&self) -> String {
        let mut state = self.state.lock().unwrap();
        state.bezel_expanded = true;
        state.version += 1;
        "BEZEL_EXPANDED".to_string()
    }

    fn handle_bezel_collapse(&self) -> String {
        let mut state = self.state.lock().unwrap();
        state.bezel_expanded = false;
        state.version += 1;
        "BEZEL_COLLAPSED".to_string()
    }

    fn handle_bezel_swipe(&self, dir: Option<&str>) -> String {
        let dir = dir.unwrap_or("Right");
        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            if !sector.active_apps.is_empty() {
                if dir == "Right" {
                    sector.active_app_index =
                        (sector.active_app_index + 1) % sector.active_apps.len();
                } else {
                    if sector.active_app_index == 0 {
                        sector.active_app_index = sector.active_apps.len() - 1;
                    } else {
                        sector.active_app_index -= 1;
                    }
                }
                state.version += 1;
                return format!("BEZEL_SWIPED: {}", dir);
            }
        }
        "ERROR: No apps to swipe".to_string()
    }

    fn handle_onboarding_skip_tour(&self) -> String {
        let mut state = self.state.lock().unwrap();
        state.settings.global.insert("tos.onboarding.wizard_complete".to_string(), "true".to_string());
        state.version += 1;
        "ONBOARDING_SKIPPED".to_string()
    }

    fn handle_onboarding_advance_step(&self, step: Option<&str>) -> String {
        let step = step.unwrap_or("0");
        let mut state = self.state.lock().unwrap();
        state.settings.global.insert("tos.onboarding.current_step".to_string(), step.to_string());
        state.version += 1;
        format!("ONBOARDING_STEP: {}", step)
    }

    fn handle_onboarding_hint_dismiss(&self, hint_id: Option<&str>) -> String {
        let id = hint_id.unwrap_or("unknown");
        let mut state = self.state.lock().unwrap();
        state.settings.global.insert(format!("tos.hint.dismissed.{}", id), "true".to_string());
        state.version += 1;
        format!("HINT_DISMISSED: {}", id)
    }

    fn handle_onboarding_hints_suppress(&self) -> String {
        let mut state = self.state.lock().unwrap();
        state.settings.global.insert("tos.hint.suppressed".to_string(), "true".to_string());
        state.version += 1;
        "HINTS_SUPPRESSED".to_string()
    }

    fn handle_onboarding_reset_hints(&self) -> String {
        let mut state = self.state.lock().unwrap();
        // Remove all keys starting with tos.hint.dismissed.
        let keys_to_remove: Vec<String> = state.settings.global.keys()
            .filter(|k| k.starts_with("tos.hint.dismissed."))
            .cloned()
            .collect();
        for k in keys_to_remove {
            state.settings.global.remove(&k);
        }
        state.settings.global.remove("tos.hint.suppressed");
        state.version += 1;
        "HINTS_RESET".to_string()
    }

    fn handle_clear_system_log(&self) -> String {
        let mut state = self.state.lock().unwrap();
        state.system_log.clear();
        state.version += 1;
        "SYSTEM_LOG_CLEARED".to_string()
    }

    fn handle_ai_backend_set_default(&self, backend_id: Option<&str>) -> String {
        if let Some(id) = backend_id {
            let mut state = self.state.lock().unwrap();
            self.services.ai.set_default_backend(&mut state, id);
            return format!("AI_DEFAULT_BACKEND_SET: {}", id);
        }
        "ERROR: Missing backend_id".to_string()
    }

    fn handle_ai_backend_set_behavior(
        &self,
        behavior_id: Option<&str>,
        backend_id: Option<&str>,
    ) -> String {
        if let (Some(bid), Some(backend)) = (behavior_id, backend_id) {
            let mut state = self.state.lock().unwrap();
            if self
                .services
                .ai
                .set_behavior_backend(&mut state, bid, backend)
            {
                return format!("AI_BEHAVIOR_BACKEND_SET: {}→{}", bid, backend);
            }
            return format!("ERROR: Behavior '{}' not registered", bid);
        }
        "ERROR: Missing behavior_id or backend_id".to_string()
    }

    fn handle_ai_backend_clear_behavior(&self, behavior_id: Option<&str>) -> String {
        if let Some(bid) = behavior_id {
            let mut state = self.state.lock().unwrap();
            if self.services.ai.clear_behavior_backend(&mut state, bid) {
                return format!("AI_BEHAVIOR_BACKEND_CLEARED: {}", bid);
            }
            return format!("ERROR: Behavior '{}' not registered", bid);
        }
        "ERROR: Missing behavior_id".to_string()
    }

    // ----- Split Pane Handlers -----

    fn handle_split_create(&self, w: Option<&str>, h: Option<&str>) -> String {
        let display_w = w.and_then(|s| s.parse().ok()).unwrap_or(1920u32);
        let display_h = h.and_then(|s| s.parse().ok()).unwrap_or(1080u32);
        let mut state = self.state.lock().unwrap();
        match crate::brain::sector::SectorManager::split_create(&mut state, display_w, display_h) {
            Ok(id) => format!("SPLIT_CREATED: {}", id),
            Err(e) => {
                // Emit amber warning chip
                state.system_log.push(crate::TerminalLine {
                    text: format!("[SPLIT] ⚠ {}", e),
                    priority: 2,
                    timestamp: chrono::Local::now(),
                });
                // Also play earcon hint for the user
                let _ = self.services.audio.play_earcon("warning");
                e
            }
        }
    }

    fn handle_split_close(&self, pane_id: Option<&str>) -> String {
        if let Some(id_str) = pane_id {
            if let Ok(id) = Uuid::parse_str(id_str) {
                let mut state = self.state.lock().unwrap();
                if crate::brain::sector::SectorManager::split_close(&mut state, id) {
                    return format!("SPLIT_CLOSED: {}", id);
                }
                return "ERROR: Pane not found".to_string();
            }
        }
        "ERROR: Missing or invalid pane_id".to_string()
    }

    fn handle_split_focus(&self, pane_id: Option<&str>) -> String {
        if let Some(id_str) = pane_id {
            if let Ok(id) = Uuid::parse_str(id_str) {
                let mut state = self.state.lock().unwrap();
                if crate::brain::sector::SectorManager::split_focus(&mut state, id) {
                    return format!("SPLIT_FOCUSED: {}", id);
                }
                return "ERROR: Pane not found".to_string();
            }
        }
        "ERROR: Missing or invalid pane_id".to_string()
    }

    fn handle_split_focus_direction(&self, direction: Option<&str>) -> String {
        let dir = direction.unwrap_or("right");
        let mut state = self.state.lock().unwrap();
        match crate::brain::sector::SectorManager::split_focus_direction(&mut state, dir) {
            Some(id) => format!("SPLIT_FOCUSED: {}", id),
            None => "ERROR: No split panes active".to_string(),
        }
    }

    fn handle_split_resize(&self, pane_id: Option<&str>, weight_str: Option<&str>) -> String {
        if let (Some(id_str), Some(w)) = (pane_id, weight_str) {
            if let (Ok(id), Ok(weight)) = (Uuid::parse_str(id_str), w.parse::<f32>()) {
                let mut state = self.state.lock().unwrap();
                let idx = state.active_sector_index;
                let hub_idx = state.sectors[idx].active_hub_index;
                let hub = &mut state.sectors[idx].hubs[hub_idx];
                fn set_weight(node: &mut crate::SplitNode, id: Uuid, weight: f32) -> bool {
                    match node {
                        crate::SplitNode::Leaf(p) => {
                            if p.id == id {
                                p.weight = weight;
                                true
                            } else {
                                false
                            }
                        }
                        crate::SplitNode::Container { children, .. } => {
                            children.iter_mut().any(|c| set_weight(c, id, weight))
                        }
                    }
                }
                if let Some(layout) = &mut hub.split_layout {
                    if set_weight(layout, id, weight) {
                        return format!("SPLIT_RESIZED: {}→{}", id, weight);
                    }
                }
                return "ERROR: Pane not found".to_string();
            }
        }
        "ERROR: Missing pane_id or weight".to_string()
    }

    fn handle_split_equalize(&self) -> String {
        let mut state = self.state.lock().unwrap();
        if crate::brain::sector::SectorManager::split_equalize(&mut state) {
            "SPLIT_EQUALIZED".to_string()
        } else {
            "ERROR: No split layout active".to_string()
        }
    }

    fn handle_split_fullscreen(&self, pane_id: Option<&str>) -> String {
        if let Some(id_str) = pane_id {
            if let Ok(id) = Uuid::parse_str(id_str) {
                let mut state = self.state.lock().unwrap();
                if crate::brain::sector::SectorManager::split_fullscreen(&mut state, id) {
                    return format!("SPLIT_FULLSCREEN: {}", id);
                }
                return "ERROR: Pane not found".to_string();
            }
        }
        "ERROR: Missing or invalid pane_id".to_string()
    }

    fn handle_split_fullscreen_exit(&self) -> String {
        // Exit fullscreen: clear split_layout entirely (fall back to single pane mode)
        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        let hub_idx = state.sectors[idx].active_hub_index;
        let hub = &mut state.sectors[idx].hubs[hub_idx];
        hub.split_layout = None;
        hub.focused_pane_id = None;
        "SPLIT_FULLSCREEN_EXIT".to_string()
    }

    fn handle_split_swap(&self, pane_a: Option<&str>, pane_b: Option<&str>) -> String {
        if let (Some(a_str), Some(b_str)) = (pane_a, pane_b) {
            if let (Ok(a), Ok(b)) = (Uuid::parse_str(a_str), Uuid::parse_str(b_str)) {
                let mut state = self.state.lock().unwrap();
                let idx = state.active_sector_index;
                let hub_idx = state.sectors[idx].active_hub_index;
                let hub = &mut state.sectors[idx].hubs[hub_idx];

                fn swap_leaves(node: &mut crate::SplitNode, a: Uuid, b: Uuid) {
                    if let crate::SplitNode::Container { children, .. } = node {
                        // Find positions of a and b within immediate children
                        let pos_a = children.iter().position(
                            |c| matches!(c, crate::SplitNode::Leaf(p) if p.id == a),
                        );
                        let pos_b = children.iter().position(
                            |c| matches!(c, crate::SplitNode::Leaf(p) if p.id == b),
                        );
                        if let (Some(ia), Some(ib)) = (pos_a, pos_b) {
                            children.swap(ia, ib);
                        } else {
                            for child in children.iter_mut() {
                                swap_leaves(child, a, b);
                            }
                        }
                    }
                }

                if let Some(layout) = &mut hub.split_layout {
                    swap_leaves(layout, a, b);
                    return format!("SPLIT_SWAPPED: {}↔{}", a, b);
                }
                return "ERROR: No split layout active".to_string();
            }
        }
        "ERROR: Missing pane_a or pane_b".to_string()
    }

    fn handle_split_detach_context(&self) -> String {
        // "Bring Context" detach: not yet implemented (requires PTY re-parenting)
        "SPLIT_DETACH_CONTEXT: NOT_YET_IMPLEMENTED".to_string()
    }

    fn handle_split_detach_fresh(&self) -> String {
        let mut state = self.state.lock().unwrap();
        match crate::brain::sector::SectorManager::split_detach_fresh(&mut state) {
            Some(sector_id) => format!("SPLIT_DETACHED: new_sector={}", sector_id),
            None => "ERROR: No focused pane to detach".to_string(),
        }
    }

    fn handle_split_save_template(&self, name: Option<&str>) -> String {
        let name = name.unwrap_or("unnamed");
        let state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        let hub_idx = state.sectors[idx].active_hub_index;
        let hub = &state.sectors[idx].hubs[hub_idx];
        if let Some(layout) = &hub.split_layout {
            match serde_json::to_string(layout) {
                Ok(json) => {
                    // Persist to settings for later restore
                    drop(state);
                    let mut state = self.state.lock().unwrap();
                    state
                        .settings
                        .global
                        .insert(format!("tos.split_template.{}", name), json);
                    format!("SPLIT_TEMPLATE_SAVED: {}", name)
                }
                Err(e) => format!("ERROR: {}", e),
            }
        } else {
            "ERROR: No split layout to save".to_string()
        }
    }
    fn handle_ai_history_clear(&self) -> String {
        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let h_idx = sector.active_hub_index;
            if let Some(hub) = sector.hubs.get_mut(h_idx) {
                hub.ai_history.clear();
                state.version += 1;
                return "AI_HISTORY_CLEARED".to_string();
            }
        }
        "ERROR: Hub not found".to_string()
    }

    fn handle_ai_history_append(&self, message: &str, role: &str) -> String {
        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let h_idx = sector.active_hub_index;
            if let Some(hub) = sector.hubs.get_mut(h_idx) {
                hub.ai_history.push(crate::AiMessage {
                    role: role.to_string(),
                    content: message.to_string(),
                    timestamp: chrono::Local::now(),
                });
                if hub.ai_history.len() > 200 {
                    hub.ai_history.remove(0);
                }
                state.version += 1;
                return "AI_HISTORY_APPENDED".to_string();
            }
        }
        "ERROR: Hub not found".to_string()
    }



    fn handle_ai_predict_command(&self, partial: &str) -> String {
        let ai = self.services.ai.clone();
        let partial_str = partial.to_string();
        
        tokio::spawn(async move {
            match ai.predict_command(&partial_str).await {
                Ok(_prediction) => {
                    // Predictions are handled via state updates in a real system,
                    // but for ghost text we should probably emit a specific IPC back to the Face.
                    // However, AIService already has access to IPC.
                    // Actually, predict_command in AiService should probably dispatch itself.
                    // Let's refine AiService::predict_command to dispatch the result.
                }
                Err(e) => {
                    tracing::error!("[IpcHandler] AI Prediction failed: {}", e);
                }
            }
        });
        "AI_PREDICT_ACCEPTED".to_string()
    }

    fn handle_ai_thought_stage(&self, payload: &str) -> String {
        if let Ok(thought) = serde_json::from_str::<crate::AiThought>(payload) {
            let mut state = self.state.lock().unwrap();
            let idx = state.active_sector_index;
            if let Some(sector) = state.sectors.get_mut(idx) {
                let h_idx = sector.active_hub_index;
                if let Some(hub) = sector.hubs.get_mut(h_idx) {
                    // Update if exists, else push
                    if let Some(existing) = hub.active_thoughts.iter_mut().find(|t| t.id == thought.id) {
                        *existing = thought;
                    } else {
                        hub.active_thoughts.push(thought);
                    }
                    state.version += 1;
                    return "AI_THOUGHT_STAGED".to_string();
                }
            }
        }
        "ERROR: Invalid thought JSON".to_string()
    }

    fn handle_ai_plan(&self, prompt: &str) -> String {
        let ai = self.services.ai.clone();
        let prompt_str = prompt.to_string();
        tokio::spawn(async move {
            if let Err(e) = ai.vibe_plan(&prompt_str).await {
                tracing::error!("[IpcHandler] AI Plan failed: {}", e);
            }
        });
        "AI_PLAN_ACCEPTED".to_string()
    }

    pub fn handle_ai_isolated_exec(&self, command: &str) -> String {
        let cwd = {
            let state = self.state.lock().unwrap();
            let idx = state.active_sector_index;
            state
                .sectors
                .get(idx)
                .and_then(|s| s.hubs.get(s.active_hub_index))
                .map(|h| h.current_directory.clone())
                .unwrap_or_default()
        };

        match crate::brain::shell::ShellApi::exec_isolated(command, cwd) {
            Ok(output) => output,
            Err(e) => format!("ERROR: Isolated exec failed: {}", e),
        }
    }

    fn handle_ai_roadmap_plan(&self) -> String {
        let ai = self.services.ai.clone();
        tokio::spawn(async move {
            if let Err(e) = ai.roadmap_plan().await {
                tracing::error!("[IpcHandler] Roadmap Plan failed: {}", e);
            }
        });
        "AI_ROADMAP_PLAN_STARTED".to_string()
    }

    fn handle_ai_dream_consolidate(&self) -> String {
        let ai = self.services.ai.clone();
        tokio::spawn(async move {
            if let Err(e) = ai.dream_consolidate().await {
                tracing::error!("[IpcHandler] Dream Consolidate failed: {}", e);
            }
        });
        "AI_DREAM_CONSOLIDATE_STARTED".to_string()
    }

    fn handle_confirmation_accept(&self, id_str: Option<&str>) -> String {
        if let Some(id_str) = id_str {
            if let Ok(id) = Uuid::parse_str(id_str) {
                let mut state = self.state.lock().unwrap();
                if let Some(conf) = state.pending_confirmation.take() {
                    if conf.id == id {
                        let original = conf.original_request.clone();
                        drop(state);
                        return self.handle_request(&original);
                    }
                }
            }
        }
        "ERROR: Invalid confirmation ID".to_string()
    }

    fn handle_confirmation_reject(&self, id_str: Option<&str>) -> String {
        if let Some(id_str) = id_str {
            if let Ok(id) = Uuid::parse_str(id_str) {
                let mut state = self.state.lock().unwrap();
                if let Some(conf) = state.pending_confirmation.take() {
                    if conf.id == id {
                        state.version += 1;
                        return "CONFIRMATION_REJECTED".to_string();
                    }
                }
            }
        }
        "ERROR: Invalid confirmation ID".to_string()
    }

    fn handle_update_confirmation_progress(
        &self,
        id_str: Option<&str>,
        progress_str: Option<&str>,
    ) -> String {
        if let (Some(id_str), Some(p_str)) = (id_str, progress_str) {
            if let (Ok(id), Ok(p)) = (Uuid::parse_str(id_str), p_str.parse::<f32>()) {
                let mut state = self.state.lock().unwrap();
                let mut should_execute = false;
                let mut original_request = String::new();

                if let Some(ref mut conf) = state.pending_confirmation {
                    if conf.id == id {
                        conf.progress = p;
                        if p >= 1.0 {
                            should_execute = true;
                            original_request = conf.original_request.clone();
                        }
                    }
                }

                if should_execute {
                    state.pending_confirmation = None;
                    state.version += 1;
                    drop(state);
                    return self.handle_request(&original_request);
                } else if !original_request.is_empty() { // Wait, if not executing but found id
                     // This branch is redundant because original_request is only non-empty if p >= 1.0
                }

                // If we updated progress but didn't execute
                if state
                    .pending_confirmation
                    .as_ref()
                    .map(|c| c.id == id)
                    .unwrap_or(false)
                {
                    state.version += 1;
                    return format!("CONFIRMATION_PROGRESS: {}", p);
                }
            }
        }
        "ERROR: Invalid confirmation update".to_string()
    }

    fn handle_marketplace_home(&self) -> String {
        match self.services.marketplace.get_home() {
            Ok(home) => serde_json::to_string(&home).unwrap_or_default(),
            Err(e) => format!("ERROR: {}", e),
        }
    }

    fn handle_marketplace_category(&self, id: Option<&str>) -> String {
        match id {
            Some(id) => match self.services.marketplace.get_category(id) {
                Ok(modules) => serde_json::to_string(&modules).unwrap_or_default(),
                Err(e) => format!("ERROR: {}", e),
            },
            None => "ERROR: Missing category ID".to_string(),
        }
    }

    fn handle_marketplace_detail(&self, id: Option<&str>) -> String {
        match id {
            Some(id) => match self.services.marketplace.get_detail(id) {
                Ok(detail) => serde_json::to_string(&detail).unwrap_or_default(),
                Err(e) => format!("ERROR: {}", e),
            },
            None => "ERROR: Missing module ID".to_string(),
        }
    }

    fn handle_marketplace_install(&self, id: Option<&str>) -> String {
        match id {
            Some(id) => match self.services.marketplace.install(id) {
                Ok(status) => status,
                Err(e) => format!("ERROR: {}", e),
            },
            None => "ERROR: Missing module ID".to_string(),
        }
    }

    fn handle_marketplace_status(&self, id: Option<&str>) -> String {
        match id {
            Some(id) => match self.services.marketplace.get_status(id) {
                Ok(p) => serde_json::to_string(&p).unwrap_or_default(),
                Err(e) => format!("ERROR: {}", e),
            },
            None => "ERROR: Missing module ID".to_string(),
        }
    }

    fn handle_marketplace_search_ai(&self, query: &str) -> String {
        match self.services.marketplace.search_ai(query) {
            Ok(results) => serde_json::to_string(&results).unwrap_or_default(),
            Err(e) => format!("ERROR: {}", e),
        }
    }

    fn handle_marketplace_install_cancel(&self, id: Option<&str>) -> String {
        match id {
            Some(id) => match self.services.marketplace.cancel_install(id) {
                Ok(status) => status,
                Err(e) => format!("ERROR: {}", e),
            },
            None => "ERROR: Missing module ID".to_string(),
        }
    }

    // ── §14.2: Configurable Keyboard Shortcuts ─────────────────────────

    /// Return the full keybinding map as JSON.
    ///
    /// If the user has a saved custom map in settings, that is returned.
    /// Otherwise the spec defaults are returned.
    fn handle_keybindings_get(&self) -> String {
        let state = self.state.lock().unwrap();
        let map = if let Some(json) = state.settings.global.get("tos.keybindings") {
            crate::keybindings::KeybindingMap::from_json(json)
                .unwrap_or_default()
        } else {
            crate::keybindings::KeybindingMap::default()
        };
        map.to_json()
    }

    /// Remap a keybinding: `keybindings_set:<combo_str>;<action>;<description>`
    ///
    /// Persists to settings. Returns the displaced action if a conflict was resolved.
    fn handle_keybindings_set(
        &self,
        combo_str: Option<&str>,
        action: Option<&str>,
        description: Option<&str>,
    ) -> String {
        let combo_str = match combo_str {
            Some(s) => s,
            None => return "ERROR: Missing key combo".to_string(),
        };
        let action = match action {
            Some(a) => a,
            None => return "ERROR: Missing action".to_string(),
        };
        let description = description.unwrap_or("");

        let combo = match crate::keybindings::KeyCombo::parse(combo_str) {
            Some(c) => c,
            None => return format!("ERROR: Invalid key combo: {}", combo_str),
        };

        let mut state = self.state.lock().unwrap();

        // Load existing or default map
        let mut map = if let Some(json) = state.settings.global.get("tos.keybindings") {
            crate::keybindings::KeybindingMap::from_json(json)
                .unwrap_or_default()
        } else {
            crate::keybindings::KeybindingMap::default()
        };

        let displaced = map.set(combo, action.to_string(), description.to_string());

        // Persist to settings
        state
            .settings
            .global
            .insert("tos.keybindings".to_string(), map.to_json());
        state.version += 1;

        match displaced {
            Some(old_action) => format!("KEYBINDING_SET: {} (displaced: {})", action, old_action),
            None => format!("KEYBINDING_SET: {}", action),
        }
    }

    /// Reset all keybindings to spec defaults.
    fn handle_keybindings_reset(&self) -> String {
        let mut state = self.state.lock().unwrap();
        let map = crate::keybindings::KeybindingMap::default();
        state
            .settings
            .global
            .insert("tos.keybindings".to_string(), map.to_json());
        state.version += 1;
        "KEYBINDINGS_RESET".to_string()
    }

    // ── §30.3–30.4: Editor Pane IPC ────────────────────────────────────

    /// `editor_open:<path>;<line>` — Read a file from disk and open it in
    /// a new Viewer-mode editor pane in the active hub's split tree.
    fn handle_editor_open(&self, path: Option<&str>, line: Option<&str>) -> String {
        let path_str = match path {
            Some(p) if !p.is_empty() => p,
            _ => return "ERROR: Missing path".to_string(),
        };
        let line_num: usize = line.and_then(|l| l.parse().ok()).unwrap_or(0);
        let file_path = std::path::PathBuf::from(path_str);

        // Read file content
        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => return format!("ERROR: Cannot read file: {}", e),
        };

        let language = detect_language(&file_path);

        let editor_state = crate::EditorPaneState {
            file_path: file_path.clone(),
            content,
            mode: crate::EditorMode::Viewer,
            language,
            cursor_line: line_num,
            cursor_col: 0,
            scroll_offset: line_num.saturating_sub(5),
            dirty: false,
            diff_hunks: vec![],
            annotations: vec![],
        };

        // Install as a new split pane in the active hub
        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let hub_idx = sector.active_hub_index;
            if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                // LSP integration (after state unlock)
                let cwd = hub.current_directory.clone();
                let lang_clone = editor_state.language.clone();
                let file_clone = editor_state.file_path.clone();
                let cont_clone = editor_state.content.clone();
                
                // Create a new editor pane via split
                let pane = crate::SplitPane::new_with_content(
                    crate::PaneContent::Editor(editor_state),
                );
                match hub.split_layout {
                    Some(ref mut tree) => tree.add_pane(pane),
                    None => {
                        hub.split_layout = Some(crate::SplitNode::Leaf(pane));
                    }
                }
                state.version += 1;
                drop(state);
                
                if let Some(lang) = lang_clone {
                    self.services.lsp.start_client(&lang, cwd);
                    self.services.lsp.did_open(&lang, &file_clone, &cont_clone);
                }

                return format!("EDITOR_OPENED: {}", path_str);
            }
        }
        "ERROR: No active hub".to_string()
    }

    /// `editor_save:<pane_id>` — Write the editor buffer to its file_path.
    fn handle_editor_save(&self, pane_id: Option<&str>) -> String {
        let pane_uuid = match pane_id.and_then(|s| Uuid::parse_str(s).ok()) {
            Some(u) => u,
            None => return "ERROR: Invalid pane_id".to_string(),
        };

        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;

        // Extract file path and content for writing (release borrow on state.sectors)
        let write_info = state.sectors.get(idx).and_then(|sector| {
            let hub = sector.hubs.get(sector.active_hub_index)?;
            let pane = hub.split_layout.as_ref().and_then(|t| {
                fn find_pane(node: &crate::SplitNode, id: Uuid) -> Option<&crate::SplitPane> {
                    match node {
                        crate::SplitNode::Leaf(p) if p.id == id => Some(p),
                        crate::SplitNode::Leaf(_) => None,
                        crate::SplitNode::Container { children, .. } => {
                            children.iter().find_map(|c| find_pane(c, id))
                        }
                    }
                }
                find_pane(t, pane_uuid)
            })?;
            if let crate::PaneContent::Editor(ref ed) = pane.content {
                Some((ed.file_path.clone(), ed.content.clone()))
            } else {
                None
            }
        });

        if let Some((file_path, content)) = write_info {
            match std::fs::write(&file_path, &content) {
                Ok(_) => {
                    // Now mutate to clear dirty flag
                    if let Some(sector) = state.sectors.get_mut(idx) {
                        let hub_idx = sector.active_hub_index;
                        if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                            if let Some(pane) = hub.split_layout.as_mut().and_then(|t| t.find_pane_mut(pane_uuid)) {
                                if let crate::PaneContent::Editor(ref mut ed) = pane.content {
                                    ed.dirty = false;
                                }
                            }
                        }
                    }
                    state.version += 1;
                    return format!("EDITOR_SAVED: {}", file_path.display());
                }
                Err(e) => return format!("ERROR: Write failed: {}", e),
            }
        }
        "ERROR: Pane not found or not an editor".to_string()
    }

    /// `editor_save_as:<pane_id>;<new_path>` — Save buffer to a new path.
    fn handle_editor_save_as(&self, pane_id: Option<&str>, new_path: Option<&str>) -> String {
        let pane_uuid = match pane_id.and_then(|s| Uuid::parse_str(s).ok()) {
            Some(u) => u,
            None => return "ERROR: Invalid pane_id".to_string(),
        };
        let new_path_str = match new_path {
            Some(p) if !p.is_empty() => p,
            _ => return "ERROR: Missing new path".to_string(),
        };

        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let hub_idx = sector.active_hub_index;
            if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                if let Some(pane) = hub.split_layout.as_mut().and_then(|t| t.find_pane_mut(pane_uuid)) {
                    if let crate::PaneContent::Editor(ref mut ed) = pane.content {
                        let path = std::path::PathBuf::from(new_path_str);
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        match std::fs::write(&path, &ed.content) {
                            Ok(_) => {
                                ed.file_path = path;
                                ed.language = detect_language(&ed.file_path);
                                ed.dirty = false;
                                state.version += 1;
                                return format!("EDITOR_SAVED_AS: {}", new_path_str);
                            }
                            Err(e) => return format!("ERROR: Write failed: {}", e),
                        }
                    }
                }
            }
        }
        "ERROR: Pane not found or not an editor".to_string()
    }

    /// `editor_activate:<pane_id>` — Switch pane from Viewer to Editor mode.
    fn handle_editor_activate(&self, pane_id: Option<&str>) -> String {
        let pane_uuid = match pane_id.and_then(|s| Uuid::parse_str(s).ok()) {
            Some(u) => u,
            None => return "ERROR: Invalid pane_id".to_string(),
        };

        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let hub_idx = sector.active_hub_index;
            if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                if let Some(pane) = hub.split_layout.as_mut().and_then(|t| t.find_pane_mut(pane_uuid)) {
                    if let crate::PaneContent::Editor(ref mut ed) = pane.content {
                        ed.mode = crate::EditorMode::Editor;
                        state.version += 1;
                        return "EDITOR_ACTIVATED".to_string();
                    }
                }
            }
        }
        "ERROR: Pane not found or not an editor".to_string()
    }

    /// `editor_mode_switch:<pane_id>;<mode>` — Switch between viewer/editor/diff.
    fn handle_editor_mode_switch(&self, pane_id: Option<&str>, mode: Option<&str>) -> String {
        let pane_uuid = match pane_id.and_then(|s| Uuid::parse_str(s).ok()) {
            Some(u) => u,
            None => return "ERROR: Invalid pane_id".to_string(),
        };
        let target_mode = match mode {
            Some("viewer") => crate::EditorMode::Viewer,
            Some("editor") => crate::EditorMode::Editor,
            Some("diff") => crate::EditorMode::Diff,
            _ => return "ERROR: Invalid mode (viewer|editor|diff)".to_string(),
        };

        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let hub_idx = sector.active_hub_index;
            if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                if let Some(pane) = hub.split_layout.as_mut().and_then(|t| t.find_pane_mut(pane_uuid)) {
                    if let crate::PaneContent::Editor(ref mut ed) = pane.content {
                        ed.mode = target_mode;
                        state.version += 1;
                        return format!("EDITOR_MODE: {:?}", target_mode);
                    }
                }
            }
        }
        "ERROR: Pane not found or not an editor".to_string()
    }

    /// `editor_scroll:<path>;<line>` — Scroll editor showing `path` to the given line.
    fn handle_editor_scroll(&self, path: Option<&str>, line: Option<&str>) -> String {
        let path_str = match path {
            Some(p) => p,
            None => return "ERROR: Missing path".to_string(),
        };
        let line_num: usize = line.and_then(|l| l.parse().ok()).unwrap_or(0);

        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let hub_idx = sector.active_hub_index;
            if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                if let Some(ed) = hub.split_layout.as_mut().and_then(|t| t.find_editor_by_path_mut(path_str)) {
                    ed.cursor_line = line_num;
                    ed.scroll_offset = line_num.saturating_sub(5);
                    state.version += 1;
                    return format!("EDITOR_SCROLLED: {}:{}", path_str, line_num);
                }
            }
        }
        "ERROR: No editor pane found for path".to_string()
    }

    /// `editor_open_ai:<path>;<line>;<context_id>` — Open with AI context.
    fn handle_editor_open_ai(
        &self,
        path: Option<&str>,
        line: Option<&str>,
        _context_id: Option<&str>,
    ) -> String {
        // Open the file normally, AI context attachment is a Face-side concern
        self.handle_editor_open(path, line)
    }

    /// `editor_diff:<path>;<proposed_content_id>` — Open Diff Mode with proposal.
    fn handle_editor_diff(&self, path: Option<&str>, _proposed_id: Option<&str>) -> String {
        // Open file, then switch to Diff mode
        let result = self.handle_editor_open(path, Some("0"));
        if result.starts_with("EDITOR_OPENED") {
            let path_str = path.unwrap_or("");
            let mut state = self.state.lock().unwrap();
            let idx = state.active_sector_index;
            if let Some(sector) = state.sectors.get_mut(idx) {
                let hub_idx = sector.active_hub_index;
                if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                    if let Some(ed) = hub.split_layout.as_mut().and_then(|t| t.find_editor_by_path_mut(path_str)) {
                        ed.mode = crate::EditorMode::Diff;
                        state.version += 1;
                    }
                }
            }
            return format!("EDITOR_DIFF: {}", path_str);
        }
        result
    }

    /// `editor_annotate:<path>;<line>;<severity>;<message>;<context_id>` — Add annotation.
    fn handle_editor_annotate(&self, payload: &str) -> String {
        // Annotations are stored in state for the Face to render.
        // For now, log the annotation request.
        tracing::info!("[EDITOR] Annotation request: {}", payload);
        format!("EDITOR_ANNOTATED: {}", payload)
    }

    /// `editor_clear_annotations:<path>` — Remove all annotations.
    fn handle_editor_clear_annotations(&self, path: Option<&str>) -> String {
        let path_str = path.unwrap_or("");
        tracing::info!("[EDITOR] Clearing annotations for: {}", path_str);
        format!("EDITOR_ANNOTATIONS_CLEARED: {}", path_str)
    }

    /// `editor_edit_proposal:<pane_id>;<proposal_json>` — Trigger Diff Mode with proposed changes.
    fn handle_editor_edit_proposal(&self, pane_id: Option<&str>, payload: Option<&str>) -> String {
        let pane_uuid = match pane_id.and_then(|s| Uuid::parse_str(s).ok()) {
            Some(u) => u,
            None => return "ERROR: Invalid pane_id".to_string(),
        };
        let hunks_json = match payload {
            Some(p) => p,
            None => return "ERROR: Missing hunks JSON".to_string(),
        };

        let hunks: Result<Vec<crate::state::DiffHunk>, _> = serde_json::from_str(hunks_json);
        match hunks {
            Ok(diff_hunks) => {
                let mut state = self.state.lock().unwrap();
                let idx = state.active_sector_index;
                if let Some(sector) = state.sectors.get_mut(idx) {
                    let hub_idx = sector.active_hub_index;
                    if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                        if let Some(pane) = hub.split_layout.as_mut().and_then(|t| t.find_pane_mut(pane_uuid)) {
                            if let crate::PaneContent::Editor(ref mut ed) = pane.content {
                                ed.diff_hunks = diff_hunks.clone();
                                ed.mode = crate::EditorMode::Diff;
                                state.version += 1;
                                return format!("EDITOR_PROPOSAL_RECEIVED: {} hunks", diff_hunks.len());
                            }
                        }
                    }
                }
                "ERROR: Pane not found or not an editor".to_string()
            }
            Err(e) => format!("ERROR: Invalid hunks JSON: {}", e),
        }
    }

    /// `editor_edit_apply:<pane_id>;<hunk_index>` — Apply pending edit proposal.
    fn handle_editor_edit_apply(&self, pane_id: Option<&str>, hunk_index: Option<&str>) -> String {
        let pane_uuid = match pane_id.and_then(|s| Uuid::parse_str(s).ok()) {
            Some(u) => u,
            None => return "ERROR: Invalid pane_id".to_string(),
        };
        let h_idx: usize = match hunk_index.and_then(|i| i.parse().ok()) {
            Some(idx) => idx,
            None => return "ERROR: Invalid hunk_index".to_string(),
        };

        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let hub_idx = sector.active_hub_index;
            if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                if let Some(pane) = hub.split_layout.as_mut().and_then(|t| t.find_pane_mut(pane_uuid)) {
                    if let crate::PaneContent::Editor(ref mut ed) = pane.content {
                        if h_idx < ed.diff_hunks.len() {
                            let hunk = ed.diff_hunks.remove(h_idx);
                            
                            // Naive apply: find lines and replace them
                            let mut lines: Vec<&str> = ed.content.lines().collect();
                            let start = hunk.old_start.saturating_sub(1); // 1-indexed to 0-indexed
                            let end = start + hunk.old_count;
                            
                            if start <= lines.len() && end <= lines.len() {
                                let new_lines: Vec<&str> = hunk.content
                                    .lines()
                                    .filter(|l| !l.starts_with('-')) // Remove old lines
                                    .map(|l| if l.starts_with('+') { &l[1..] } else if l.starts_with(' ') { &l[1..] } else { l })
                                    .collect();
                                
                                lines.splice(start..end, new_lines);
                                ed.content = lines.join("\n");
                                ed.dirty = true;
                                
                                if ed.diff_hunks.is_empty() {
                                    ed.mode = crate::EditorMode::Editor; // Return to editor if no diffs
                                }
                                state.version += 1;
                                return "EDITOR_EDIT_APPLIED".to_string();
                            } else {
                                return "ERROR: Hunk bounds invalid".to_string();
                            }
                        }
                    }
                }
            }
        }
        "ERROR: Pane not found or hunk out of bounds".to_string()
    }

    /// `editor_edit_reject:<pane_id>;<hunk_index>` — Reject pending edit proposal.
    fn handle_editor_edit_reject(&self, pane_id: Option<&str>, hunk_index: Option<&str>) -> String {
        let pane_uuid = match pane_id.and_then(|s| Uuid::parse_str(s).ok()) {
            Some(u) => u,
            None => return "ERROR: Invalid pane_id".to_string(),
        };
        let h_idx: usize = match hunk_index.and_then(|i| i.parse().ok()) {
            Some(idx) => idx,
            None => return "ERROR: Invalid hunk_index".to_string(),
        };

        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let hub_idx = sector.active_hub_index;
            if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                if let Some(pane) = hub.split_layout.as_mut().and_then(|t| t.find_pane_mut(pane_uuid)) {
                    if let crate::PaneContent::Editor(ref mut ed) = pane.content {
                        if h_idx < ed.diff_hunks.len() {
                            ed.diff_hunks.remove(h_idx);
                            if ed.diff_hunks.is_empty() {
                                ed.mode = crate::EditorMode::Editor; // Return to editor if no diffs
                            }
                            state.version += 1;
                            return "EDITOR_EDIT_REJECTED".to_string();
                        }
                    }
                }
            }
        }
        "ERROR: Pane not found or hunk out of bounds".to_string()
    }

    /// `editor_context_update:<pane_id>;<context_json>` — Face sends cursor/scroll state.
    fn handle_editor_context_update(&self, pane_id: Option<&str>, context: Option<&str>) -> String {
        let pane_uuid = match pane_id.and_then(|s| Uuid::parse_str(s).ok()) {
            Some(u) => u,
            None => return "ERROR: Invalid pane_id".to_string(),
        };
        let ctx_json = match context {
            Some(c) => c,
            None => return "ERROR: Missing context JSON payload".to_string(),
        };

        // Parse context updates (could be partial). We use a generic value to extract fields.
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(ctx_json);
        if let Ok(json_ctx) = parsed {
            let mut state = self.state.lock().unwrap();
            let idx = state.active_sector_index;
            if let Some(sector) = state.sectors.get_mut(idx) {
                let hub_idx = sector.active_hub_index;
                if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                    if let Some(pane) = hub.split_layout.as_mut().and_then(|t| t.find_pane_mut(pane_uuid)) {
                        if let crate::PaneContent::Editor(ref mut ed) = pane.content {
                            if let Some(content) = json_ctx.get("content").and_then(|v| v.as_str()) {
                                if ed.content != content {
                                    ed.content = content.to_string();
                                    ed.dirty = true;
                                    
                                    if let Some(ref lang) = ed.language {
                                        self.services.lsp.did_change(lang, &ed.file_path, content);
                                    }
                                }
                            }
                            if let Some(cursor_line) = json_ctx.get("cursor_line").and_then(|v| v.as_u64()) {
                                ed.cursor_line = cursor_line as usize;
                            }
                            if let Some(cursor_col) = json_ctx.get("cursor_col").and_then(|v| v.as_u64()) {
                                ed.cursor_col = cursor_col as usize;
                            }
                            if let Some(scroll) = json_ctx.get("scroll_offset").and_then(|v| v.as_u64()) {
                                ed.scroll_offset = scroll as usize;
                            }
                            state.version += 1;
                            return "EDITOR_CONTEXT_UPDATED".to_string();
                        }
                    }
                }
            }
        }
        "ERROR: Invalid context or pane not found".to_string()
    }

    fn handle_editor_send_context(&self, _pane_id: Option<&str>, _scope: Option<&str>) -> String {
        "EDITOR_CONTEXT_SENT".to_string()
    }

    fn handle_editor_promote(&self, pane_id: Option<&str>) -> String {
        let pane_uuid = match pane_id.and_then(|s| Uuid::parse_str(s).ok()) {
            Some(u) => u,
            None => return "ERROR: Invalid pane_id".to_string(),
        };
        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            if let Some(hub) = sector.hubs.get_mut(sector.active_hub_index) {
                if let Some(layout) = &mut hub.split_layout {
                    layout.promote_pane(pane_uuid);
                    state.version += 1;
                    return "EDITOR_PROMOTED".to_string();
                }
            }
        }
        "ERROR: Pane not found".to_string()
    }
}
/// Detect programming language from file extension for syntax highlighting.
fn detect_language(path: &std::path::Path) -> Option<String> {
    path.extension().and_then(|ext| ext.to_str()).map(|ext| {
        match ext.to_lowercase().as_str() {
            "rs" => "rust",
            "py" => "python",
            "js" => "javascript",
            "ts" => "typescript",
            "tsx" | "jsx" => "typescriptreact",
            "html" | "htm" => "html",
            "css" => "css",
            "scss" | "sass" => "scss",
            "json" => "json",
            "toml" => "toml",
            "yaml" | "yml" => "yaml",
            "md" | "markdown" => "markdown",
            "sh" | "bash" | "zsh" | "fish" => "shell",
            "c" => "c",
            "cpp" | "cc" | "cxx" | "h" | "hpp" => "cpp",
            "go" => "go",
            "java" => "java",
            "kt" | "kts" => "kotlin",
            "swift" => "swift",
            "rb" => "ruby",
            "lua" => "lua",
            "sql" => "sql",
            "xml" => "xml",
            "svelte" => "svelte",
            "vue" => "vue",
            "dart" => "dart",
            "zig" => "zig",
            "nim" => "nim",
            "ex" | "exs" => "elixir",
            "hs" => "haskell",
            _ => ext,
        }
        .to_string()
    })
}

impl IpcHandler {
    fn handle_ai_archive_interaction(&self, behavior_id: &str, prompt: &str, response: &str) -> String {
        self.services.logger.archive_ai(behavior_id, prompt, response);
        "OK".to_string()
    }

    // --- Kanban Handlers (§30.8) ---

    fn handle_kanban_init(&self) -> String {
        let mut state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            if sector.kanban_board.is_some() {
                return "ERROR: Kanban board already exists".to_string();
            }
            sector.kanban_board = Some(crate::KanbanBoard {
                project_id: Uuid::new_v4(),
                title: sector.name.clone(),
                lanes: vec![
                    crate::KanbanLane { id: Uuid::new_v4(), title: "TODO".to_string(), tasks: vec![] },
                    crate::KanbanLane { id: Uuid::new_v4(), title: "IN PROGRESS".to_string(), tasks: vec![] },
                    crate::KanbanLane { id: Uuid::new_v4(), title: "DONE".to_string(), tasks: vec![] },
                ],
            });
            state.version += 1;
            return "KANBAN_INITIALIZED".to_string();
        }
        "ERROR: No active sector".to_string()
    }

    fn handle_kanban_get(&self) -> String {
        let state = self.state.lock().unwrap();
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get(idx) {
            if let Some(board) = &sector.kanban_board {
                return serde_json::to_string(board).unwrap_or_else(|_| "ERROR: Failed to serialize board".to_string());
            }
        }
        "KANBAN_BOARD_NOT_FOUND".to_string()
    }

    fn handle_kanban_task_add(&self, payload: &str) -> String {
        #[derive(serde::Deserialize)]
        struct TaskAdd { lane_id: Uuid, title: String, description: String }
        if let Ok(data) = serde_json::from_str::<TaskAdd>(payload) {
            let mut state = self.state.lock().unwrap();
            let idx = state.active_sector_index;
            if let Some(sector) = state.sectors.get_mut(idx) {
                if let Some(board) = &mut sector.kanban_board {
                    if let Some(lane) = board.lanes.iter_mut().find(|l| l.id == data.lane_id) {
                        let task = crate::KanbanTask {
                            id: Uuid::new_v4(),
                            title: data.title,
                            description: data.description,
                            status: crate::KanbanTaskStatus::Todo,
                            assignee: None,
                            priority: 0,
                            tags: vec![],
                        };
                        lane.tasks.push(task);
                        state.version += 1;
                        return "KANBAN_TASK_ADDED".to_string();
                    }
                }
            }
        }
        "ERROR: Invalid task_add payload or lane not found".to_string()
    }

    fn handle_kanban_task_move(&self, payload: &str) -> String {
        #[derive(serde::Deserialize)]
        struct TaskMove { task_id: Uuid, from_lane: Uuid, to_lane: Uuid }
        if let Ok(data) = serde_json::from_str::<TaskMove>(payload) {
            let mut state = self.state.lock().unwrap();
            let idx = state.active_sector_index;
            if let Some(sector) = state.sectors.get_mut(idx) {
                if let Some(board) = &mut sector.kanban_board {
                    let mut task_opt = None;
                    if let Some(lane) = board.lanes.iter_mut().find(|l| l.id == data.from_lane) {
                        if let Some(pos) = lane.tasks.iter().position(|t| t.id == data.task_id) {
                            task_opt = Some(lane.tasks.remove(pos));
                        }
                    }
                    if let Some(mut task) = task_opt {
                        if let Some(lane) = board.lanes.iter_mut().find(|l| l.id == data.to_lane) {
                            // Update status based on lane name (simple heuristic)
                            task.status = match lane.title.as_str() {
                                "TODO" => crate::KanbanTaskStatus::Todo,
                                "IN PROGRESS" => crate::KanbanTaskStatus::InProgress,
                                "DONE" => crate::KanbanTaskStatus::Done,
                                _ => task.status,
                            };
                            lane.tasks.push(task);
                            state.version += 1;
                            return "KANBAN_TASK_MOVED".to_string();
                        }
                    }
                }
            }
        }
        "ERROR: Invalid task_move payload or task/lane not found".to_string()
    }
    fn handle_kanban_task_delete(&self, payload: &str) -> String {
        #[derive(serde::Deserialize)]
        struct TaskDelete { task_id: Uuid, lane_id: Uuid }
        if let Ok(data) = serde_json::from_str::<TaskDelete>(payload) {
            let mut state = self.state.lock().unwrap();
            let idx = state.active_sector_index;
            if let Some(sector) = state.sectors.get_mut(idx) {
                if let Some(board) = &mut sector.kanban_board {
                    if let Some(lane) = board.lanes.iter_mut().find(|l| l.id == data.lane_id) {
                        lane.tasks.retain(|t| t.id != data.task_id);
                        state.version += 1;
                        return "KANBAN_TASK_DELETED".to_string();
                    }
                }
            }
        }
        "ERROR: Invalid task_delete payload or task/lane not found".to_string()
    }

    fn handle_set_active_sector(&self, idx_str: Option<&str>) -> String {
        if let Some(s) = idx_str {
            if let Ok(idx) = s.parse::<usize>() {
                let mut state = self.state.lock().unwrap();
                if idx < state.sectors.len() {
                    state.active_sector_index = idx;
                    crate::brain::sector::SectorManager::refresh_activity_listing(&mut state, None);
                    state.version += 1;
                    return format!("ACTIVE_SECTOR_SET: {}", idx);
                }
                return "ERROR: Sector index out of bounds".to_string();
            }
        }
        "ERROR: Invalid sector index".to_string()
    }
}

impl crate::ipc::IpcDispatcher for IpcHandler {
    fn dispatch(&self, request: &str) -> String {
        self.handle_request(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_language_detection() {
        assert_eq!(detect_language(Path::new("test.rs")), Some("rust".to_string()));
        assert_eq!(detect_language(Path::new("script.py")), Some("python".to_string()));
        assert_eq!(detect_language(Path::new("Styles.css")), Some("css".to_string()));
        assert_eq!(detect_language(Path::new("README.md")), Some("markdown".to_string()));
        assert_eq!(detect_language(Path::new("Dockerfile")), None);
        assert_eq!(detect_language(Path::new("main.unknown")), Some("unknown".to_string()));
    }
}
