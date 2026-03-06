//! AIService — Behavior Registry + Rolling Context Aggregator
//!
//! This module implements the refactored AI subsystem per the AI-Copilot-Specification.
//! Key responsibilities:
//!  - Behavior module registry (register, enable, disable, configure)
//!  - Rolling context aggregator (assemble context object per-behavior's declared fields)
//!  - Per-behavior backend resolution cascade (behavior override → system default)
//!  - Preserve existing ai_query / ai_tool_call internal messages as backend protocol

use std::sync::{Arc, Mutex};
use crate::common::{TosState, AiBehavior};
use crate::common::ipc_dispatcher::IpcDispatcher;
use serde_json::json;

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

    let terminal_tail = hub.terminal_output
        .iter()
        .rev()
        .take(10)
        .map(|l| l.text.clone())
        .collect::<Vec<_>>();

    let last_command = hub.prompt.clone();
    let env_hint = std::env::var("TOS_ENV_HINT").unwrap_or_else(|_| "linux".to_string());

    AiContext {
        cwd: hub.current_directory.display().to_string(),
        sector_name: sector.name.clone(),
        shell_module: hub.shell_module.clone().unwrap_or_else(|| "unknown".to_string()),
        terminal_tail,
        last_command,
        active_mode: format!("{:?}", hub.mode),
        session_version: state.version,
        env_hint,
    }
}

// ---------------------------------------------------------------------------
// AiService
// ---------------------------------------------------------------------------

pub struct AiService {
    ipc: Arc<Mutex<Option<Arc<dyn IpcDispatcher>>>>,
    modules: Arc<Mutex<Option<Arc<crate::brain::module_manager::ModuleManager>>>>,
}

impl AiService {
    pub fn new() -> Self {
        Self {
            ipc: Arc::new(Mutex::new(None)),
            modules: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_ipc(&self, ipc: Arc<dyn IpcDispatcher>) {
        *self.ipc.lock().unwrap() = Some(ipc);
    }

    pub fn set_module_manager(&self, modules: Arc<crate::brain::module_manager::ModuleManager>) {
        *self.modules.lock().unwrap() = Some(modules);
    }

    // --- Behavior Registry Ops ---

    pub fn register_behavior(&self, state: &mut TosState, behavior: AiBehavior) {
        state.ai_behaviors.retain(|b| b.id != behavior.id);
        state.ai_behaviors.push(behavior);
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

    pub fn configure_behavior(&self, state: &mut TosState, id: &str, key: &str, value: &str) -> bool {
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

    pub fn set_behavior_backend(&self, state: &mut TosState, behavior_id: &str, backend_id: &str) -> bool {
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
        state.ai_behaviors
            .iter()
            .find(|b| b.id == behavior_id)
            .and_then(|b| b.backend_override.as_deref())
            .unwrap_or_else(|| state.ai_default_backend.as_str())
    }

    // --- Query ---

    /// Process natural language query and stage a command for user review.
    /// Dispatches through the module's configured endpoint/provider.
    pub async fn query(&self, prompt: &str) -> anyhow::Result<()> {
        let ipc = self.ipc.lock().unwrap().clone()
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
                eprintln!("[AiService] Failed to parse state JSON: {}", e);
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
                    "cwd".to_string(), "sector_name".to_string(), "shell".to_string(),
                    "terminal_tail".to_string(), "last_command".to_string(), "mode".to_string(),
                ];
                let context = ctx.filter_to_fields(&ctx_fields);
                let req = crate::common::modules::AiQuery {
                    prompt: prompt.to_string(),
                    context,
                    stream: false,
                };
                match ai_mod.query(req) {
                    Ok(resp) => {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&resp.choice.content) {
                            let cmd = parsed["command"].as_str().unwrap_or("echo 'Error'").to_string();
                            let expl = parsed["explanation"].as_str().unwrap_or("No explanation").to_string();
                            (cmd, expl)
                        } else {
                            (format!("echo '{}'", resp.choice.content), "Raw response from AI module".to_string())
                        }
                    }
                    Err(e) => {
                        eprintln!("[AiService] Module query failed: {}. Using fallback.", e);
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
        Ok(())
    }

    async fn fallback_query(&self, prompt: &str, ctx: &AiContext) -> (String, String) {
        let api_key = std::env::var("OPENAI_API_KEY");
        let api_base = std::env::var("OPENAI_API_BASE")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let ipc = self.ipc.lock().unwrap().clone();

        if let Ok(key) = api_key {
            let client = reqwest::Client::new();
            let system_prompt = format!(
                "You are TOS Alpha-2 Brain AI. Sector: '{}', Path: '{}', Mode: {}. \
                 Translate the user's request into a single shell command. \
                 Always respond with JSON: {{\"command\": \"<cmd>\", \"explanation\": \"<short desc>\"}}.",
                ctx.sector_name, ctx.cwd, ctx.active_mode
            );

            let req_body = json!({
                "model": "gpt-4o-mini",
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": prompt }
                ],
                "response_format": { "type": "json_object" }
            });

            match client.post(format!("{}/chat/completions", api_base))
                .header("Authorization", format!("Bearer {}", key))
                .json(&req_body)
                .send()
                .await
            {
                Ok(resp) => {
                    if let Ok(resp_json) = resp.json::<serde_json::Value>().await {
                        let content = resp_json["choices"][0]["message"]["content"].as_str().unwrap_or("{}");
                        let parsed = serde_json::from_str::<serde_json::Value>(content)
                            .unwrap_or(json!({"command": "echo 'AI Parse Error'", "explanation": ""}));
                        let cmd = parsed["command"].as_str().unwrap_or("echo 'Error'").to_string();
                        let expl = parsed["explanation"].as_str().unwrap_or("").to_string();

                        if cmd.starts_with("semantic_search:") {
                            let term = cmd.replace("semantic_search:", "").trim().to_string();
                            if let Some(i) = ipc {
                                let _ = i.dispatch(&format!("semantic_search:{}", term));
                            }
                            ("zoom_to:CommandHub".to_string(), format!("Found matches for '{}'. Zooming to Command Hub.", term))
                        } else {
                            (cmd, expl)
                        }
                    } else {
                        ("echo 'LLM JSON Error'".to_string(), "AI returned invalid JSON.".to_string())
                    }
                }
                Err(e) => (format!("echo 'LLM Error: {}'", e), "Network request failed.".to_string()),
            }
        } else {
            // Offline keyword heuristics
            let p = prompt.to_lowercase();
            if p.contains("where") && p.contains("am") && p.contains("i") {
                ("pwd".to_string(), format!("You are in sector '{}' at {}.", ctx.sector_name, ctx.cwd))
            } else if (p.contains("list") || p.contains("show")) && p.contains("files") {
                ("ls -la".to_string(), "List all files in long format.".to_string())
            } else if p.contains("search") || p.contains("find") {
                let term = prompt.split_whitespace().last().unwrap_or("everything");
                if let Some(i) = ipc {
                    let _ = i.dispatch(&format!("semantic_search:{}", term));
                }
                ("zoom_to:CommandHub".to_string(), format!("Searching for '{}'.", term))
            } else {
                (format!("echo 'AI suggest: {}'", prompt), "Staged echo command for review.".to_string())
            }
        }
    }
}
