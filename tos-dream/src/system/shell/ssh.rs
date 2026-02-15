//! Phase 12: SSH Shell Provider
//! 
//! Provides PTY spawning for remote SSH connections.

use crate::system::pty::PtyHandle;
use super::ShellProvider;

#[derive(Debug, Default)]
pub struct SshShellProvider;

impl SshShellProvider {
    pub fn new() -> Self {
        Self
    }
    
    /// Spawn SSH connection to a specific host
    pub fn spawn_ssh(&self, host: &str, username: Option<&str>) -> Option<PtyHandle> {
        let ssh_target = if let Some(user) = username {
            format!("{}@{}", user, host)
        } else {
            host.to_string()
        };
        
        // We use -t to force TTY allocation for interactive shell
        PtyHandle::spawn_with_args("/usr/bin/ssh", &["-t", &ssh_target], ".")
    }
}

impl ShellProvider for SshShellProvider {
    fn name(&self) -> &str {
        "ssh"
    }

    fn default_path(&self) -> &str {
        "/usr/bin/ssh"
    }

    fn get_integration_script(&self) -> String {
        // SSH doesn't have a local integration script like fish, 
        // as the integration happens on the remote side.
        String::new()
    }

    fn spawn(&self, _cwd: &str) -> Option<PtyHandle> {
        // Default spawn for SSH without host is not very useful, 
        // but we can provide a placeholder or help text.
        None
    }
}
