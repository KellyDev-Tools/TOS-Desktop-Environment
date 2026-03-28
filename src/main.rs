use tos_lib::brain::Brain;
use tos_lib::face::{Face, MockFace};
use tos_lib::platform::RemoteServer;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use std::env;

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
    let is_self_test = args.iter().any(|arg| arg == "--self-test");
    let is_headless = args.iter().any(|arg| arg == "--headless");

    // 1. Initialize Brain Core
    let brain = Brain::new()?;
    let ipc = brain.ipc.clone();
    let state = brain.state.clone();

    if is_self_test {
        // --- Self-Test Demo Mode ---
        let mut face_raw = Face::new(state.clone(), ipc.clone());
        #[cfg(target_os = "linux")]
        {
            let renderer = tos_lib::platform::linux::LinuxRenderer::new();
            let backend = renderer.get_capture_backend();
            brain.services.capture.set_backend(std::sync::Arc::new(backend));
            face_raw = face_raw.with_renderer(Box::new(renderer));
        }
        let mut mock_face = MockFace(face_raw);

        tracing::info!("\n--- SYSTEM SELF-TEST SEQUENCE ---");
        sleep(Duration::from_secs(1)).await;

        mock_face.0.render();
        sleep(Duration::from_secs(2)).await;
        mock_face.simulate_bezel_zoom_in();
        mock_face.0.render();
        sleep(Duration::from_secs(2)).await;
        mock_face.simulate_prompt_submit("ls -la");
        sleep(Duration::from_secs(1)).await;
        mock_face.0.render();
        sleep(Duration::from_secs(2)).await;

        tracing::info!("\nSELF-TEST SEQUENCE COMPLETE.");
    } else {
        // Start IPC Server (TCP 7000, WS 7001, UDS)
        let server = RemoteServer::new(ipc.clone());
        tokio::spawn(async move {
            if let Err(e) = server.run(7000).await {
                tracing::error!("[BRAIN] IPC Server failure: {}", e);
            }
        });

        if is_headless {
            let elapsed = start_time.elapsed();
            // --- Headless Server Mode (for `make run-web`) ---
            // No terminal dashboard — just serve IPC to the Web Face.
            tracing::info!("[BRAIN] System initialized in {:.2?}", elapsed);
            tracing::info!("[BRAIN] Headless mode — serving IPC on 7000/7001. No terminal dashboard.");
            tracing::info!("[BRAIN] Press Ctrl+C to stop.");

            // Park the main task; the Tokio runtime keeps IPC tasks alive.
            loop {
                sleep(Duration::from_secs(60)).await;
            }
        } else {
            // --- Terminal Dashboard Mode (standalone) ---
            let mut face_raw = Face::new(state.clone(), ipc.clone());
            #[cfg(target_os = "linux")]
            {
                let renderer = tos_lib::platform::linux::LinuxRenderer::new();
                let backend = renderer.get_capture_backend();
                brain.services.capture.set_backend(std::sync::Arc::new(backend));
                face_raw = face_raw.with_renderer(Box::new(renderer));
            }

            let elapsed = start_time.elapsed();
            tracing::info!("[BRAIN] System initialized in {:.2?}. Terminal dashboard active. IPC on 7000/7001.", elapsed);

            loop {
                tracing::info!("[BRAIN] Life Signal: OS composition cycle active...");
                face_raw.render();
                sleep(Duration::from_millis(1000)).await;
            }
        }
    }
    
    Ok(())
}
