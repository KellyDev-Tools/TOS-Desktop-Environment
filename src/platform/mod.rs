pub mod linux;
// Android Face lives in its own crate: android-handheld/
// Build separately with: make android-build
pub mod quest;
pub mod mock;
pub mod remote;
pub mod remote_server;
pub mod remote_session;
pub mod ssh_fallback;
pub mod headless;

pub use headless::HeadlessRenderer;
pub use remote::RemoteRenderer;
pub use remote_server::RemoteServer;

use std::path::Path;
use std::sync::Arc;


/// §15.1: Core Platform Traits

pub struct SurfaceConfig {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SurfaceHandle(pub u32);

pub trait SurfaceContent {
    fn pixel_data(&self) -> &[u8] {
        &[]
    }
    fn text_data(&self) -> Option<&str> {
        None
    }
}

pub trait Renderer: Send {
    fn create_surface(&mut self, config: SurfaceConfig) -> SurfaceHandle;
    fn update_surface(&mut self, handle: SurfaceHandle, content: &dyn SurfaceContent);
    fn register_pid(&mut self, pid: u32, handle: SurfaceHandle);
    fn composite(&mut self);
    fn get_capture_backend(&self) -> Arc<dyn crate::services::capture::CaptureBackend> {
        Arc::new(crate::services::capture::MockCaptureBackend)
    }
}

pub enum RawInputEvent {
    KeyDown(String),
    TouchDown(f32, f32),
    Click(f32, f32),
}

pub use tos_protocol::ipc::SemanticEvent;

pub trait InputSource {
    fn poll_events(&mut self) -> Vec<RawInputEvent>;
    fn map_to_semantic(&self, raw: RawInputEvent) -> Option<SemanticEvent>;
}

pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub mem_usage: u64,
}

pub struct ProcessHandle(pub u32);

pub trait SystemServices {
    fn spawn_process(&self, cmd: &str, args: &[&str]) -> anyhow::Result<ProcessHandle>;
    fn read_dir(&self, path: &Path) -> anyhow::Result<Vec<crate::common::DirectoryEntry>>;
    fn get_system_metrics(&self) -> SystemMetrics;
    fn open_url(&self, url: &str);
}
