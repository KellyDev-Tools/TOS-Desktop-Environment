use crate::platform::{Renderer, SurfaceConfig, SurfaceHandle, SurfaceContent, InputSource, RawInputEvent, SemanticEvent, SystemServices, SystemMetrics, ProcessHandle};
use crate::common::{DirectoryEntry};
use std::path::Path;

pub struct MockRenderer;

impl Renderer for MockRenderer {
    fn create_surface(&mut self, _config: SurfaceConfig) -> SurfaceHandle {
        SurfaceHandle(999)
    }
    fn update_surface(&mut self, _handle: SurfaceHandle, _content: &dyn SurfaceContent) {}
    fn composite(&mut self) {}
}

pub struct MockInput;

impl InputSource for MockInput {
    fn poll_events(&mut self) -> Vec<RawInputEvent> {
        vec![] // Simulated events can be injected here
    }
    fn map_to_semantic(&self, raw: RawInputEvent) -> Option<SemanticEvent> {
        match raw {
            RawInputEvent::KeyDown(k) if k == "z" => Some(SemanticEvent::ZoomIn),
            _ => None,
        }
    }
}

pub struct MockServices;

impl SystemServices for MockServices {
    fn spawn_process(&self, _cmd: &str, _args: &[&str]) -> anyhow::Result<ProcessHandle> {
        Ok(ProcessHandle(1234))
    }

    fn read_dir(&self, _path: &Path) -> anyhow::Result<Vec<DirectoryEntry>> {
        Ok(vec![
            DirectoryEntry { name: "test_file.txt".to_string(), is_dir: false, size: 1024 },
            DirectoryEntry { name: "test_dir".to_string(), is_dir: true, size: 0 },
        ])
    }

    fn get_system_metrics(&self) -> SystemMetrics {
        SystemMetrics { cpu_usage: 0.1, mem_usage: 1024 * 1024 }
    }

    fn open_url(&self, _url: &str) {}
}

