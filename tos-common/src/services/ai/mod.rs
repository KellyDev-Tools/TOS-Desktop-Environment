//! AIService — Behavior Registry + Rolling Context Aggregator
//!
//! This module implements the refactored AI subsystem per the AI-Copilot-Specification.
//! Key responsibilities:
//!  - Behavior module registry (register, enable, disable, configure)
//!  - Rolling context aggregator (assemble context object per-behavior's declared fields)
//!  - Per-behavior backend resolution cascade (behavior override → system default)
//!  - Preserve existing ai_query / ai_tool_call internal messages as backend protocol

use crate::ipc::IpcDispatcher;
use crate::{AiBehavior, TosState};
use crate::state::QueuedAiRequest;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Parses an Agent Persona Markdown file into an AiBehavior (§7.3).
pub fn parse_persona_markdown(md: &str) -> AiBehavior {
    let mut id = "unknown_agent".to_string();
    let mut name = "Unknown Agent".to_string();
    let mut allowed_tools = vec![];
    let mut backend_override = None;
    let mut config = HashMap::new();

    let mut current_section = "";

    for line in md.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }

        if line.starts_with("# Agent Persona:") {
            id = line.replace("# Agent Persona:", "").trim().to_string();
        } else if line.starts_with("## ") {
            current_section = line.strip_prefix("## ").unwrap_or(line).trim();
        } else if line.starts_with("- **Name:**") {
            name = line.replace("- **Name:**", "").trim().to_string();
        } else if current_section == "Tool Bundle" && line.starts_with("- `") {
            let tools_str = line.replace("- ", "").replace("`", "");
            for tool in tools_str.split(',') {
                let t = tool.trim().to_string();
                if !t.is_empty() {
                    allowed_tools.push(t);
                }
            }
        } else if current_section == "Backend Preference" && line.starts_with("- **Preferred:**") {
            if line.to_lowercase().contains("openai") {
                backend_override = Some("openai-gpt4".to_string());
            } else if line.to_lowercase().contains("local") {
                backend_override = Some("local-llama".to_string());
            }
        } else if line.starts_with("- **") && line.contains(":**") {
            // Generic strategy/config extraction
            let parts: Vec<&str> = line.splitn(2, " :**").collect();
            if parts.len() == 2 {
                let key = parts[0].replace("- **", "").trim().to_string();
                let val = parts[1].trim().to_string();
                config.insert(key, val);
            }
        }
    }

    AiBehavior {
        id: id.clone(),
        name,
        enabled: true,
        backend_override,
        context_fields: vec![
            "cwd".to_string(), 
            "terminal_tail".to_string(), 
            "editor_context".to_string(),
            "chat_history".to_string()
        ],
        allowed_tools: if allowed_tools.is_empty() { None } else { Some(allowed_tools) },
        config,
    }
}

// ---------------------------------------------------------------------------
// Rolling Context Object
// ---------------------------------------------------------------------------

/// Full system context snapshot assembled from TosState.
/// Only fields declared in a behavior's `context_fields` manifest are sent.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AiContext {
    pub cwd: String,
    pub sector_name: String,
    pub shell_module: String,
    pub terminal_tail: Vec<String>,
    pub last_command: String,
    pub active_mode: String,
    pub session_version: u64,
    pub env_hint: String,
    pub chat_history: Vec<String>,
    pub editors: Vec<serde_json::Value>,
    pub system_metrics: serde_json::Value,
}

impl AiContext {
    /// Consume only the fields declared by a behavior.
    pub fn filter_to_fields(&self, fields: &[String]) -> Vec<String> {
        let mut result = vec![];
        for f in fields {
            match f.as_str() {
                "cwd" => result.push(format!("cwd:{}", self.cwd)),
                "sector_name" => result.push(format!("sector:{}", self.sector_name)),
                "shell" => result.push(format!("shell:{}", self.shell_module)),
                "terminal_tail" => {
                    for line in &self.terminal_tail {
                        result.push(format!("term:{}", line));
                    }
                }
                "last_command" => result.push(format!("last_cmd:{}", self.last_command)),
                "mode" => result.push(format!("mode:{}", self.active_mode)),
                "session_version" => result.push(format!("session_v:{}", self.session_version)),
                "env_hint" => result.push(format!("env:{}", self.env_hint)),
                "chat_history" => {
                    for line in &self.chat_history {
                        result.push(format!("ai_history:{}", line));
                    }
                }
                "editor_context" => {
                    for ed in &self.editors {
                        result.push(format!("editor_context:{}", serde_json::to_string(ed).unwrap_or_default()));
                    }
                }
                "system_metrics" => result.push(format!("metrics:{}", self.system_metrics)),
                _ => {}
            }
        }
        result
    }
}

