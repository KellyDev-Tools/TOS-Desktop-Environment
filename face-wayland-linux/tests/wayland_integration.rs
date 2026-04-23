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

#[test]
fn test_dmabuf_buffer_attachment() {
    if !WaylandShell::can_connect() {
        return;
    }

    let mut shell = WaylandShell::new().unwrap();
    
    // Create a surface
    let surface = shell.create_layer_surface("DMABUF Test", 640, 480);
    
    // Create a dummy buffer (memfd)
    // We use libc directly to simulate the renderer's allocation
    unsafe {
        let name = std::ffi::CString::new("test_buffer").unwrap();
        let fd = libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC);
        assert!(fd >= 0, "Failed to create memfd for test");
        
        let size = 640 * 480 * 4;
        libc::ftruncate(fd, size as libc::off_t);
        
        // Try to attach the buffer
        // This will exercise the DMABUF path (or SHM fallback)
        shell.attach_buffer(&surface, fd, 640, 480);
        
        // Dispatch to process protocol messages
        shell.dispatch();
        
        libc::close(fd);
    }
    
    println!("Wayland Integration: DMABUF/SHM buffer attachment test passed.");
}
