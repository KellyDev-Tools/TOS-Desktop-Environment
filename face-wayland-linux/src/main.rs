use face_wayland_linux::LinuxRenderer;
use std::sync::{Arc, Mutex};
use tos_common::platform::Renderer;
use tos_common::state::TosState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("[FACE-WAYLAND] Starting TOS Wayland Face...");

    let _state = Arc::new(Mutex::new(TosState::default()));
    let mut renderer = LinuxRenderer::new();

    let surface_config = tos_common::platform::SurfaceConfig {
        width: 1280,
        height: 720,
    };

    let _handle = renderer.create_surface(surface_config);
    tracing::info!("[FACE-WAYLAND] Surface created. Entering composition loop...");

    loop {
        renderer.composite();
        tokio::time::sleep(std::time::Duration::from_millis(16)).await; // ~60fps
    }
}
