use std::path::Path;
use std::process::Command;
use tos_common::platform::{
    CaptureBackend, FrameCapture, InputSource, ProcessHandle, RawInputEvent, Renderer,
    SemanticEvent, SurfaceConfig, SurfaceContent, SurfaceHandle, SystemMetrics, SystemServices,
};
use tos_common::state::DirectoryEntry;

use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::ptr;

pub mod wayland;

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use tiny_skia::{Color, Paint, PixmapMut, Rect, Transform};

pub struct LinuxRenderer {
    pub surfaces: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u32, SurfaceState>>>,
    pub pid_map: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u32, u32>>>,
    next_handle: u32,
    shell: Option<wayland::WaylandShell>,
    font: Option<FontArc>,
}

pub struct SurfaceState {
    pub buffer: WaylandBuffer,
    pub wl_surface: Option<wayland_client::protocol::wl_surface::WlSurface>,
    pub depth: u8,
}

pub struct WaylandBuffer {
    fd: RawFd,
    size: usize,
    width: u32,
    height: u32,
    memory_ptr: *mut libc::c_void,
}

// SAFETY: WaylandBuffer owns its FD and memory mapping, which are thread-safe to move.
unsafe impl Send for WaylandBuffer {}
// SAFETY: LinuxRenderer synchronization is handled via internal Mutexes.
unsafe impl Send for LinuxRenderer {}

impl Drop for WaylandBuffer {
    fn drop(&mut self) {
        // SAFETY: Simple wrapper around standard libc cleanup syscalls.
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
        let tokens_json = include_str!("../../assets/design_tokens.json");
        let tokens: serde_json::Value = serde_json::from_str(tokens_json).unwrap_or_default();
        if let Some(primary) = tokens.pointer("/themes/dark/primary") {
            tracing::info!(
                "Wayland Renderer loaded theme tokens (Primary: {})",
                primary.as_str().unwrap_or("")
            );
        }

        let font_paths = [
            "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
        ];
        let mut font = None;
        for path in font_paths {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(f) = FontArc::try_from_vec(data) {
                    font = Some(f);
                    tracing::info!("Wayland Renderer loaded system font: {}", path);
                    break;
                }
            }
        }

        let shell = wayland::WaylandShell::new();
        if shell.is_none() {
            tracing::warn!("Wayland: No display found. Running in Headless/Capture-only mode.");
        }

        Self {
            surfaces: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            pid_map: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            next_handle: 1,
            shell,
            font,
        }
    }

    /// Allocates a zero-copy buffer backed by an anonymous file descriptor (memfd).
    /// This acts as the physical memory region shared directly with the Wayland Compositor.
    fn allocate_dmabuf_shm(width: u32, height: u32) -> anyhow::Result<WaylandBuffer> {
        let size = (width * height * 4) as usize; // ARGB8888 32-bit format

        // SAFETY: Invokes standard posix/linux memory-backed file creation and mapping.
        unsafe {
            let name = CString::new("tos_wayland_surface").unwrap();
            let fd = libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING);
            if fd < 0 {
                return Err(anyhow::anyhow!(
                    "Failed to allocate memfd for Wayland buffer"
                ));
            }

            if libc::ftruncate(fd, size as libc::off_t) < 0 {
                libc::close(fd);
                return Err(anyhow::anyhow!("Failed to truncate Wayland buffer"));
            }

            // Seal the file to prevent resizing, required by strict Wayland compositors
            libc::fcntl(
                fd,
                libc::F_ADD_SEALS,
                libc::F_SEAL_SHRINK | libc::F_SEAL_GROW | libc::F_SEAL_SEAL,
            );

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

            tracing::info!(
                "Allocated Zero-Copy Wayland Buffer: {}x{} ({} bytes, FD: {})",
                width,
                height,
                size,
                fd
            );

            Ok(WaylandBuffer {
                fd,
                size,
                width,
                height,
                memory_ptr: mem,
            })
        }
    }

    fn render_text_to_buffer(&self, buf: &mut WaylandBuffer, text: &str) {
        // SAFETY: Mapping exactly the size allocated in allocate_dmabuf_shm.
        let mut pixmap = unsafe {
            let slice = std::slice::from_raw_parts_mut(buf.memory_ptr as *mut u8, buf.size);
            PixmapMut::from_bytes(slice, buf.width, buf.height)
                .expect("Failed to create PixmapMut from Wayland buffer pointer")
        };

        // Clear with near-black (LCARS vibe)
        pixmap.fill(Color::from_rgba8(10, 10, 15, 255));

        if let Some(ref font) = self.font {
            let scale = PxScale::from(16.0);
            let scaled_font = font.as_scaled(scale);
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(0, 240, 255, 255)); // Cyan / LCARS accent

            let x_offset = 20.0f32;
            let mut y_offset = 30.0f32;

            for line in text.lines() {
                let mut x = x_offset;
                for c in line.chars() {
                    let glyph = scaled_font.scaled_glyph(c);
                    if let Some(_outline) = font.outline_glyph(glyph) {
                        // Drawing logic...
                        let rect = Rect::from_xywh(x, y_offset - 12.0, 8.0, 14.0).unwrap();
                        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                    }
                    x += 9.0;
                }
                y_offset += 18.0;
            }
        }
    }
}

