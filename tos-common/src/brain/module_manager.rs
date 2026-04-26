use crate::modules::{AiModule, ShellIntegration, ShellModule};
use crate::services::marketplace::ModuleManifest;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct ModuleManager {
    modules: HashMap<String, ModuleManifest>,
    base_path: PathBuf,
}

impl ModuleManager {
    pub fn new(base_path: PathBuf) -> Self {
        let mut manager = Self {
            modules: HashMap::new(),
            base_path,
        };
        let _ = manager.discover_all();
        manager
    }

    /// Scans the base directory for valid TOS modules.
    pub fn discover_all(&mut self) -> anyhow::Result<()> {
        if !self.base_path.exists() {
            std::fs::create_dir_all(&self.base_path)?;
        }

        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Ok(manifest) =
                    crate::services::marketplace::MarketplaceService::discover_module_local(path)
                {
                    self.modules.insert(manifest.id.clone(), manifest);
                }
            }
        }
        Ok(())
    }

    pub fn get_manifest(&self, id: &str) -> Option<&ModuleManifest> {
        self.modules.get(id)
    }

    pub fn list_modules(&self) -> Vec<&ModuleManifest> {
        self.modules.values().collect()
    }

    /// Instantiates a ShellModule from a manifest.
    pub fn load_shell(&self, id: &str) -> anyhow::Result<Box<dyn ShellModule>> {
        let manifest = self
            .get_manifest(id)
            .ok_or_else(|| anyhow::anyhow!("Module not found"))?;
        if manifest.module_type != "shell" {
            return Err(anyhow::anyhow!("Module is not a shell"));
        }

        let exe = manifest
            .executable
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing executable config"))?;
        let integration = manifest.integration.clone().unwrap_or(ShellIntegration {
            osc_directory: true,
            osc_command_result: true,
            osc_suggestions: false,
        });

        // If the manifest specifies an absolute path (e.g. "/usr/bin/fish"),
        // use it directly. Otherwise resolve relative to the module directory.
        let full_path = if Path::new(&exe.path).is_absolute() {
            PathBuf::from(&exe.path)
        } else {
            let mut p = self.base_path.clone();
            p.push(id);
            p.push(&exe.path);
            p
        };

        Ok(Box::new(GenericShellModule {
            path: full_path,
            args: exe.args.clone(),
            integration,
        }))
    }

    /// Instantiates an AiModule from a manifest.
    pub fn load_ai(&self, id: &str) -> anyhow::Result<Box<dyn AiModule>> {
        let manifest = self
            .get_manifest(id)
            .ok_or_else(|| anyhow::anyhow!("Module not found"))?;
        if manifest.module_type != "ai" {
            return Err(anyhow::anyhow!("Module is not an AI backend"));
        }

        let caps = manifest.capabilities.clone().unwrap_or_default();
        let path = manifest.executable.as_ref().map(|exe| {
            let mut p = self.base_path.clone();
            p.push(id);
            p.push(&exe.path);
            p
        });

        Ok(Box::new(GenericAiModule {
            path,
            name: manifest.name.clone(),
            capabilities: caps,
            provider: manifest
                .provider
                .clone()
                .unwrap_or_else(|| "module".to_string()),
            endpoint: manifest.endpoint.clone(),
            latency_profile: manifest
                .latency_profile
                .clone()
                .unwrap_or_else(|| "medium".to_string()),
        }))
    }

    /// Instantiates a TerminalOutputModule from a manifest.
    pub fn load_terminal_output(
        &self,
        id: &str,
    ) -> anyhow::Result<Box<dyn crate::modules::TerminalOutputModule>> {
        let manifest = self
            .get_manifest(id)
            .ok_or_else(|| anyhow::anyhow!("Module not found"))?;
        if manifest.module_type != "TerminalOutput" {
            return Err(anyhow::anyhow!("Module is not a terminal output module"));
        }

        Ok(Box::new(GenericTerminalOutputModule { id: id.to_string() }))
    }

    /// Instantiates an AssistantModule from a manifest.
    pub fn load_assistant(&self, id: &str) -> anyhow::Result<Box<dyn crate::modules::AssistantModule>> {
        let manifest = self
            .get_manifest(id)
            .ok_or_else(|| anyhow::anyhow!("Module not found"))?;
        if manifest.module_type != "assistant" {
            return Err(anyhow::anyhow!("Module is not an assistant"));
        }

        Ok(Box::new(GenericAssistantModule {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            manifest: manifest.clone(),
        }))
    }

    /// Instantiates a CuratorModule from a manifest.
    pub fn load_curator(&self, id: &str) -> anyhow::Result<Box<dyn crate::modules::CuratorModule>> {
        let manifest = self
            .get_manifest(id)
            .ok_or_else(|| anyhow::anyhow!("Module not found"))?;
        if manifest.module_type != "curator" {
            return Err(anyhow::anyhow!("Module is not a curator"));
        }

        Ok(Box::new(GenericCuratorModule {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            manifest: manifest.clone(),
        }))
    }

    /// Instantiates an AgentModule from a manifest.
    pub fn load_agent(&self, id: &str) -> anyhow::Result<Box<dyn crate::modules::AgentModule>> {
        let manifest = self
            .get_manifest(id)
            .ok_or_else(|| anyhow::anyhow!("Module not found"))?;
        if manifest.module_type != "agent" {
            return Err(anyhow::anyhow!("Module is not an agent"));
        }

        Ok(Box::new(GenericAgentModule {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            manifest: manifest.clone(),
        }))
    }
}