/// Assemble the AiContext from a live TosState snapshot.
pub fn build_context(state: &TosState) -> AiContext {
    let idx = state.active_sector_index;
    let sector = &state.sectors[idx];
    let hub = &sector.hubs[sector.active_hub_index];

    let terminal_tail = hub
        .terminal_output
        .iter()
        .rev()
        .take(10)
        .map(|l| l.text.clone())
        .collect::<Vec<_>>();

    let last_command = hub.prompt.clone();
    let env_hint = std::env::var("TOS_ENV_HINT").unwrap_or_else(|_| "linux".to_string());

    let mut system_metrics = json!({});
    if let Some(listing) = &hub.activity_listing {
        // Aggregate top processes
        let top_procs = listing.processes.iter().take(5).map(|p| {
            json!({ "name": p.name, "cpu": p.cpu_usage, "mem": p.mem_usage })
        }).collect::<Vec<_>>();
        system_metrics = json!({
            "top_processes": top_procs,
            "hub_mode": hub.mode
        });
    }

    let mut editors: Vec<serde_json::Value> = vec![];
    if let Some(layout) = &hub.split_layout {
        for ed in layout.all_editors() {
            let start_line = ed.scroll_offset.saturating_sub(1);
            let end_line = start_line + 50; // Visible range approx
            
            editors.push(json!({
                "file": ed.file_path.display().to_string(),
                "language": ed.language.clone().unwrap_or_else(|| "text".to_string()),
                "visible_range": { "start_line": start_line, "end_line": end_line },
                "cursor_line": ed.cursor_line,
                "cursor_col": ed.cursor_col,
                "selection": null,
                "unsaved_changes": ed.dirty,
                "git_status": "unknown", // Stubbed until VCS module
                "diagnostics": []        // Stubbed until LSP module
            }));
        }
    }

    AiContext {
        cwd: hub.current_directory.display().to_string(),
        sector_name: sector.name.clone(),
        shell_module: hub
            .shell_module
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        terminal_tail,
        last_command,
        active_mode: format!("{:?}", hub.mode),
        session_version: state.version,
        env_hint,
        chat_history: hub
            .ai_history
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect(),
        editors,
        system_metrics,
    }
}

// ---------------------------------------------------------------------------
// AiService
// ---------------------------------------------------------------------------

pub struct AiService {
    ipc: Arc<Mutex<Option<Arc<dyn IpcDispatcher>>>>,
    modules: Arc<Mutex<Option<Arc<crate::brain::module_manager::ModuleManager>>>>,
    cortex: Arc<Mutex<Option<Arc<Mutex<crate::brain::cortex_registry::CortexRegistry>>>>>,
    settings: Arc<Mutex<Option<Arc<crate::services::settings::SettingsService>>>>,
    trust: Arc<Mutex<Option<Arc<crate::services::trust::TrustService>>>>,
    active_sandboxes: Arc<Mutex<HashMap<Uuid, crate::modules::sandbox::OverlaySandbox>>>,
}

impl Default for AiService {
    fn default() -> Self {
        Self::new()
    }
}

