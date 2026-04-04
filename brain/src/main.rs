use tos_common::brain::Brain;
// use tos_common::{Face, MockFace};
use std::env;
use std::time::Instant;
use tos_common::platform::RemoteServer;
// use tos_common::brain::renderer_manager::RendererMode;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let start_time = Instant::now();
    // Tracing writes to stderr so it doesn't interleave with the
    // Face's terminal dashboard on stdout. Defaults to WARN level
    // unless RUST_LOG is set for finer control.
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(env_filter)
        .init();

    let args: Vec<String> = env::args().collect();
    let _is_self_test = args.iter().any(|arg| arg == "--self-test");
    let _is_headless = args.iter().any(|arg| arg == "--headless");

    // 1. Initialize Brain Core
    let brain = Brain::new()?;
    let ipc = brain.ipc.clone();
    let _state = brain.state.clone();

    // 2. Initialize Renderer for Capture/Thumbnails
    let render_mode = tos_common::brain::renderer_manager::RendererManager::detect();
    let renderer = tos_common::brain::renderer_manager::RendererManager::init(render_mode)?;
    let capture_backend = renderer.get_capture_backend();
    brain.services.capture.set_backend(capture_backend);

    // 3. Start IPC Server (TCP 7000, WS 7001)
    let server = RemoteServer::new(ipc.clone());
    tokio::spawn(async move {
        if let Err(e) = server.run(7000).await {
            tracing::error!("[BRAIN] IPC Server failure: {}", e);
        }
    });

    let elapsed = start_time.elapsed();
    tracing::info!("[BRAIN] TOS Daemon initialized in {:.2?}", elapsed);
    tracing::info!("[BRAIN] Serving IPC on 7000/7001. Press Ctrl+C to stop.");

    // Park the main loop until Ctrl+C
    tokio::signal::ctrl_c().await?;
    tracing::info!("[BRAIN] Shutting down.");

    Ok(())
}
