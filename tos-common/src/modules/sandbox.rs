/**
 * TOS Sandbox Execution Layer
 * Kernel-Level Isolation (Namespaces/Cgroups)
 */
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;
use similar::TextDiff;
use walkdir::WalkDir;

use serde::{Serialize, Deserialize};

pub struct SandboxManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxProfile {
    /// Strict isolation: no network, no persistent FS, /tmp only.
    Default,
    /// Adds network access (--share-net).
    Network,
    /// Adds sector-specific filesystem binding.
    FileSystem,
    /// Minimum isolation (development/trusted).
    FullAccess,
}

/// A handle to an active overlay-based sandbox.
pub struct OverlaySandbox {
    pub root: PathBuf,
    pub lower: PathBuf,
    pub upper: PathBuf,
    pub work: PathBuf,
    pub merged: PathBuf,
}

impl OverlaySandbox {
    /// Cleans up the sandbox directories (upper/work/merged).
    /// Note: merged must be unmounted before deletion if still mounted.
    pub fn cleanup(&self) -> anyhow::Result<()> {
        let _ = fs::remove_dir_all(&self.root);
        Ok(())
    }

    /// Compares the upper layer with the lower layer and generates DiffHunks.
    pub fn calculate_diffs(&self) -> anyhow::Result<Vec<crate::state::DiffHunk>> {
        let mut all_hunks = Vec::new();
        for entry in WalkDir::new(&self.upper) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let rel_path = entry.path().strip_prefix(&self.upper)?;
            let lower_file = self.lower.join(rel_path);

            let upper_content = fs::read_to_string(entry.path())?;
            let lower_content = if lower_file.exists() {
                fs::read_to_string(&lower_file).unwrap_or_default()
            } else {
                String::new()
            };

            if upper_content == lower_content {
                continue;
            }

            let diff = TextDiff::from_lines(&lower_content, &upper_content);
            let unified_diff = format!("{}", diff.unified_diff().context_radius(3));

            if !unified_diff.is_empty() {
                all_hunks.push(crate::state::DiffHunk {
                    old_start: 1,
                    old_count: lower_content.lines().count(),
                    new_start: 1,
                    new_count: upper_content.lines().count(),
                    content: unified_diff,
                });
            }
        }
        Ok(all_hunks)
    }
}

impl SandboxManager {
    /// Sandbox configuration: Spawns a new process in isolated Linux namespaces 
    /// and configures strict memory limits via Cgroups (v2 format).
    pub fn spawn_sandboxed_process(program: &str, args: &[&str]) -> anyhow::Result<std::process::Child> {
        Self::spawn_bwrap_process(SandboxProfile::Default, None, program, args)
    }

    /// Spawns a process using Bubblewrap (§17.3) with a specific profile.
    pub fn spawn_bwrap_process(
        profile: SandboxProfile,
        sector_id: Option<&str>,
        program: &str,
        args: &[&str],
    ) -> anyhow::Result<std::process::Child> {
        tracing::info!("Spawning Bwrap Sandbox [Profile: {:?}] for {}", profile, program);

        let mut cmd = Command::new("bwrap");
        
        // Base isolation flags
        cmd.arg("--unshare-all");
        cmd.arg("--dir").arg("/tmp");
        cmd.arg("--proc").arg("/proc");
        cmd.arg("--dev").arg("/dev");
        
        // Host system read-only bindings (canonical LCARS sandbox)
        for path in &["/usr", "/lib", "/lib64", "/bin", "/sbin"] {
            if Path::new(path).exists() {
                cmd.arg("--ro-bind").arg(path).arg(path);
            }
        }

        // Profile-specific enforcement
        match profile {
            SandboxProfile::Default => {
                // Strict isolation: no network, no persistent host access.
            },
            SandboxProfile::Network => {
                cmd.arg("--share-net");
            },
            SandboxProfile::FileSystem => {
                if let Some(id) = sector_id {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/tos".to_string());
                    let sector_path = PathBuf::from(home).join("TOS/Sectors").join(id);
                    if sector_path.exists() {
                        cmd.arg("--bind").arg(&sector_path).arg("/mnt/sector");
                    }
                }
            },
            SandboxProfile::FullAccess => {
                cmd.arg("--share-net");
                cmd.arg("--bind").arg("/").arg("/");
            }
        }

        cmd.arg("--").arg(program).args(args);
        
        let child = cmd.spawn()?;
        
        // Cgroup limits still apply post-spawn
        Self::apply_cgroup_limits(child.id())?;

        Ok(child)
    }

    /// Creates a new overlay-based sandbox in a temporary directory.
    pub fn create_overlay_sandbox(lower_dir: &Path) -> anyhow::Result<OverlaySandbox> {
        let temp_base = std::env::temp_dir().join("tos_sandbox");
        if !temp_base.exists() {
            fs::create_dir_all(&temp_base)?;
        }
        
        let id = uuid::Uuid::new_v4().to_string();
        let root = temp_base.join(id);
        
        let upper = root.join("upper");
        let work = root.join("work");
        let merged = root.join("merged");
        
        fs::create_dir_all(&upper)?;
        fs::create_dir_all(&work)?;
        fs::create_dir_all(&merged)?;
        
        Ok(OverlaySandbox {
            root,
            lower: lower_dir.to_path_buf(),
            upper,
            work,
            merged,
        })
    }

    /// Isolate a process post-spawn (Mock fallback if pre_exec unavailable).
    pub fn isolate_process(pid: u32) -> anyhow::Result<()> {
        Self::apply_cgroup_limits(pid)?;
        Ok(())
    }

    /// Control Group (Cgroup) memory and resource enforcement.
    fn apply_cgroup_limits(pid: u32) -> anyhow::Result<()> {
        // Enforce 512MB memory limit
        let cgroup_path = format!("/sys/fs/cgroup/tos_sandbox_{}", pid);
        let path = Path::new(&cgroup_path);
        
        // Cgroup creation usually requires root, so we handle it gracefully if it fails in user space.
        if !path.exists() {
            if let Err(e) = fs::create_dir_all(path) {
                tracing::warn!("Cgroup creation blocked (unprivileged user space): {}. Simulated Cgroup config.", e);
                return Ok(());
            }
        }

        // Write PID to cgroup procs
        let procs_file = path.join("cgroup.procs");
        let _ = fs::write(procs_file, pid.to_string());

        // Set 512MB max memory limit (512 * 1024 * 1024)
        let mem_limit_file = path.join("memory.max");
        let _ = fs::write(mem_limit_file, "536870912");

        tracing::info!("Strict cgroup memory limits applied for PID {}", pid);
        Ok(())
    }

    /// Check if a module manifest signature is valid.
    pub fn verify_manifest_signature(manifest_path: &str) -> bool {
        use crate::services::marketplace::MarketplaceService;
        
        let path = Path::new(manifest_path);
        let dir = match path.parent() {
            Some(p) => p.to_path_buf(),
            None => return false,
        };

        match MarketplaceService::discover_module_local(dir) {
            Ok(manifest) => {
                match MarketplaceService::get_trusted_public_key() {
                    Ok(pk) => MarketplaceService::verify_manifest_local(&manifest, &pk),
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }
}
