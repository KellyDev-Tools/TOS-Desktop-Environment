//! Android Services — system-level operations for the Android Face.

use std::path::Path;
use tos_common::state::DirectoryEntry;
use tos_common::platform::{
    Renderer, SurfaceConfig, SurfaceHandle, SurfaceContent,
    SystemServices, SystemMetrics, ProcessHandle,
};

use crate::api;

/// Android system services — clipboard, files, notifications, etc.
#[derive(Default)]
pub struct AndroidServices;

impl SystemServices for AndroidServices {
    fn spawn_process(&self, cmd: &str, args: &[&str]) -> anyhow::Result<ProcessHandle> {
        // On Android: spawn via Intent or NDK exec
        tracing::debug!("Android Face: Spawning process: {} {:?}", cmd, args);
        Ok(ProcessHandle(5555))
    }

    fn read_dir(&self, path: &Path) -> anyhow::Result<Vec<DirectoryEntry>> {
        // On Android: use Storage Access Framework (SAF)
        tracing::debug!("Android Face: Reading directory: {:?}", path);
        Ok(vec![])
    }

    fn get_system_metrics(&self) -> SystemMetrics {
        let mem = api::get_memory_info();
        SystemMetrics {
            cpu_usage: 0.5, // placeholder — real impl reads /proc/stat
            mem_usage: mem.total_memory - mem.free_memory,
        }
    }

    fn open_url(&self, url: &str) {
        // On Android: Intent.ACTION_VIEW + Uri.parse(url)
        tracing::info!("Android Face: Opening URL: {}", url);
    }
}

impl Renderer for AndroidServices {
    fn create_surface(&mut self, _config: SurfaceConfig) -> SurfaceHandle {
        // On Android: create EGL surface from NativeWindow
        SurfaceHandle(1)
    }

    fn update_surface(&mut self, _handle: SurfaceHandle, _content: &dyn SurfaceContent) {}

    fn register_pid(&mut self, _pid: u32, _handle: SurfaceHandle) {}

    fn composite(&mut self) {}
}
