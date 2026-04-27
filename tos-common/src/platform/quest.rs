use crate::DirectoryEntry;
use crate::platform::{
    InputSource, ProcessHandle, RawInputEvent, Renderer, SemanticEvent, SurfaceConfig,
    SurfaceContent, SurfaceHandle, SystemMetrics, SystemServices,
};
use std::path::Path;

pub struct QuestRenderer;

impl Renderer for QuestRenderer {
    fn create_surface(&mut self, config: SurfaceConfig) -> SurfaceHandle {
        tracing::info!(
            "Allocating OpenXR Swapchain Surface: {}x{}",
            config.width,
            config.height
        );
        SurfaceHandle(77) // Placeholder for actual xrCreateSwapchain
    }

    fn update_surface(&mut self, handle: SurfaceHandle, content: &dyn SurfaceContent) {
        tracing::debug!("xrAcquireSwapchainImage for handle {}", handle.0);
        tracing::debug!("xrWaitSwapchainImage for handle {}", handle.0);

        let data = content.pixel_data();
        if !data.is_empty() {
            tracing::debug!("Copied {} bytes to OpenXR Swapchain image", data.len());
        }

        tracing::debug!("xrReleaseSwapchainImage for handle {}", handle.0);
    }

    fn set_surface_depth(&mut self, _handle: SurfaceHandle, _depth: u8) {
        tracing::debug!("QuestRenderer: Setting depth for throttle logic");
    }

    fn register_pid(&mut self, _pid: u32, _handle: SurfaceHandle) {}

    fn composite(&mut self) {
        tracing::debug!("xrEndFrame: Compositing OpenXR projection layers");
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