struct GenericAssistantModule {
    id: String,
    name: String,
    manifest: ModuleManifest,
}

impl crate::modules::AssistantModule for GenericAssistantModule {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn query(&self, request: crate::modules::AiQuery) -> anyhow::Result<crate::modules::AiResponse> {
        // Assistant queries use the GenericAiModule logic under the hood for now,
        // but with manifest-driven configuration.
        let provider = self.manifest.provider.clone().unwrap_or_else(|| "module".to_string());
        let endpoint = self.manifest.endpoint.clone();
        
        // Wrap in a temporary GenericAiModule to reuse its logic
        let ai_mod = GenericAiModule {
            path: None, // Assistants usually don't have a binary unless provider is "module"
            name: self.name.clone(),
            capabilities: self.manifest.capabilities.clone().unwrap_or_default(),
            provider,
            endpoint,
            latency_profile: "medium".to_string(),
        };
        ai_mod.query(request)
    }
    fn list_models(&self) -> Vec<String> { vec!["default".to_string()] }
    fn capabilities(&self) -> &[String] {
        self.manifest.capabilities.as_deref().unwrap_or(&[])
    }
}

struct GenericCuratorModule {
    id: String,
    name: String,
    manifest: ModuleManifest,
}

impl crate::modules::CuratorModule for GenericCuratorModule {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn get_context(&self, _prompt: &str, _auth: &HashMap<String, String>) -> anyhow::Result<Vec<String>> {
        // MCP logic would go here. For now, it's a stub.
        Ok(vec![format!("Context from curator '{}'", self.name)])
    }
}

struct GenericAgentModule {
    id: String,
    name: String,
    manifest: ModuleManifest,
}

impl crate::modules::AgentModule for GenericAgentModule {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn prompt_identity(&self) -> &str {
        self.manifest.prompt.as_ref().map(|p| p.identity.as_str()).unwrap_or("You are a TOS Agent.")
    }
    fn prompt_constraints(&self) -> &[String] {
        self.manifest.prompt.as_ref().map(|p| p.constraints.as_slice()).unwrap_or(&[])
    }
}

struct GenericTerminalOutputModule {
    id: String,
}

impl crate::modules::TerminalOutputModule for GenericTerminalOutputModule {
    fn init(&mut self, _context: crate::TerminalContext, _config: serde_json::Value) {}
    fn push_lines(&mut self, _lines: Vec<crate::TerminalLine>) {
        // Logically, the Brain doesn't render; it just passes lines through.
        // In a headless system, this could pipe to a log or external surface.
    }
    fn get_id(&self) -> &str {
        &self.id
    }
}

// Internal generic implementation for built-in or simple shell modules
struct GenericShellModule {
    path: PathBuf,
    args: Vec<String>,
    integration: ShellIntegration,
}

impl ShellModule for GenericShellModule {
    fn get_executable_path(&self) -> &Path {
        &self.path
    }
    fn get_default_args(&self) -> &[String] {
        &self.args
    }
    fn get_integration_config(&self) -> &ShellIntegration {
        &self.integration
    }
}

struct GenericAiModule {
    path: Option<PathBuf>,
    name: String,
    capabilities: Vec<String>,
    provider: String,
    endpoint: Option<String>,
    #[allow(dead_code)]
    latency_profile: String,
}

impl AiModule for GenericAiModule {
    fn query(
        &self,
        request: crate::modules::AiQuery,
    ) -> anyhow::Result<crate::modules::AiResponse> {
        // --- Provider-driven HTTP dispatch ---
        // If the manifest declares an endpoint + provider, make a real API call
        // via a blocking tokio task. Otherwise fall through to subprocess exec.
        match self.provider.as_str() {
            "openai" | "anthropic" | "ollama" => {
                let base = self
                    .endpoint
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("AI module '{}' has no endpoint", self.name))?;
                let provider = self.provider.clone();
                let prompt = request.prompt.clone();
                let context = request.context.clone();
                let auth = request.auth.clone();

                // Blocking runtime for sync trait boundary
                let rt = tokio::runtime::Handle::try_current();
                let result = if let Ok(handle) = rt {
                    tokio::task::block_in_place(|| {
                        handle.block_on(llm_http_call(
                            &provider,
                            &base,
                            &auth,
                            &prompt,
                            &context,
                        ))
                    })
                } else {
                    tokio::runtime::Runtime::new()?.block_on(llm_http_call(
                        &provider,
                        &base,
                        &auth,
                        &prompt,
                        &context,
                    ))
                };

                return result;
            }
            _ => {} // Fall through to subprocess
        }