impl Renderer for LinuxRenderer {
    fn create_surface(&mut self, config: SurfaceConfig) -> SurfaceHandle {
        let handle_id = self.next_handle;
        self.next_handle += 1;

        let mut wl_surface = None;
        if let Some(ref mut shell) = self.shell {
            wl_surface = Some(shell.create_layer_surface("TOS Native Layer", config.width, config.height));
        }

        match Self::allocate_dmabuf_shm(config.width, config.height) {
            Ok(buffer) => {
                self.surfaces.lock().unwrap().insert(handle_id, SurfaceState {
                    buffer,
                    wl_surface,
                    depth: config.depth,
                });
            }
            Err(e) => {
                tracing::error!("Wayland allocation failed: {}", e);
            }
        }

        SurfaceHandle(handle_id)
    }

    fn update_surface(&mut self, handle: SurfaceHandle, content: &dyn SurfaceContent) {
        let mut surfaces = self.surfaces.lock().unwrap();
        if let Some(state) = surfaces.get_mut(&handle.0) {
            let buf = &mut state.buffer;
            tracing::debug!("Synchronizing Wayland Buffer FD: {}", buf.fd);

            if let Some(text) = content.text_data() {
                self.render_text_to_buffer(buf, text);
            } else {
                let data = content.pixel_data();
                if !data.is_empty() && data.len() <= buf.size {
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            data.as_ptr(),
                            buf.memory_ptr as *mut u8,
                            data.len(),
                        );
                    }
                    tracing::debug!("Copied {} bytes to Wayland SHM", data.len());
                }
            }

