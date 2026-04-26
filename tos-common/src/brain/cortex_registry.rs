use crate::brain::module_manager::ModuleManager;
use crate::modules::{AssistantModule, CuratorModule, AgentModule};
use std::sync::Arc;
use std::collections::HashMap;

/// The CortexRegistry manages the modular reasoning components (Assistants, Curators, Agents).
/// It decouples the monolithic AI Service into pluggable parts.
pub struct CortexRegistry {
    module_manager: Arc<ModuleManager>,
    assistants: HashMap<String, Box<dyn AssistantModule>>,
    curators: HashMap<String, Box<dyn CuratorModule>>,
    agents: HashMap<String, Box<dyn AgentModule>>,
}

impl CortexRegistry {
    pub fn new(module_manager: Arc<ModuleManager>) -> Self {
        let mut registry = Self {
            module_manager,
            assistants: HashMap::new(),
            curators: HashMap::new(),
            agents: HashMap::new(),
        };
        let _ = registry.reload_all();
        registry
    }

    /// Reloads all cortex components from the ModuleManager's discovered manifests.
    pub fn reload_all(&mut self) -> anyhow::Result<()> {
        let manifests = self.module_manager.list_modules();
        
        self.assistants.clear();
        self.curators.clear();
        self.agents.clear();

        for manifest in manifests {
            match manifest.module_type.as_str() {
                "assistant" => {
                    if let Ok(m) = self.module_manager.load_assistant(&manifest.id) {
                        self.assistants.insert(manifest.id.clone(), m);
                    }
                }
                "curator" => {
                    if let Ok(m) = self.module_manager.load_curator(&manifest.id) {
                        self.curators.insert(manifest.id.clone(), m);
                    }
                }
                "agent" => {
                    if let Ok(m) = self.module_manager.load_agent(&manifest.id) {
                        self.agents.insert(manifest.id.clone(), m);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn get_assistant(&self, id: &str) -> Option<&dyn AssistantModule> {
        self.assistants.get(id).map(|b| b.as_ref())
    }

    pub fn get_curator(&self, id: &str) -> Option<&dyn CuratorModule> {
        self.curators.get(id).map(|b| b.as_ref())
    }

    pub fn get_agent(&self, id: &str) -> Option<&dyn AgentModule> {
        self.agents.get(id).map(|b| b.as_ref())
    }

    pub fn list_assistants(&self) -> Vec<&dyn AssistantModule> {
        self.assistants.values().map(|b| b.as_ref()).collect()
    }

    pub fn list_curators(&self) -> Vec<&dyn CuratorModule> {
        self.curators.values().map(|b| b.as_ref()).collect()
    }

    pub fn list_agents(&self) -> Vec<&dyn AgentModule> {
        self.agents.values().map(|b| b.as_ref()).collect()
    }
}