        // --- Subprocess exec ("module" provider) ---
        let path = match &self.path {
            Some(p) => p,
            None => {
                return Err(anyhow::anyhow!(
                    "AI Module '{}' has no executable path",
                    self.name
                ))
            }
        };

        if !path.exists() {
            return Err(anyhow::anyhow!(
                "AI Module executable not found at: {}",
                path.display()
            ));
        }

        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to open stdin"))?;
        let request_json = serde_json::to_string(&request)?;
        stdin.write_all(request_json.as_bytes())?;
        stdin.write_all(b"\n")?;
        drop(stdin);

        let output = child.wait_with_output()?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("AI Module execution failed: {}", err));
        }

        let response: crate::modules::AiResponse = serde_json::from_slice(&output.stdout)?;
        Ok(response)
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
}

/// Generic HTTP LLM call supporting OpenAI, Anthropic, and Ollama protocols.
async fn llm_http_call(
    provider: &str,
    base: &str,
    auth: &HashMap<String, String>,
    prompt: &str,
    context: &[String],
) -> anyhow::Result<crate::modules::AiResponse> {
    use serde_json::json;

    let client = reqwest::Client::new();
    let ctx_str = context.join("; ");
    let system = format!(
        "You are TOS Alpha-2 Brain AI. Context: {}. Respond with JSON {{\"command\": \"<shell cmd>\", \"explanation\": \"<short>\"}}",
        ctx_str
    );

    // Credential resolution cascade (§1.3.4):
    // 1. Injected auth map (from secure settings)
    // 2. Provider-specific env vars (legacy fallback)
    let api_key = auth.get("api_key")
        .cloned()
        .or_else(|| auth.get("token").cloned())
        .or_else(|| {
            match provider {
                "openai" => std::env::var("OPENAI_API_KEY").ok(),
                "anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
                _ => std::env::var("TOS_LLM_API_KEY").ok(),
            }
        });

    let (url, body, auth_header) = match provider {
        "anthropic" => {
            let key = api_key.ok_or_else(|| anyhow::anyhow!("ANTHROPIC_API_KEY or auth.api_key not set"))?;
            let body = json!({
                "model": "claude-3-5-sonnet-20241022",
                "max_tokens": 512,
                "system": system,
                "messages": [{"role": "user", "content": prompt}]
            });
            (
                format!("{}/messages", base),
                body,
                format!("x-api-key: {}", key),
            )
        }
        "ollama" => {
            let body = json!({
                "model": "llama3",
                "prompt": format!("{}: {}", system, prompt),
                "stream": false,
                "format": "json"
            });
            (format!("{}/api/generate", base), body, String::new())
        }
        _ => {
            // openai-compatible
            let key = api_key.ok_or_else(|| anyhow::anyhow!("OPENAI_API_KEY or auth.api_key not set"))?;
            let body = json!({
                "model": "gpt-4o-mini",
                "messages": [
                    {"role": "system", "content": system},
                    {"role": "user", "content": prompt}
                ],
                "response_format": {"type": "json_object"}
            });
            (
                format!("{}/chat/completions", base),
                body,
                format!("Bearer {}", key),
            )
        }
    };

    let mut req = client.post(&url).json(&body);
    if !auth_header.is_empty() {
        if auth_header.starts_with("x-api-key") {
            let parts: Vec<&str> = auth_header.splitn(2, ": ").collect();
            req = req
                .header("x-api-key", parts[1])
                .header("anthropic-version", "2023-06-01");
        } else {
            req = req.header("Authorization", &auth_header);
        }
    }

    let resp = req.send().await?.json::<serde_json::Value>().await?;

    // Normalize response across providers
    let content = match provider {
        "anthropic" => resp["content"][0]["text"]
            .as_str()
            .unwrap_or("{}")
            .to_string(),
        "ollama" => resp["response"].as_str().unwrap_or("{}").to_string(),
        _ => resp["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("{}")
            .to_string(),
    };

    Ok(crate::modules::AiResponse {
        id: uuid::Uuid::new_v4(),
        choice: crate::modules::AiChoice {
            role: "assistant".to_string(),
            content,
        },
        usage: crate::modules::AiUsage { tokens: 0 },
        status: crate::modules::AiStatus::Complete,
    })
}
