use std::process::Command;
use uuid::Uuid;

pub struct SshSession {
    pub id: Uuid,
    pub host: String,
}

impl SshSession {
    /// §12.1: Try to establish an SSH connection as a fallback
    pub fn connect(host: &str) -> anyhow::Result<Self> {
        tracing::info!("Attempting SSH fallback for host: {}", host);
        
        // Basic reachability check
        let status = Command::new("ssh")
            .args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=5", host, "exit"])
            .status()?;

        if status.success() {
            Ok(Self {
                id: Uuid::new_v4(),
                host: host.to_string(),
            })
        } else {
            anyhow::bail!("SSH connection failed to {}", host)
        }
    }

    pub fn execute_command(&self, cmd: &str) -> anyhow::Result<String> {
        let output = Command::new("ssh")
            .arg(&self.host)
            .arg(cmd)
            .output()?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            anyhow::bail!("Remote command failed: {}", String::from_utf8_lossy(&output.stderr))
        }
    }
}

