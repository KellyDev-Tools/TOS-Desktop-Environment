use face_wayland_linux::wayland::WaylandShell;
use std::time::Duration;

#[test]
fn test_wayland_connection_and_registry() {
    // This test requires a running Wayland compositor (WAYLAND_DISPLAY).
    // If not present, we skip or pass if it fails gracefully.
    if !WaylandShell::can_connect() {
        println!("Skipping real Wayland test: WAYLAND_DISPLAY not found or compositor unreachable.");
        return;
    }

    let mut shell = WaylandShell::new().expect("Failed to initialize WaylandShell even though can_connect() was true");
    
    // Dispatch a few times to let the registry sync
    for _ in 0..5 {
        shell.dispatch();
        std::thread::sleep(Duration::from_millis(10));
    }

    // Verify we have basic globals
    assert!(shell.state.xdg_shell.is_some() || shell.state.layer_shell.is_some(), 
        "Compositor must support either xdg_shell or layer_shell");
    
    println!("Wayland Integration: Successfully connected and verified shells.");
}

#[test]
fn test_layer_surface_creation() {
    if !WaylandShell::can_connect() {
        return;
    }

    let mut shell = WaylandShell::new().unwrap();
    
    // Attempt to create a surface
    shell.create_layer_surface("Integration Test Surface", 100, 100);
    
    // Dispatch to ensure the request is sent
    shell.dispatch();
    
    // If we didn't panic or crash, it's a good sign for a headless/integration test
    println!("Wayland Integration: Surface creation request dispatched.");
}
