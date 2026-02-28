use crate::platform::{Renderer, SurfaceConfig, SurfaceHandle, SurfaceContent, InputSource, RawInputEvent, SemanticEvent, SystemServices, SystemMetrics, ProcessHandle};
use crate::common::{DirectoryEntry};
use std::path::Path;
use std::process::Command;

use std::os::unix::io::RawFd;
use std::ptr;
use std::ffi::CString;

pub struct LinuxRenderer {
    surfaces: std::collections::HashMap<u32, WaylandBuffer>,
    next_handle: u32,
}

struct WaylandBuffer {
    fd: RawFd,
    size: usize,
    width: u32,
    height: u32,
    memory_ptr: *mut libc::c_void,
}

impl Drop for WaylandBuffer {
    fn drop(&mut self) {
        unsafe {
            if !self.memory_ptr.is_null() && self.memory_ptr != libc::MAP_FAILED {
                libc::munmap(self.memory_ptr, self.size);
            }
            if self.fd >= 0 {
                libc::close(self.fd);
            }
        }
    }
}

impl LinuxRenderer {
    pub fn new() -> Self {
        Self {
            surfaces: std::collections::HashMap::new(),
            next_handle: 1,
        }
    }

    /// Allocates a zero-copy buffer backed by an anonymous file descriptor (memfd).
    /// This acts as the physical memory region shared directly with the Wayland Compositor.
    fn allocate_dmabuf_shm(width: u32, height: u32) -> anyhow::Result<WaylandBuffer> {
        let size = (width * height * 4) as usize; // ARGB8888 32-bit format

        unsafe {
            let name = CString::new("tos_wayland_surface").unwrap();
            let fd = libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING);
            if fd < 0 {
                return Err(anyhow::anyhow!("Failed to allocate memfd for Wayland buffer"));
            }

            if libc::ftruncate(fd, size as libc::off_t) < 0 {
                libc::close(fd);
                return Err(anyhow::anyhow!("Failed to truncate Wayland buffer"));
            }

            // Seal the file to prevent resizing, required by strict Wayland compositors
            libc::fcntl(fd, libc::F_ADD_SEALS, libc::F_SEAL_SHRINK | libc::F_SEAL_GROW | libc::F_SEAL_SEAL);

            let mem = libc::mmap(
                ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            );

            if mem == libc::MAP_FAILED {
                libc::close(fd);
                return Err(anyhow::anyhow!("Failed to map Wayland zero-copy buffer"));
            }

            tracing::info!("Allocated Zero-Copy Wayland Buffer: {}x{} ({} bytes, FD: {})", width, height, size, fd);

            Ok(WaylandBuffer {
                fd,
                size,
                width,
                height,
                memory_ptr: mem,
            })
        }
    }
}

impl Renderer for LinuxRenderer {
    fn create_surface(&mut self, config: SurfaceConfig) -> SurfaceHandle {
        let handle_id = self.next_handle;
        self.next_handle += 1;

        match Self::allocate_dmabuf_shm(config.width, config.height) {
            Ok(buffer) => {
                self.surfaces.insert(handle_id, buffer);
            }
            Err(e) => {
                tracing::error!("Wayland allocation failed: {}", e);
            }
        }

        SurfaceHandle(handle_id)
    }

    fn update_surface(&mut self, handle: SurfaceHandle, _content: &dyn SurfaceContent) {
        if let Some(buf) = self.surfaces.get_mut(&handle.0) {
            tracing::debug!("Synchronizing Wayland Buffer FD: {}", buf.fd);
            // In a full implementation, content pixel data is copied directly to `buf.memory_ptr`
            // and the compositor is notified via `wl_surface_attach` and `wl_surface_commit`.
        }
    }

    fn composite(&mut self) {
        tracing::debug!("Triggering native Wayland composition cycle.");
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    struct MockContent;
    impl SurfaceContent for MockContent {}

    #[test]
    fn test_linux_renderer_allocation() {
        let mut renderer = LinuxRenderer::new();
        
        let config = SurfaceConfig {
            width: 1920,
            height: 1080,
        };

        let handle = renderer.create_surface(config);
        assert!(handle.0 > 0, "Invalid handle ID returned");

        let buffer = renderer.surfaces.get(&handle.0).expect("Surface buffer missing from tracking map");
        
        let expected_size = 1920 * 1080 * 4;
        assert_eq!(buffer.size, expected_size, "Memfd map size incorrect");
        assert!(buffer.fd > 0, "Invalid file descriptor assigned for DMABUF");
        assert!(!buffer.memory_ptr.is_null());
        assert_ne!(buffer.memory_ptr, libc::MAP_FAILED);
    }
}