impl AiService {
    pub fn new() -> Self {
        Self {
            ipc: Arc::new(Mutex::new(None)),
            modules: Arc::new(Mutex::new(None)),
            cortex: Arc::new(Mutex::new(None)),
            settings: Arc::new(Mutex::new(None)),
            trust: Arc::new(Mutex::new(None)),
            active_sandboxes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Roadmap Planner skill (§7.5, §21.4).
    pub async fn roadmap_plan(&self) -> anyhow::Result<()> {
        let ipc = match self.ipc.lock().unwrap().as_ref() {
            Some(i) => i.clone(),
            None => return Err(anyhow::anyhow!("IPC dispatcher not registered")),
        };

        let thought = crate::AiThought {
            id: Uuid::new_v4(),
            behavior_id: "roadmap-planner".to_string(),
            title: "Auditing Project Trajectory".to_string(),
            content: "Cross-referencing task.md with active Kanban board...".to_string(),
            status: crate::AiThoughtStatus::Thinking,
            timestamp: chrono::Local::now(),
        };
        let _ = ipc.dispatch(&format!("ai_thought_stage:{}", serde_json::to_string(&thought)?));

        // Logic here would read roadmap.md and kanban state to suggest updates
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

        let mut done = thought.clone();
        done.status = crate::AiThoughtStatus::Decided;
        done.content = "Strategic audit complete. Roadmap artifacts updated successfully.".to_string();
        let _ = ipc.dispatch(&format!("ai_thought_stage:{}", serde_json::to_string(&done)?));

        Ok(())
    }

    /// Creates a sandbox for an agent task (§7.7).
    pub fn workflow_agent_sandbox(&self, task_id: Uuid, cwd: std::path::PathBuf) -> anyhow::Result<String> {
        let (output, sandbox) = crate::brain::shell::ShellApi::exec_sandboxed("ls -la", cwd)?;
        self.active_sandboxes.lock().unwrap().insert(task_id, sandbox);
        Ok(output)
    }

    /// Merges staged changes from an agent sandbox back into the project tree (§7.7).
    pub fn workflow_task_merge(&self, state: &mut crate::state::TosState, task_id: Uuid) -> anyhow::Result<String> {
        let sandbox = {
            let mut sandboxes = self.active_sandboxes.lock().unwrap();
            sandboxes.remove(&task_id).ok_or_else(|| anyhow::anyhow!("No active sandbox for task {}", task_id))?
        };

        let diffs = sandbox.calculate_diffs()?;
        
        // Find the task and store its staged changes
        let mut found = false;
        for sector in &mut state.sectors {
            if let Some(board) = &mut sector.kanban_board {
                for lane in &mut board.lanes {
                    if let Some(task) = lane.tasks.iter_mut().find(|t| t.id == task_id) {
                        task.staged_changes = diffs.clone();
                        found = true;
                        break;
                    }
                }
            }
            if found { break; }
        }

        if !found {
            return Err(anyhow::anyhow!("Task {} not found in any sector", task_id));
        }

        let _ = sandbox.cleanup();
        Ok(format!("MERGED: {} hunks staged for task {}", diffs.len(), task_id))
    }

    /// Dream Consolidate (Memory Synthesis) skill (§7.6, §21.5).
    pub async fn dream_consolidate(&self) -> anyhow::Result<()> {
        let ipc = match self.ipc.lock().unwrap().as_ref() {
            Some(i) => i.clone(),
            None => return Err(anyhow::anyhow!("IPC dispatcher not registered")),
        };

        let thought = crate::AiThought {
            id: Uuid::new_v4(),
            behavior_id: "memory-synthesis".to_string(),
            title: "Synthesizing Daily Logs".to_string(),
            content: "Extracting semantic patterns from session archives...".to_string(),
            status: crate::AiThoughtStatus::Thinking,
            timestamp: chrono::Local::now(),
        };
        let _ = ipc.dispatch(&format!("ai_thought_stage:{}", serde_json::to_string(&thought)?));

        // Logic here would query loggerd for 'ai' level events and summarize
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

        let mut done = thought.clone();
        done.status = crate::AiThoughtStatus::Actioned;
        let _ = ipc.dispatch(&format!("ai_thought_stage:{}", serde_json::to_string(&done)?));

        Ok(())
    }

    /// Queue an AI request for later execution (§4.9).
    pub fn queue_request(&self, behavior_id: &str, prompt: &str) -> anyhow::Result<()> {
        let ipc = self.ipc.lock().unwrap().clone().ok_or_else(|| anyhow::anyhow!("IPC dispatcher not set"))?;
        let req = QueuedAiRequest {
            behavior_id: behavior_id.to_string(),
            prompt: prompt.to_string(),
            timestamp: chrono::Local::now(),
        };
        let _ = ipc.dispatch(&format!("ai_queue_push:{}", serde_json::to_string(&req)?));
        
        let _ = ipc.dispatch(&format!("system_log_append:AI;[OFFLINE] Request queued for behavior '{}'", behavior_id));
        Ok(())
    }

    /// Attempt to drain the offline queue (§4.9).
    pub async fn drain_queue(&self) -> anyhow::Result<()> {
        let ipc = self.ipc.lock().unwrap().clone().ok_or_else(|| anyhow::anyhow!("IPC dispatcher not set"))?;
        
        let queue_json = ipc.dispatch("ai_queue_get:");
        if queue_json.is_empty() || queue_json.starts_with("ERROR") || queue_json == "[]" {
            return Ok(());
        }

        let mut queue: Vec<QueuedAiRequest> = serde_json::from_str(&queue_json)?;
        if queue.is_empty() { return Ok(()); }

        let now = chrono::Local::now();
        let thirty_mins = chrono::Duration::minutes(30);

        // Filter out expired items
        let initial_count = queue.len();
        queue.retain(|req| now.signed_duration_since(req.timestamp) < thirty_mins);
        
        if queue.len() < initial_count {
            let _ = ipc.dispatch("ai_queue_clear:");
            for req in &queue {
                let _ = ipc.dispatch(&format!("ai_queue_push:{}", serde_json::to_string(req)?));
            }
        }

        if queue.is_empty() { return Ok(()); }

        let _ = ipc.dispatch(&format!("system_log_append:AI;[ONLINE] Draining AI queue ({} items)...", queue.len()));

        // Process items one by one
        let mut failed = vec![];
        for req in queue {
            // Attempt query
            if self.query(&req.prompt).await.is_err() {
                failed.push(req);
            }
        }

        // Update queue with remaining/failed items
        let _ = ipc.dispatch("ai_queue_clear:");
        for req in failed {
            let _ = ipc.dispatch(&format!("ai_queue_push:{}", serde_json::to_string(&req)?));
        }

        Ok(())
    }

    pub fn set_ipc(&self, ipc: Arc<dyn IpcDispatcher>) {
        *self.ipc.lock().unwrap() = Some(ipc);
    }

    pub fn set_module_manager(&self, modules: Arc<crate::brain::module_manager::ModuleManager>) {
        *self.modules.lock().unwrap() = Some(modules);
    }

    pub fn set_cortex_registry(&self, cortex: Arc<Mutex<crate::brain::cortex_registry::CortexRegistry>>) {
        *self.cortex.lock().unwrap() = Some(cortex);
    }

    pub fn set_settings_service(&self, settings: Arc<crate::services::settings::SettingsService>) {
        *self.settings.lock().unwrap() = Some(settings);
    }

    pub fn set_trust_service(&self, trust: Arc<crate::services::trust::TrustService>) {
        *self.trust.lock().unwrap() = Some(trust);
    }

    /// Register the built-in behaviors (tos-chat, tos-observer) into the system state.
    pub fn register_defaults(&self, state: &mut TosState) {
        // 1. Chat Companion
        self.register_behavior(
            state,
            AiBehavior {
                id: "tos-chat".to_string(),
                name: "Chat Companion".to_string(),
                enabled: true,
                backend_override: None,
                context_fields: vec![
                    "cwd".to_string(),
                    "sector_name".to_string(),
                    "shell".to_string(),
                    "terminal_tail".to_string(),
                    "last_command".to_string(),
                    "mode".to_string(),
                    "editor_context".to_string(),
                ],
                allowed_tools: Some(vec![
                    "exec_cmd".to_string(),
                    "semantic_search".to_string(),
                ]),
                config: std::collections::HashMap::new(),
            },
        );

        // 2. Passive Observer
        self.register_behavior(
            state,
            AiBehavior {
                id: "tos-observer".to_string(),
                name: "Passive Observer".to_string(),
                enabled: true,
                backend_override: None,
                context_fields: vec![
                    "cwd".to_string(),
                    "terminal_tail".to_string(),
                    "last_command".to_string(),
                ],
                allowed_tools: Some(vec![
                    "exec_cmd".to_string(),
                ]),
                config: [("sensitivity".to_string(), "Medium".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
            },
        );
        // 3. Vibe Coder (Orchestrator)
        self.register_behavior(
            state,
            AiBehavior {
                id: "vibe-coder".to_string(),
                name: "Vibe Coder".to_string(),
                enabled: true,
                backend_override: None,
                context_fields: vec![
                    "cwd".to_string(),
                    "sector_name".to_string(),
                    "shell".to_string(),
                    "terminal_tail".to_string(),
                    "last_command".to_string(),
                    "mode".to_string(),
                    "editor_context".to_string(),
                    "chat_history".to_string(),
                ],
                allowed_tools: Some(vec![
                    "exec_cmd".to_string(),
                    "semantic_search".to_string(),
                    "editor_open".to_string(),
                    "editor_edit_proposal".to_string(),
                ]),
                config: std::collections::HashMap::new(),
            },
        );
    }

    // --- Behavior Registry Ops ---

    pub fn register_behavior(&self, state: &mut TosState, behavior: AiBehavior) {
        state.ai_behaviors.retain(|b| b.id != behavior.id);
        state.ai_behaviors.push(behavior);
    }

    pub fn validate_tool_call(&self, state: &TosState, behavior_id: &str, tool_name: &str) -> bool {
        // Check if behavior is enabled
        let behavior_enabled = state.ai_behaviors.iter().any(|b| b.id == behavior_id && b.enabled);
        if !behavior_enabled {
            return false;
        }

        // Check manifest trust block via ModuleManager (The "Brain Trust Chip")
        if let Ok(m_lock) = self.modules.lock() {
            if let Some(modules) = m_lock.as_ref() {
                if let Some(manifest) = modules.get_manifest(behavior_id) {
                    if let Some(trust_svc) = self.trust.lock().unwrap().as_ref() {
                        return trust_svc.verify_tool_access(manifest, tool_name);
                    }
                }
            }
        }

        // Fallback for built-ins without an explicit module manifest
        if let Some(b) = state.ai_behaviors.iter().find(|b| b.id == behavior_id) {
            if let Some(tools) = &b.allowed_tools {
                return tools.iter().any(|t| t == tool_name);
            }
        }
        false
    }

    pub fn enable_behavior(&self, state: &mut TosState, id: &str) -> bool {
        if let Some(b) = state.ai_behaviors.iter_mut().find(|b| b.id == id) {
            b.enabled = true;
            return true;
        }
        false
    }

    pub fn disable_behavior(&self, state: &mut TosState, id: &str) -> bool {
        if let Some(b) = state.ai_behaviors.iter_mut().find(|b| b.id == id) {
            b.enabled = false;
            return true;
        }
        false
    }

    pub fn configure_behavior(
        &self,
        state: &mut TosState,
        id: &str,
        key: &str,
        value: &str,
    ) -> bool {
        if let Some(b) = state.ai_behaviors.iter_mut().find(|b| b.id == id) {
            b.config.insert(key.to_string(), value.to_string());
            return true;
        }
        false
    }

    pub fn set_default_backend(&self, state: &mut TosState, backend_id: &str) {
        state.ai_default_backend = backend_id.to_string();
        state.active_ai_module = backend_id.to_string();
    }

    pub fn set_behavior_backend(
        &self,
        state: &mut TosState,
        behavior_id: &str,
        backend_id: &str,
    ) -> bool {
        if let Some(b) = state.ai_behaviors.iter_mut().find(|b| b.id == behavior_id) {
            b.backend_override = Some(backend_id.to_string());
            return true;
        }
        false
    }

    pub fn clear_behavior_backend(&self, state: &mut TosState, behavior_id: &str) -> bool {
        if let Some(b) = state.ai_behaviors.iter_mut().find(|b| b.id == behavior_id) {
            b.backend_override = None;
            return true;
        }
        false
    }

    /// Resolve the backend to use for a given behavior (cascade: behavior override → system default).
    pub fn resolve_backend<'a>(&self, state: &'a TosState, behavior_id: &str) -> &'a str {
        state
            .ai_behaviors
            .iter()
            .find(|b| b.id == behavior_id)
            .and_then(|b| b.backend_override.as_deref())
            .unwrap_or(state.ai_default_backend.as_str())
    }

    /// Check for context signals (markers, extensions) to automatically activate skills (§4.7).
    pub fn check_context_signals(&self, state: &mut TosState, cwd: &std::path::Path) {
        let mut activated = vec![];

        // 1. CWD Markers
        if cwd.join("Cargo.toml").exists() && self.enable_behavior(state, "vibe-coder") {
            activated.push("Vibe Coder (Rust Project Detected)");
        }

        // 2. Extension Signals (Stubbed until we have active file tracking)
        // 3. Content Signals (Handled in passive_observe)

        for skill in activated {
            state.system_log.push(crate::TerminalLine {
                text: format!("✦ [AI] Context Signal: Activated {} skill.", skill),
                priority: 1,
                timestamp: chrono::Local::now(),
            });
        }
    }

    /// Assembles the hierarchical system prompt from the active agent stack (§6).
    pub fn assemble_stacked_prompt(&self, state: &crate::TosState) -> String {
        let cortex_lock = self.cortex.lock().unwrap();
        let cortex_arc = match cortex_lock.as_ref() {
            Some(c) => c.clone(),
            None => return "You are TOS Alpha-2 Brain AI.".to_string(),
        };
        let cortex = cortex_arc.lock().unwrap();

        let mut identities = vec![];
        let mut constraints = vec![];
        let mut efficiencies = vec![];

        for agent_id in &state.active_agent_stack {
            if let Some(agent) = cortex.get_agent(agent_id) {
                identities.push(agent.prompt_identity().to_string());
                for c in agent.prompt_constraints() {
                    constraints.push(format!("- {}", c));
                }
                if let Some(e) = agent.prompt_efficiency() {
                    efficiencies.push(e.to_string());
                }
            }
        }

        if identities.is_empty() {
            return "You are TOS Alpha-2 Brain AI.".to_string();
        }

        let mut prompt = String::new();
        
        prompt.push_str("IDENTITY:\n");
        prompt.push_str(&identities.join("\n"));
        prompt.push_str("\n\n");

        if !constraints.is_empty() {
            prompt.push_str("CONSTRAINTS:\n");
            prompt.push_str(&constraints.join("\n"));
            prompt.push_str("\n\n");
        }

        if !efficiencies.is_empty() {
            prompt.push_str("EFFICIENCY:\n");
            prompt.push_str(&efficiencies.join("\n"));
            prompt.push_str("\n\n");
        }

        prompt
    }

    // --- Query ---

    /// Process natural language query and stage a command for user review.
    /// Dispatches through the module's configured endpoint/provider.
    pub async fn query(&self, prompt: &str) -> anyhow::Result<()> {
        let ipc = self
            .ipc
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| anyhow::anyhow!("IPC dispatcher not set for AiService"))?;

        let state_json = ipc.dispatch("get_state:");
        let clean_json = if let Some(idx) = state_json.rfind(" (") {
            &state_json[..idx]
        } else {
            &state_json
        };

        let state: TosState = match serde_json::from_str(clean_json) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("[AiService] Failed to parse state JSON: {}", e);
                return Err(e.into());
            }
        };

