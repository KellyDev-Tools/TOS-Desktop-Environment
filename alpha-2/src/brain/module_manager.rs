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

        Ok(Box::new(GenericAiModule {
            id: id.to_string(),
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
    id: String,
    name: String,
    capabilities: Vec<String>,
}

impl AiModule for GenericAiModule {
    fn query(&self, _request: crate::common::modules::AiQuery) -> anyhow::Result<crate::common::modules::AiResponse> {
        // Real implementation would involve calling an external process or network API
        Err(anyhow::anyhow!("AI Module execution not yet implemented (Mock Mode)"))
    }
    fn name(&self) -> &str { &self.name }
    fn capabilities(&self) -> &[String] { &self.capabilities }
}
