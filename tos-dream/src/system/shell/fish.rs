use crate::system::pty::PtyHandle;
use crate::system::shell_api::ShellApiConfig;
use super::ShellProvider;
use std::io::Write;
use tempfile::NamedTempFile;

#[derive(Debug, Default)]
pub struct FishShellProvider;

impl FishShellProvider {
    pub fn new() -> Self {
        Self
    }
}

impl ShellProvider for FishShellProvider {
    fn name(&self) -> &str {
        "fish"
    }

    fn default_path(&self) -> &str {
        "/usr/bin/fish"
    }

    fn get_integration_script(&self) -> String {
        // Use the generator from shell_api
        let config = ShellApiConfig::default();
        let parser = crate::system::shell_api::OscParser::with_config(config);
        parser.generate_fish_integration()
    }

    fn spawn(&self, cwd: &str) -> Option<PtyHandle> {
        let script = self.get_integration_script();
        
        // Create a temporary file for the integration script
        // In a real implementation, we might use a persistent path or pass via env
        let mut temp = NamedTempFile::new().ok()?;
        write!(temp, "{}", script).ok()?;
        let path = temp.path().to_str()?.to_string();
        
        // Keep the temp file alive for a bit or assume fish sources it immediately
        // Actually, for fish, we can use --init-command
        let _init_cmd = format!("source {}", path);
        
        // We need to modify PtyHandle::spawn to accept arguments
        // For now, we'll use a trick: spawn a script that sources our integration
        let _shell_cmd = format!("source {}; exec fish", path);
        
        // This is a bit hacky because NamedTempFile will be deleted when 'temp' goes out of scope.
        // We'll use a more stable approach: write to a known location in ~/.local/share/tos
        let tos_dir = dirs::data_dir()?.join("tos");
        std::fs::create_dir_all(&tos_dir).ok()?;
        let script_path = tos_dir.join("fish_integration.fish");
        std::fs::write(&script_path, script).ok()?;
        
        let path_str = script_path.to_str()?;
        
        // Note: We need to update PtyHandle to support arguments if we want to do this properly.
        // For now, we'll use the environment variable trick if possible, or just spawn fish.
        // Fish doesn't easily take an init file from env like bash's BASH_ENV.
        
        // Let's assume we'll update PtyHandle::spawn soon.
        // For now, just spawn it and we'll fix PtyHandle in the next step.
        PtyHandle::spawn_with_args("/usr/bin/fish", &["-C", &format!("source {}", path_str)], cwd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fish_integration_script_content() {
        let provider = FishShellProvider::new();
        let script = provider.get_integration_script();
        
        // Verify key TOS integration functions are present
        assert!(script.contains("__tos_send_osc"));
        assert!(script.contains("__tos_preexec"));
        assert!(script.contains("__tos_postexec"));
        assert!(script.contains("printf \"\\033]9003;%s\\007\" $PWD"));
    }
}
