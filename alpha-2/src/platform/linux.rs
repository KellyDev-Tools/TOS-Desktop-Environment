use crate::platform::{Renderer, SurfaceConfig, SurfaceHandle, SurfaceContent, InputSource, RawInputEvent, SemanticEvent, SystemServices, SystemMetrics, ProcessHandle};
use crate::common::{DirectoryEntry};
use std::path::Path;
use std::process::Command;

pub struct LinuxRenderer;

impl Renderer for LinuxRenderer {
    fn create_surface(&mut self, _config: SurfaceConfig) -> SurfaceHandle {
        SurfaceHandle(0) // Placeholder
    }
    fn update_surface(&mut self, _handle: SurfaceHandle, _content: &dyn SurfaceContent) {}
    fn composite(&mut self) {}
}

pub struct LinuxInput;

impl InputSource for LinuxInput {
    fn poll_events(&mut self) -> Vec<RawInputEvent> {
        vec![] // Placeholder
    }
    fn map_to_semantic(&self, _raw: RawInputEvent) -> Option<SemanticEvent> {
        None
    }
}

pub struct LinuxServices;

impl SystemServices for LinuxServices {
    fn spawn_process(&self, cmd: &str, args: &[&str]) -> anyhow::Result<ProcessHandle> {
        let child = Command::new(cmd)
            .args(args)
            .spawn()?;
        Ok(ProcessHandle(child.id()))
    }

    fn read_dir(&self, path: &Path) -> anyhow::Result<Vec<DirectoryEntry>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            entries.push(DirectoryEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
            });
        }
        Ok(entries)
    }

    fn get_system_metrics(&self) -> SystemMetrics {
        SystemMetrics {
            cpu_usage: 0.0,
            mem_usage: 0,
        }
    }

    fn open_url(&self, url: &str) {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
}