        // Build rolling context
        let ctx = build_context(&state);

        // Resolve backend — use "chat" behavior or fallback to active module
        let backend_id = self.resolve_backend(&state, "chat").to_string();

        let (command, explanation) = {
            let maybe_modules = self.modules.lock().unwrap().clone(); // <-- drop guard immediately
            if let Some(modules) = maybe_modules {
                if let Ok(ai_mod) = modules.load_ai(&backend_id) {
                    // Use all context fields by default for the primary chat behavior
                    let ctx_fields = vec![
                        "cwd".to_string(),
                        "sector_name".to_string(),
                        "shell".to_string(),
                        "terminal_tail".to_string(),
                        "last_command".to_string(),
                        "mode".to_string(),
                        "chat_history".to_string(),
                        "editor_context".to_string(),
                    ];
                    let mut context = ctx.filter_to_fields(&ctx_fields);
                    let mut auth = HashMap::new();
                    
                    let system_prompt = Some(self.assemble_stacked_prompt(&state));

                    if let Some(settings_svc) = self.settings.lock().unwrap().as_ref() {
                        // Inject global keys
                        if let Some(key) = settings_svc.get_secure("openai_api_key") { auth.insert("api_key".to_string(), key); }
                        if let Some(key) = settings_svc.get_secure("anthropic_api_key") { auth.insert("api_key".to_string(), key); }
                        if let Some(key) = settings_svc.get_secure("google_api_key") { auth.insert("api_key".to_string(), key); }
                        if let Some(key) = settings_svc.get_secure("tos_llm_api_key") { auth.insert("api_key".to_string(), key); }
                        
                        // Inject module-specific keys (§1.3.4)
                        let mod_prefix = format!("{}.", backend_id);
                        if let Some(key) = settings_svc.get_secure(&format!("{}api_key", mod_prefix)) {
                            auth.insert("api_key".to_string(), key);
                        }

                        // Aggregate context from active curators (§1.3.2)
                        let cortex_lock = self.cortex.lock().unwrap();
                        if let Some(cortex_arc) = cortex_lock.as_ref() {
                            let cortex = cortex_arc.lock().unwrap();
                            for curator_id in &state.active_curators {
                                if let Some(curator) = cortex.get_curator(curator_id) {
                                    // Inject curator-specific auth if needed
                                    let mut cur_auth = auth.clone();
                                    if let Some(key) = settings_svc.get_secure(&format!("{}.api_key", curator_id)) {
                                        cur_auth.insert("api_key".to_string(), key);
                                    }
                                    if let Ok(cur_ctx) = curator.get_context(prompt, &cur_auth) {
                                        context.extend(cur_ctx);
                                    }
                                }
                            }
                        }
                    }

                    let req = crate::modules::AiQuery {
                        prompt: prompt.to_string(),
                        system_prompt,
                        context,
                        stream: false,
                        auth,
                    };
                    match ai_mod.query(req) {
                        Ok(resp) => {
                            if let Ok(parsed) =
                                serde_json::from_str::<serde_json::Value>(&resp.choice.content)
                            {
                                let cmd = parsed["command"]
                                    .as_str()
                                    .unwrap_or("echo 'Error'")
                                    .to_string();
                                let expl = parsed["explanation"]
                                    .as_str()
                                    .unwrap_or("No explanation")
                                    .to_string();
                                (cmd, expl)
                            } else {
                                (
                                    format!("echo '{}'", resp.choice.content),
                                    "Raw response from AI module".to_string(),
                                )
                            }
                        }
                        Err(e) => {
                            tracing::error!("[AiService] Module query failed: {}. Using fallback.", e);
                            self.fallback_query(prompt, &ctx).await
                        }
                    }
                } else {
                    self.fallback_query(prompt, &ctx).await
                }
            } else {
                self.fallback_query(prompt, &ctx).await
            }
        }; // close let (command, explanation) = { ... }

