use std::sync::Arc;
use tos_lib::brain::Brain;
use tos_lib::face::Face;
use tos_lib::platform::linux::LinuxRenderer;

fn main() -> anyhow::Result<()> {
    // 1. Initial Identity & Discovery Gate (§4.1)
    // The Wayland Face acts as a privileged system client.
    tracing_subscriber::fmt::init();
    tracing::info!("TOS-WAYLAND-FACE: Initializing Native Linux Display Engine...");

    // 2. State & IPC Bootstrap
    // For the Beta-0 RC standalone binary, we instantiate the Brain core directly.
    // In a full multi-user deployment, this would connect to a shared daemon.
    let brain = Brain::new()?;
    
    // 3. Renderer Deployment
    // Instantiate the Wayland/DMABUF backend defined in platform/linux/mod.rs
    let renderer = Box::new(LinuxRenderer::new());
    
    // 4. Face Cycle
    let mut face = Face::new(brain.state.clone(), brain.ipc.clone()).with_renderer(renderer);
    
    tracing::info!("TOS-WAYLAND-FACE: Display Cycle Active. Syncing to Wayland Compositor.");
    
    loop {
        // §6.1: Native Composition Cycle
        face.render();
        
        // Simple 60FPS throttle for the RC build
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
