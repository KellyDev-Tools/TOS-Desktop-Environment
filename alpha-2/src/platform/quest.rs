use crate::platform::{Renderer, SurfaceConfig, SurfaceHandle, SurfaceContent, InputSource, RawInputEvent, SemanticEvent, SystemServices, SystemMetrics, ProcessHandle};
use crate::common::{DirectoryEntry};
use std::path::Path;

pub struct QuestRenderer;

impl Renderer for QuestRenderer {
    fn create_surface(&mut self, _config: SurfaceConfig) -> SurfaceHandle {
        // OpenXR Swapchain implementation placeholder §16.1
        SurfaceHandle(77)
    }
    fn update_surface(&mut self, _handle: SurfaceHandle, _content: &dyn SurfaceContent) {}
    fn composite(&mut self) {}
}

pub struct QuestInput;

impl InputSource for QuestInput {
    fn poll_events(&mut self) -> Vec<RawInputEvent> {
        vec![] // OpenXR hand/trigger input
    }
    fn map_to_semantic(&self, raw: RawInputEvent) -> Option<SemanticEvent> {
        match raw {
            RawInputEvent::KeyDown(k) if k == "X" => Some(SemanticEvent::ZoomOut),
            _ => None,
        }
    }
}

pub struct QuestServices;

impl SystemServices for QuestServices {
    fn spawn_process(&self, _cmd: &str, _args: &[&str]) -> anyhow::Result<ProcessHandle> {
        Ok(ProcessHandle(8888))
    }

    fn read_dir(&self, _path: &Path) -> anyhow::Result<Vec<DirectoryEntry>> {
        Ok(vec![])
    }

    fn get_system_metrics(&self) -> SystemMetrics {
        SystemMetrics { cpu_usage: 0.3, mem_usage: 4096 * 1024 }
    }

    fn open_url(&self, _url: &str) {}
}

