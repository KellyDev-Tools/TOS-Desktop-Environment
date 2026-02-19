//! Platform Abstraction Layer (§10, §21)
//! 
//! Provides traits for decoupling core TOS logic from platform-specific 
//! implementations (Linux Wayland, Android XR, Android Phone).

use std::path::{Path, PathBuf};
use crate::HierarchyLevel;

/// Configuration for creating a surface
pub struct SurfaceConfig {
    pub title: String,
    pub class: String,
    pub initial_level: HierarchyLevel,
}

/// Opaque handle to a platform surface
pub struct SurfaceHandle(pub u64);

/// Trait for platform renderers (§10.1)
pub trait Renderer {
    fn create_surface(&mut self, config: SurfaceConfig) -> SurfaceHandle;
    fn update_surface(&mut self, handle: SurfaceHandle);
    fn composite(&mut self);
    fn set_bezel_visible(&mut self, handle: SurfaceHandle, visible: bool);
}

/// Trait for platform input sources (§10.2)
pub trait InputSource {
    fn poll_events(&mut self) -> Vec<crate::system::input::SemanticEvent>;
}

/// System metrics data
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub mem_usage: f32,
    pub battery_level: Option<f32>,
}

/// Trait for platform-specific system services (§10.3)
pub trait SystemServices {
    fn spawn_process(&self, cmd: &str, args: &[&str]) -> Result<u32, String>;
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, String>;
    fn get_system_metrics(&self) -> SystemMetrics;
    fn open_url(&self, url: &str);
}
