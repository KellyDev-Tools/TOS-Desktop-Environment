use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// §15.1: Core Platform Traits
pub struct SurfaceConfig {
    pub width: u32,
    pub height: u32,
    pub depth: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SurfaceHandle(pub u32);

/// Result of a frame capture operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameCapture {
    pub data: String,
    pub width: u32,
    pub height: u32,
}

/// Platform-agnostic interface for capturing window frames.
pub trait CaptureBackend: Send + Sync {
    fn capture_window(&self, pid: u32) -> Option<FrameCapture>;
}

pub struct MockCaptureBackend;
impl CaptureBackend for MockCaptureBackend {
    fn capture_window(&self, _pid: u32) -> Option<FrameCapture> {
        None
    }
}

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
    fn set_surface_depth(&mut self, handle: SurfaceHandle, depth: u8);
    fn register_pid(&mut self, pid: u32, handle: SurfaceHandle);
    fn composite(&mut self);
    fn get_capture_backend(&self) -> Arc<dyn CaptureBackend> {
        Arc::new(MockCaptureBackend)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RawInputEvent {
    KeyDown(String),
    TouchDown(f32, f32),
    Click(f32, f32),
    /// Game controller button press (e.g. "South", "East", "Start").
    ControllerButtonDown { controller_id: u32, button: String },
    /// Game controller axis movement (e.g. "LeftStickX", "RightTrigger").
    ControllerAxisMove { controller_id: u32, axis: String, value: f32 },
    /// XR / Spatial hand gesture (e.g. "pinch_left", "wrist_tap").
    SpatialGesture { hand: String, gesture: String },
    /// XR / Spatial gaze or eye tracking event.
    SpatialGaze { x: f32, y: f32, z: f32, dwell_ms: u32 },
}

pub use crate::ipc::SemanticEvent;

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
    fn read_dir(&self, path: &Path) -> anyhow::Result<Vec<crate::DirectoryEntry>>;
    fn get_system_metrics(&self) -> SystemMetrics;
    fn open_url(&self, url: &str);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformStatus {
    pub name: String,
    pub version: String,
    pub uptime: u64,
}

pub trait AppPlatform: Send + Sync {
    fn get_status(&self) -> PlatformStatus;
    fn get_services(&self) -> Arc<dyn SystemServices>;
    fn get_capture(&self) -> Arc<dyn CaptureBackend>;
}
