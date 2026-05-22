use face_wayland_linux::LinuxRenderer;
use std::sync::{Arc, Mutex};
use tos_common::platform::Renderer;
use tos_common::state::TosState;

struct TextContent(String);
impl tos_common::platform::SurfaceContent for TextContent {
    fn text_data(&self) -> Option<&str> {
        Some(&self.0)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("[FACE-WAYLAND] PANIC: {:?}", info);
    }));

    tracing_subscriber::fmt::init();
    eprintln!("[FACE-WAYLAND] Starting TOS Wayland Face...");

    let _state = Arc::new(Mutex::new(TosState::default()));
    let mut renderer = LinuxRenderer::new();

    let surface_config = tos_common::platform::SurfaceConfig {
        width: 1280,
        height: 720,
        depth: 1,
    };

    eprintln!("[FACE-WAYLAND] Creating surface...");
    let handle = renderer.create_surface(surface_config);
    eprintln!("[FACE-WAYLAND] Surface created with handle {:?}. Waiting for handshake...", handle);
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    eprintln!("[FACE-WAYLAND] Entering composition loop...");

    let start_time = std::time::Instant::now();

    loop {
        let uptime = start_time.elapsed().as_secs();
        let content = TextContent(format!(
            "TOS SYSTEM ONLINE\n\nPlatform: Linux Wayland\nUptime: {}s\nResolution: 1280x720\n\n[COMM-LINK] Brain Sync Active",
            uptime
        ));

        renderer.update_surface(handle, &content);
        renderer.composite();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await; // Slower for debug
    }
}
