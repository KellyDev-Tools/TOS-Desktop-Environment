use crate::system::pty::PtyHandle;
use crate::system::shell_api::ShellApiConfig;
use super::ShellProvider;

#[derive(Debug, Default)]
pub struct BashShellProvider;

impl BashShellProvider {
    pub fn new() -> Self {
        Self
    }
}

impl ShellProvider for BashShellProvider {
    fn name(&self) -> &str {
        "bash"
    }

    fn default_path(&self) -> &str {
        "/bin/bash"
    }

    fn get_integration_script(&self) -> String {
        let config = ShellApiConfig::default();
        let parser = crate::system::shell_api::OscParser::with_config(config);
        parser.generate_bash_integration()
    }

    fn spawn(&self, cwd: &str) -> Option<PtyHandle> {
        let script = self.get_integration_script();
        
        let tos_dir = dirs::data_dir()?.join("tos");
        std::fs::create_dir_all(&tos_dir).ok()?;
        let script_path = tos_dir.join("bash_integration.bash");
        std::fs::write(&script_path, script).ok()?;
        
        let path_str = script_path.to_str()?;
        
        // Use --rcfile to load integration script
        PtyHandle::spawn_with_args("/bin/bash", &["--rcfile", path_str], cwd)
    }
}
