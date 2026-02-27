use crate::platform::{Renderer, SurfaceConfig, SurfaceHandle, SurfaceContent, InputSource, RawInputEvent, SemanticEvent, SystemServices, SystemMetrics, ProcessHandle};
use crate::common::{DirectoryEntry};
use std::path::Path;

pub struct AndroidRenderer;

impl Renderer for AndroidRenderer {
    fn create_surface(&mut self, _config: SurfaceConfig) -> SurfaceHandle {
        SurfaceHandle(1)
    }
    fn update_surface(&mut self, _handle: SurfaceHandle, _content: &dyn SurfaceContent) {}
    fn composite(&mut self) {}
}

pub struct AndroidInput;

impl InputSource for AndroidInput {
    fn poll_events(&mut self) -> Vec<RawInputEvent> {
        vec![] // Simulated haptic/gesture input (§14.2)
    }
    fn map_to_semantic(&self, raw: RawInputEvent) -> Option<SemanticEvent> {
        match raw {
            RawInputEvent::TouchDown(_, _) => Some(SemanticEvent::ZoomIn),
            _ => None,
        }
    }
}

pub struct AndroidServices;

impl SystemServices for AndroidServices {
    fn spawn_process(&self, _cmd: &str, _args: &[&str]) -> anyhow::Result<ProcessHandle> {
        // NDK implementation placeholder
        Ok(ProcessHandle(5555))
    }

    fn read_dir(&self, _path: &Path) -> anyhow::Result<Vec<DirectoryEntry>> {
        // Scoped Storage implementation placeholder ($426)
        Ok(vec![])
    }

    fn get_system_metrics(&self) -> SystemMetrics {
        SystemMetrics { cpu_usage: 0.2, mem_usage: 2048 * 1024 }
    }

    fn open_url(&self, _url: &str) {
        // Intent placeholder
    }
}

