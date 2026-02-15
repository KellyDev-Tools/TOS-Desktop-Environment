use crate::system::pty::PtyHandle;
use crate::system::shell_api::OscParser;
use uuid::Uuid;

pub mod fish;

/// Trait for shell providers
pub trait ShellProvider: std::fmt::Debug + Send + Sync {
    /// Name of the shell
    fn name(&self) -> &str;
    
    /// Default executable path
    fn default_path(&self) -> &str;
    
    /// Get the integration script for this shell
    fn get_integration_script(&self) -> String;
    
    /// Spawn the shell with correct integration
    fn spawn(&self, cwd: &str) -> Option<PtyHandle>;
}

/// Registry for shell providers
#[derive(Debug, Default)]
pub struct ShellRegistry {
    providers: std::collections::HashMap<String, Box<dyn ShellProvider>>,
}

impl ShellRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            providers: std::collections::HashMap::new(),
        };
        registry.register(Box::new(fish::FishShellProvider::new()));
        registry
    }

    pub fn register(&mut self, provider: Box<dyn ShellProvider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    pub fn get(&self, name: &str) -> Option<&dyn ShellProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_registry_registration() {
        let mut registry = ShellRegistry::default();
        registry.register(Box::new(fish::FishShellProvider::new()));
        
        assert!(registry.get("fish").is_some());
        assert_eq!(registry.get("fish").unwrap().name(), "fish");
        assert!(registry.get("unknown").is_none());
    }
}
