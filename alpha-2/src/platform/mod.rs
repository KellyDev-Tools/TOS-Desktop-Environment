pub mod linux;
pub mod android;
pub mod quest;
pub mod mock;
pub mod remote;
pub mod remote_server;
pub mod remote_session;
pub mod ssh_fallback;

use std::path::Path;
use crate::common::CommandHubMode;

/// §15.1: Core Platform Traits

pub struct SurfaceConfig {
    pub width: u32,
    pub height: u32,
}

pub struct SurfaceHandle(pub u32);

pub trait SurfaceContent {}

pub trait Renderer {
    fn create_surface(&mut self, config: SurfaceConfig) -> SurfaceHandle;
    fn update_surface(&mut self, handle: SurfaceHandle, content: &dyn SurfaceContent);
    fn composite(&mut self);
}

pub enum RawInputEvent {
    KeyDown(String),
    TouchDown(f32, f32),
    Click(f32, f32),
}

pub enum SemanticEvent {
    ZoomIn,
    ZoomOut,
    SetMode(CommandHubMode),
    PromptSubmit(String),
    SectorAction(String), // close, clone, freeze
}

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