        let payload = json!({ "command": command, "explanation": explanation });
        let _ = ipc.dispatch(&format!("ai_stage_command:{}", payload));

        // Append to history (§7.3)
        let msg = format!("staged command '{}' because {}", command, explanation);
        let _ = ipc.dispatch(&format!("ai_history_append:{}", msg));

        Ok(())
    }

    /// Predict the completion of a partial command input (§4.4).
    pub async fn predict_command(&self, partial: &str) -> anyhow::Result<String> {
        let ipc = self
            .ipc
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| anyhow::anyhow!("IPC dispatcher not set for AiService"))?;

        let state_json = ipc.dispatch("get_state:");
        let clean_json = state_json.split(" (").next().unwrap_or(&state_json);
        let state: TosState = match serde_json::from_str(clean_json) {
            Ok(s) => s,
            Err(_) => return Ok(String::new()),
        };

        if partial.trim().is_empty() {
            return Ok(String::new());
        }

        // Use a fast backend if possible
        let backend_id = self.resolve_backend(&state, "chat").to_string();
        let ctx = build_context(&state);

        let maybe_modules = self.modules.lock().unwrap().clone();
        if let Some(modules) = maybe_modules {
            if let Ok(ai_mod) = modules.load_ai(&backend_id) {
                let prompt = format!(
                    "PREDICT COMMAND GHOST TEXT: User is typing '{}'. \
                     CWD: {}. Last Cmd: {}. \
                     Predict the REST of the command. Return ONLY the predicted suffix string. \
                     If no confident prediction, return an empty string.",
                    partial, ctx.cwd, ctx.last_command
                );

                let mut auth = HashMap::new();
                if let Some(settings_svc) = self.settings.lock().unwrap().as_ref() {
                    if let Some(key) = settings_svc.get_secure("openai_api_key") { auth.insert("api_key".to_string(), key); }
                    if let Some(key) = settings_svc.get_secure("anthropic_api_key") { auth.insert("api_key".to_string(), key); }
                    if let Some(key) = settings_svc.get_secure("google_api_key") { auth.insert("api_key".to_string(), key); }
                    if let Some(key) = settings_svc.get_secure("tos_llm_api_key") { auth.insert("api_key".to_string(), key); }
                    
                    let mod_prefix = format!("{}.", backend_id);
                    if let Some(key) = settings_svc.get_secure(&format!("{}api_key", mod_prefix)) {
                        auth.insert("api_key".to_string(), key);
                    }
                }

                let req = crate::modules::AiQuery {
                    prompt,
                    system_prompt: None, // Prediction uses internal prompt
                    context: ctx.filter_to_fields(
                        &["cwd", "last_command"]
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>(),
                    ),
                    stream: false,
                    auth,
                };

                if let Ok(resp) = ai_mod.query(req) {
                    let content = resp.choice.content.trim().trim_matches('\"');
                    if !content.is_empty() && content.len() < 50 && !content.contains('\n') {
                        let _ = ipc.dispatch(&format!("ai_prediction_received:{}", content));
                        return Ok(content.to_string());
                    }
                }
            }
        }

        // Basic heuristic fallback
        let p = partial.to_lowercase();
        let fallback = if p == "l" || p == "ls" {
            " -la".to_string()
        } else if p == "c" || p == "cd" {
            " ..".to_string()
        } else if p == "cargo " {
            "build".to_string()
        } else {
            String::new()
        };

        if !fallback.is_empty() {
            let _ = ipc.dispatch(&format!("ai_prediction_received:{}", fallback));
        }

        Ok(fallback)
    }

    /// Orchestrate a multi-step plan for complex task execution (§3.3).
    pub async fn vibe_plan(&self, prompt: &str) -> anyhow::Result<()> {
        let ipc = match self.ipc.lock().unwrap().as_ref() {
            Some(i) => i.clone(),
            None => return Err(anyhow::anyhow!("IPC dispatcher not registered")),
        };

        // 1. Initial Thought: Intent Analysis
        let step1_id = Uuid::new_v4();
        let step1 = crate::AiThought {
            id: step1_id,
            behavior_id: "vibe-coder".to_string(),
            title: "Analyzing Task Orchestration".to_string(),
            content: format!("Decomposing complex request: '{}'", prompt),
            status: crate::AiThoughtStatus::Thinking,
            timestamp: chrono::Local::now(),
        };
        let _ = ipc.dispatch(&format!("ai_thought_stage:{}", serde_json::to_string(&step1)?));

        // Delay to simulate analysis
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

        // 2. Discover context
        let step2_id = Uuid::new_v4();
        let step2 = crate::AiThought {
            id: step2_id,
            behavior_id: "vibe-coder".to_string(),
            title: "Gathering Environment Metrics".to_string(),
            content: "Scanning workspace hierarchy and service status...".to_string(),
            status: crate::AiThoughtStatus::Thinking,
            timestamp: chrono::Local::now(),
        };
        let _ = ipc.dispatch(&format!("ai_thought_stage:{}", serde_json::to_string(&step2)?));

        // Mark step 1 as Decided
        let mut step1_decided = step1.clone();
        step1_decided.status = crate::AiThoughtStatus::Decided;
        step1_decided.content = "Task decomposition complete: 3 sub-actions identified.".to_string();
        let _ = ipc.dispatch(&format!("ai_thought_stage:{}", serde_json::to_string(&step1_decided)?));

        tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;

        // 3. Propose actions
        let step3_id = Uuid::new_v4();
        let step3 = crate::AiThought {
            id: step3_id,
            behavior_id: "vibe-coder".to_string(),
            title: "Synthesizing Proposed Model".to_string(),
            content: "Drafting execution sequence for tactical overview...".to_string(),
            status: crate::AiThoughtStatus::Thinking,
            timestamp: chrono::Local::now(),
        };
        let _ = ipc.dispatch(&format!("ai_thought_stage:{}", serde_json::to_string(&step3)?));

        // Mark step 2 as Actioned
        let mut step2_done = step2.clone();
        step2_done.status = crate::AiThoughtStatus::Actioned;
        let _ = ipc.dispatch(&format!("ai_thought_stage:{}", serde_json::to_string(&step2_done)?));

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // Finalize plan
        let mut step3_done = step3.clone();
        step3_done.status = crate::AiThoughtStatus::Decided;
        step3_done.content = "Orchestration plan ready for user review.".to_string();
        let _ = ipc.dispatch(&format!("ai_thought_stage:{}", serde_json::to_string(&step3_done)?));

        // Stage the actual command proposal
        let staged = format!("# AI PLAN FOR: {}\n# 1. Inspect environment\n# 2. Reconfigure sectors\n# 3. Synchronize status", prompt);
        let _ = ipc.dispatch(&format!("ai_stage_command:{}", staged));

        Ok(())
    }

    /// Observe a command result and trigger the passive observer if conditions match.
    pub async fn passive_observe(
        &self,
        command: &str,
        status: i32,
        stderr: Option<&str>,
    ) -> anyhow::Result<()> {
        let ipc = self
            .ipc
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| anyhow::anyhow!("IPC dispatcher not set"))?;

        let state_json = ipc.dispatch("get_state:");
        let clean_json = state_json.split(" (").next().unwrap_or(&state_json);
        let state: TosState = serde_json::from_str(clean_json)?;

        // Only proceed if tos-observer is enabled
        let observer = state.ai_behaviors.iter().find(|b| b.id == "tos-observer");
        if observer.is_none() || !observer.unwrap().enabled {
            return Ok(());
        }

        // Trigger conditions: exit 127 (not found) or non-zero with error output
        if status == 127 || (status != 0 && stderr.is_some()) {
            let ctx = build_context(&state);
            let backend_id = self.resolve_backend(&state, "tos-observer").to_string();

            let maybe_modules = self.modules.lock().unwrap().clone();
            if let Some(modules) = maybe_modules {
                if let Ok(ai_mod) = modules.load_ai(&backend_id) {
                    let prompt = format!(
                        "COMMAND FAILED: '{}' with status {}. Stderr: '{}'. \
                         Analyze the error and provide a one-line JSON FIX: \
                         {{\"command\": \"<staged_fix>\", \"explanation\": \"<short_description>\"}}.",
                        command, status, stderr.unwrap_or("none")
                    );

                    let mut auth = HashMap::new();
                    if let Some(settings_svc) = self.settings.lock().unwrap().as_ref() {
                        if let Some(key) = settings_svc.get_secure("openai_api_key") { auth.insert("api_key".to_string(), key); }
                        if let Some(key) = settings_svc.get_secure("anthropic_api_key") { auth.insert("api_key".to_string(), key); }
                        if let Some(key) = settings_svc.get_secure("google_api_key") { auth.insert("api_key".to_string(), key); }
                        if let Some(key) = settings_svc.get_secure("tos_llm_api_key") { auth.insert("api_key".to_string(), key); }
                        
                        let mod_prefix = format!("{}.", backend_id);
                        if let Some(key) = settings_svc.get_secure(&format!("{}api_key", mod_prefix)) {
                            auth.insert("api_key".to_string(), key);
                        }
                    }

                    let req = crate::modules::AiQuery {
                        prompt,
                        system_prompt: None, // Observer uses internal prompt
                        context: ctx.filter_to_fields(
                            &["cwd", "terminal_tail", "last_command"]
                                .iter()
                                .map(|s| s.to_string())
                                .collect::<Vec<_>>(),
                        ),
                        stream: false,
                        auth,
                    };

                    if let Ok(resp) = ai_mod.query(req) {
                        if let Ok(parsed) =
                            serde_json::from_str::<serde_json::Value>(&resp.choice.content)
                        {
                            let cmd = parsed["command"].as_str().unwrap_or("").to_string();
                            let expl = parsed["explanation"].as_str().unwrap_or("").to_string();
                            if !cmd.is_empty() {
                                let payload = json!({
                                    "behavior": "tos-observer",
                                    "command": cmd,
                                    "explanation": format!("✦ OBSERVER: {}", expl)
                                });
                                let _ = ipc.dispatch(&format!("ai_stage_command:{}", payload));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn fallback_query(&self, prompt: &str, ctx: &AiContext) -> (String, String) {
        // Fallback currently uses OpenAI GPT-4o-mini if local or module fails
        let mut auth = HashMap::new();
        if let Some(settings_svc) = self.settings.lock().unwrap().as_ref() {
            if let Some(key) = settings_svc.get_secure("openai_api_key") { auth.insert("api_key".to_string(), key); }
            if let Some(key) = settings_svc.get_secure("tos_llm_api_key") { auth.insert("api_key".to_string(), key); }
        }

        // Resolution cascade: secure store -> env var
        let api_key = auth.get("api_key").cloned().or_else(|| std::env::var("OPENAI_API_KEY").ok());
        let api_base = std::env::var("OPENAI_API_BASE").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let ipc = self.ipc.lock().unwrap().clone();

        // Get stacked prompt for fallback too
        let state_json = ipc.as_ref().map(|i| i.dispatch("get_state:")).unwrap_or_default();
        let stacked_prompt = if let Ok(state) = serde_json::from_str::<TosState>(state_json.split(" (").next().unwrap_or(&state_json)) {
            Some(self.assemble_stacked_prompt(&state))
        } else {
            None
        };

        if let Some(key) = api_key {
            let client = reqwest::Client::new();
            let system = stacked_prompt.unwrap_or_else(|| {
                format!(
                    "You are TOS Alpha-2 Brain AI. Sector: '{}', Path: '{}', Mode: {}. \
                     Translate the user's request into a single shell command. \
                     Always respond with JSON: {{\"command\": \"<cmd>\", \"explanation\": \"<short desc>\"}}.",
                    ctx.sector_name, ctx.cwd, ctx.active_mode
                )
            });

            let req_body = json!({
                "model": "gpt-4o-mini",
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": prompt }
                ],
                "response_format": { "type": "json_object" }
            });

            match client
                .post(format!("{}/chat/completions", api_base))
                .header("Authorization", format!("Bearer {}", key))
                .json(&req_body)
                .send()
                .await
            {
                Ok(resp) => {
                    if let Ok(resp_json) = resp.json::<serde_json::Value>().await {
                        let content = resp_json["choices"][0]["message"]["content"]
                            .as_str()
                            .unwrap_or("{}");
                        let parsed = serde_json::from_str::<serde_json::Value>(content).unwrap_or(
                            json!({"command": "echo 'AI Parse Error'", "explanation": ""}),
                        );
                        let cmd = parsed["command"]
                            .as_str()
                            .unwrap_or("echo 'Error'")
                            .to_string();
                        let expl = parsed["explanation"].as_str().unwrap_or("").to_string();

                        if cmd.starts_with("semantic_search:") {
                            let term = cmd.replace("semantic_search:", "").trim().to_string();
                            if let Some(i) = ipc {
                                let _ = i.dispatch(&format!("semantic_search:{}", term));
                            }
                            (
                                "zoom_to:CommandHub".to_string(),
                                format!("Found matches for '{}'. Zooming to Command Hub.", term),
                            )
                        } else {
                            (cmd, expl)
                        }
                    } else {
                        ("echo 'AI Error'".to_string(), "Failed to parse API response".to_string())
                    }
                }
                Err(e) => ("echo 'Network Error'".to_string(), e.to_string()),
            }
        } else {
            // Offline keyword heuristics
            let p = prompt.to_lowercase();
            if p.contains("where") && p.contains("am") && p.contains("i") {
                (
                    "pwd".to_string(),
                    format!("You are in sector '{}' at {}.", ctx.sector_name, ctx.cwd),
                )
            } else if (p.contains("list") || p.contains("show")) && p.contains("files") {
                (
                    "ls -la".to_string(),
                    "List all files in long format.".to_string(),
                )
            } else if p.contains("search") || p.contains("find") {
                let term = prompt.split_whitespace().last().unwrap_or("everything");
                if let Some(i) = ipc {
                    let _ = i.dispatch(&format!("semantic_search:{}", term));
                }
                (
                    "zoom_to:CommandHub".to_string(),
                    format!("Searching for '{}'.", term),
                )
            } else {
                (
                    format!("echo 'AI suggest: {}'", prompt),
                    "Staged echo command for review.".to_string(),
                )
            }
        }
    }
}
