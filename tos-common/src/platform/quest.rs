use crate::DirectoryEntry;
use crate::platform::{
    InputSource, ProcessHandle, RawInputEvent, Renderer, SemanticEvent, SurfaceConfig,
    SurfaceContent, SurfaceHandle, SystemMetrics, SystemServices,
};
use std::path::Path;

pub struct QuestRenderer {
    pub surfaces: std::collections::HashMap<SurfaceHandle, QuestSurface>,
    pub next_handle: u32,
}

pub struct QuestSurface {
    pub config: SurfaceConfig,
    pub depth: u8,
    pub pid: Option<u32>,
    pub layer_type: CockpitLayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CockpitLayer {
    /// The primary 360-degree UI wrap.
    MainCylinder,
    /// Floating bezel or action panel.
    FloatingQuad,
    /// Level 3 application viewport.
    AppViewport,
}

impl QuestRenderer {
    pub fn new() -> Self {
        Self {
            surfaces: std::collections::HashMap::new(),
            next_handle: 100,
        }
    }

    fn resolve_layer(&self, config: &SurfaceConfig) -> CockpitLayer {
        if config.width > 2000 {
            CockpitLayer::MainCylinder
        } else if config.depth > 0 {
            CockpitLayer::FloatingQuad
        } else {
            CockpitLayer::AppViewport
        }
    }
}

impl Renderer for QuestRenderer {
    fn create_surface(&mut self, config: SurfaceConfig) -> SurfaceHandle {
        let handle = SurfaceHandle(self.next_handle);
        self.next_handle += 1;

        let layer_type = self.resolve_layer(&config);
        
        tracing::info!(
            "Allocating OpenXR Swapchain Surface: {}x{} as {:?}",
            config.width,
            config.height,
            layer_type
        );

        self.surfaces.insert(handle, QuestSurface {
            config,
            depth: 0,
            pid: None,
            layer_type,
        });

        handle
    }

    fn update_surface(&mut self, handle: SurfaceHandle, content: &dyn SurfaceContent) {
        if let Some(_surface) = self.surfaces.get(&handle) {
            tracing::debug!("xrAcquireSwapchainImage for handle {}", handle.0);
            tracing::debug!("xrWaitSwapchainImage for handle {}", handle.0);

            let data = content.pixel_data();
            if !data.is_empty() {
                tracing::debug!("Copied {} bytes to OpenXR Swapchain image", data.len());
            }

            tracing::debug!("xrReleaseSwapchainImage for handle {}", handle.0);
        }
    }

    fn set_surface_depth(&mut self, handle: SurfaceHandle, depth: u8) {
        if let Some(surface) = self.surfaces.get_mut(&handle) {
            surface.depth = depth;
            tracing::debug!("QuestRenderer: Updated depth for handle {} to {}", handle.0, depth);
        }
    }

    fn register_pid(&mut self, pid: u32, handle: SurfaceHandle) {
        if let Some(surface) = self.surfaces.get_mut(&handle) {
            surface.pid = Some(pid);
        }
    }

    fn composite(&mut self) {
        tracing::debug!("xrBeginFrame: Synchronizing with XR runtime");
        
        // In a real OpenXR implementation, we would iterate through surfaces
        // and create xr::CompositionLayerProjection and xr::CompositionLayerQuad
        let cylinder_count = self.surfaces.values().filter(|s| s.layer_type == CockpitLayer::MainCylinder).count();
        let quad_count = self.surfaces.values().filter(|s| s.layer_type == CockpitLayer::FloatingQuad || s.layer_type == CockpitLayer::AppViewport).count();
        
        tracing::debug!(
            "xrEndFrame: Compositing {} Cylinder layers and {} Quad layers",
            cylinder_count,
            quad_count
        );
    }
}

pub struct QuestInput {
    pub mapping: crate::DeviceMapping,
}

impl Default for QuestInput {
    fn default() -> Self {
        Self {
            mapping: crate::DeviceMapping::default(),
        }
    }
}

impl InputSource for QuestInput {
    fn poll_events(&mut self) -> Vec<RawInputEvent> {
        // Mocking some OpenXR events that would normally come from the XR runtime
        vec![
            RawInputEvent::SpatialGesture {
                hand: "left".to_string(),
                gesture: "pinch_left".to_string(),
            },
            RawInputEvent::SpatialGaze {
                x: 0.0,
                y: 0.0,
                z: -1.0,
                dwell_ms: 600,
            },
        ]
    }

    fn map_to_semantic(&self, raw: RawInputEvent) -> Option<SemanticEvent> {
        match raw {
            RawInputEvent::KeyDown(k) if k == "X" => Some(SemanticEvent::ZoomOut),
            RawInputEvent::ControllerButtonDown { button, .. } => {
                self.mapping.lookup_button(&button).and_then(|action| match action {
                    "select" => Some(SemanticEvent::Select("default".to_string())),
                    "home" => Some(SemanticEvent::Home),
                    "zoom_out" => Some(SemanticEvent::ZoomOut),
                    "zoom_in" => Some(SemanticEvent::ZoomIn),
                    "command_hub" => Some(SemanticEvent::CommandHub),
                    "toggle_bezel" => Some(SemanticEvent::ToggleBezel),
                    _ => None,
                })
            }
            RawInputEvent::SpatialGesture { gesture, .. } => {
                self.mapping.lookup_gesture(&gesture).and_then(|action| match action {
                    "zoom_out" => Some(SemanticEvent::ZoomOut),
                    "zoom_in" => Some(SemanticEvent::ZoomIn),
                    "open_hub" => Some(SemanticEvent::CommandHub),
                    _ => None,
                })
            }
            RawInputEvent::SpatialGaze { dwell_ms, .. } if dwell_ms >= self.mapping.gaze_dwell_ms => {
                Some(SemanticEvent::Select("gaze_target".to_string()))
            }
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
        SystemMetrics {
            cpu_usage: 0.3,
            mem_usage: 4096 * 1024,
        }
    }

    fn open_url(&self, _url: &str) {}
}
