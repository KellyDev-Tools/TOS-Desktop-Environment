use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::services::marketplace::ModuleManifest;
use crate::common::modules::{ShellModule, AiModule, ShellIntegration};

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
                if let Ok(manifest) = crate::services::marketplace::MarketplaceService::discover_module_local(path) {
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
        let manifest = self.get_manifest(id).ok_or_else(|| anyhow::anyhow!("Module not found"))?;
        if manifest.module_type != "shell" {
            return Err(anyhow::anyhow!("Module is not a shell"));
        }

        let exe = manifest.executable.as_ref().ok_or_else(|| anyhow::anyhow!("Missing executable config"))?;
        let integration = manifest.integration.clone().unwrap_or(ShellIntegration {
            osc_directory: true,
            osc_command_result: true,
            osc_suggestions: false,
        });

        let mut full_path = self.base_path.clone();
        full_path.push(id);
        full_path.push(&exe.path);

        Ok(Box::new(GenericShellModule {
            path: full_path,
            args: exe.args.clone(),
            integration,
        }))
    }

    /// Instantiates an AiModule from a manifest.
    pub fn load_ai(&self, id: &str) -> anyhow::Result<Box<dyn AiModule>> {
        let manifest = self.get_manifest(id).ok_or_else(|| anyhow::anyhow!("Module not found"))?;
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
        }))
    }
}

// Internal generic implementation for built-in or simple shell modules
struct GenericShellModule {
    path: PathBuf,
    args: Vec<String>,
    integration: ShellIntegration,
}

impl ShellModule for GenericShellModule {
    fn get_executable_path(&self) -> &Path { &self.path }
    fn get_default_args(&self) -> &[String] { &self.args }
    fn get_integration_config(&self) -> &ShellIntegration { &self.integration }
}

struct GenericAiModule {
    path: Option<PathBuf>,
    name: String,
    capabilities: Vec<String>,
}

impl AiModule for GenericAiModule {
    fn query(&self, request: crate::common::modules::AiQuery) -> anyhow::Result<crate::common::modules::AiResponse> {
        let path = match &self.path {
            Some(p) => p,
            None => return Err(anyhow::anyhow!("AI Module '{}' has no executable path", self.name)),
        };

        if !path.exists() {
            return Err(anyhow::anyhow!("AI Module executable not found at: {}", path.display()));
        }

        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("Failed to open stdin"))?;
        let request_json = serde_json::to_string(&request)?;
        stdin.write_all(request_json.as_bytes())?;
        stdin.write_all(b"\n")?;
        drop(stdin); // Signal EOF

        let output = child.wait_with_output()?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("AI Module execution failed: {}", err));
        }

        let response: crate::common::modules::AiResponse = serde_json::from_slice(&output.stdout)?;
        Ok(response)
    }
    fn name(&self) -> &str { &self.name }
    fn capabilities(&self) -> &[String] { &self.capabilities }
}
