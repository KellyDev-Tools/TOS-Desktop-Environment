use crate::common::DirectoryEntry;
use crate::platform::{
    InputSource, ProcessHandle, RawInputEvent, Renderer, SemanticEvent, SurfaceConfig,
    SurfaceContent, SurfaceHandle, SystemMetrics, SystemServices,
};
use std::path::Path;

pub struct RemoteRenderer;

impl Renderer for RemoteRenderer {
    fn create_surface(&mut self, _config: SurfaceConfig) -> SurfaceHandle {
        // §12.1: Route surface creation command over WebSocket/WebRTC
        SurfaceHandle(100)
    }
    fn update_surface(&mut self, _handle: SurfaceHandle, _content: &dyn SurfaceContent) {}
    fn register_pid(&mut self, _pid: u32, _handle: SurfaceHandle) {}
    fn composite(&mut self) {}
}

pub struct RemoteInput;

impl InputSource for RemoteInput {
    fn poll_events(&mut self) -> Vec<RawInputEvent> {
        vec![] // Events received from remote host via WebSocket
    }
    fn map_to_semantic(&self, _raw: RawInputEvent) -> Option<SemanticEvent> {
        None
    }
}

pub struct RemoteServices;

impl SystemServices for RemoteServices {
    fn spawn_process(&self, _cmd: &str, _args: &[&str]) -> anyhow::Result<ProcessHandle> {
        // §12.1: JSON-RPC over WebSocket to spawn on remote host
        Ok(ProcessHandle(1337))
    }

    fn read_dir(&self, _path: &Path) -> anyhow::Result<Vec<DirectoryEntry>> {
        // §27.3: Remote Directory fallback logic
        Ok(vec![])
    }

    fn get_system_metrics(&self) -> SystemMetrics {
        SystemMetrics {
            cpu_usage: 0.0,
            mem_usage: 0,
        }
    }

    fn open_url(&self, _url: &str) {
        // Option to open on local viewer or remote host
    }
}