            if let (Some(ref mut shell), Some(ref surface)) = (self.shell.as_mut(), state.wl_surface.as_ref()) {
                shell.attach_buffer(surface, buf.fd, buf.width as i32, buf.height as i32);
            }
        }
    }

    fn set_surface_depth(&mut self, handle: SurfaceHandle, depth: u8) {
        let mut surfaces = self.surfaces.lock().unwrap();
        if let Some(state) = surfaces.get_mut(&handle.0) {
            state.depth = depth;
            tracing::debug!("Wayland: Set surface {} depth to {}", handle.0, depth);
        }
    }

    fn register_pid(&mut self, pid: u32, handle: SurfaceHandle) {
        let mut pid_map = self.pid_map.lock().unwrap();
        pid_map.insert(pid, handle.0);
        tracing::info!(
            "Wayland: Associated PID {} with Surface Handle {}",
            pid,
            handle.0
        );
    }

    fn composite(&mut self) {
        tracing::debug!("Triggering Native OS Composition Cycle (§6.1)");

        if let Some(ref mut shell) = self.shell {
            shell.dispatch();
        }

        let surfaces = self.surfaces.lock().unwrap();
        let surface_count = surfaces.len();
        tracing::debug!(
            "GL/Vulkan: Compositing Layer Stack ({} surfaces active)",
            surface_count
        );

        for (handle, state) in surfaces.iter() {
            let buf = &state.buffer;
            
            // §16.1 Depth-based render throttling
            if state.depth > 2 {
                tracing::debug!("   [THROTTLED] Skipping full composition for background surface {} (depth: {})", handle, state.depth);
                continue; // Background levels are throttled/static
            }

            // Simulated GL render pass
            tracing::debug!(
                "   [PASS 1] Sampler2D(surface_handle={}) -> Texture0",
                handle
            );
            tracing::debug!("   [PASS 2] VertexShader: applying zoom_transform, border_scale");
            tracing::debug!(
                "   [PASS 3] FragmentShader: alpha_blend, depth_blur(FD: {})",
                buf.fd
            );
        }
    }

    fn get_capture_backend(&self) -> std::sync::Arc<dyn CaptureBackend> {
        std::sync::Arc::new(LinuxCaptureBackend::new(self))
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
        let child = Command::new(cmd).args(args).spawn()?;
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
        let mut mem_usage = 0;
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            let mut total = 0;
            let mut free = 0;
            let mut buffers = 0;
            let mut cached = 0;
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    total = line
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                } else if line.starts_with("MemFree:") {
                    free = line
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                } else if line.starts_with("Buffers:") {
                    buffers = line
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                } else if line.starts_with("Cached:") {
                    cached = line
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                }
            }
            if total > 0 {
                // Approximate used memory in KB
                mem_usage = total - free - buffers - cached;
            }
        }

        SystemMetrics {
            cpu_usage: 0.0,              // CPU usage requires temporal sampling, skipped
            mem_usage: mem_usage * 1024, // Convert KB to bytes
        }
    }

    fn open_url(&self, url: &str) {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_renderer_allocation() {
        let mut renderer = LinuxRenderer::new();

        let config = SurfaceConfig {
            width: 1920,
            height: 1080,
        };

        let handle = renderer.create_surface(config);
        assert!(handle.0 > 0, "Invalid handle ID returned");

        let buffer_lock = renderer.surfaces.lock().unwrap();
        let state = buffer_lock
            .get(&handle.0)
            .expect("Surface buffer missing from tracking map");
        let buffer = &state.buffer;

        let expected_size = 1920 * 1080 * 4;
        assert_eq!(buffer.size, expected_size, "Memfd map size incorrect");
        assert!(buffer.fd > 0, "Invalid file descriptor assigned for DMABUF");
        assert!(!buffer.memory_ptr.is_null());
        assert_ne!(buffer.memory_ptr, libc::MAP_FAILED);
    }
}

/// CaptureBackend implementation for Linux, leveraging the Wayland/DMABUF surfaces.
pub struct LinuxCaptureBackend {
    surfaces: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u32, SurfaceState>>>,
    pid_map: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u32, u32>>>,
}

impl LinuxCaptureBackend {
    pub fn new(renderer: &LinuxRenderer) -> Self {
        Self {
            surfaces: renderer.surfaces.clone(),
            pid_map: renderer.pid_map.clone(),
        }
    }
}

impl CaptureBackend for LinuxCaptureBackend {
    fn capture_window(&self, pid: u32) -> Option<FrameCapture> {
        let handle = {
            let pid_map = self.pid_map.lock().unwrap();
            *pid_map.get(&pid)?
        };

        let surfaces = self.surfaces.lock().unwrap();
        let state = surfaces.get(&handle)?;
        let buffer = &state.buffer;

        // For Alpha-2.2, we convert the raw SHM/DMABUF data to a Base64 PNG.
        // In the future, we could pass the FD directly for zero-copy.

        // SAFETY: Buffer is guaranteed valid for the lifetime of the surfaces lock.
        unsafe {
            let data_slice =
                std::slice::from_raw_parts(buffer.memory_ptr as *const u8, buffer.size);

            // Basic RGBA -> PNG encoding (simulated for now, would use a crate like `image` or `png` in production)
            use base64::{engine::general_purpose, Engine as _};
            let encoded = general_purpose::STANDARD.encode(data_slice);
            let data_url = format!("data:image/raw;base64,{}", encoded);

            Some(FrameCapture {
                data: data_url,
                width: buffer.width,
                height: buffer.height,
            })
        }
    }
}
