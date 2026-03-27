// Wayland Compositor Backend
// Based on "Native Desktop Architecture.md" — Smithay-based Wayland compositor
//
// This module defines the TOS compositor's Wayland protocol handling:
// - Client connection management
// - Surface lifecycle (create, commit, destroy)
// - Seat handling (keyboard, pointer, touch)
// - XWayland support for legacy apps
// - Server-Side Decoration (SSD) for LCARS frames
//
// The actual Smithay integration is gated behind the "compositor" feature flag.
// When compiled without it, this module provides the data structures and
// logic that will connect to Smithay once the dependency is available.

use std::collections::HashMap;
use std::time::Instant;

// ─── Wayland Surface Types ─────────────────────────────

/// Unique identifier for a Wayland client
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(pub u32);

/// Unique identifier for a wl_surface
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WlSurfaceId(pub u64);

/// The role assigned to a surface
#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceRole {
    /// A standard top-level window (xdg_toplevel)
    Toplevel(ToplevelState),
    /// A popup/dropdown attached to a parent surface
    Popup { parent: WlSurfaceId },
    /// An XWayland window (legacy X11 app)
    XWayland(XWaylandState),
    /// The TOS LCARS overlay layer (always on top)
    LcarsOverlay,
    /// A subsurface (child of another surface)
    Subsurface { parent: WlSurfaceId },
}

/// State for an xdg_toplevel surface
#[derive(Debug, Clone, PartialEq)]
pub struct ToplevelState {
    pub title: String,
    pub app_id: String,
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,
    pub decorations: DecorationMode,
    pub activated: bool,
    pub fullscreen: bool,
    pub maximized: bool,
    pub resizing: bool,
}

impl Default for ToplevelState {
    fn default() -> Self {
        Self {
            title: String::new(),
            app_id: String::new(),
            min_size: None,
            max_size: None,
            decorations: DecorationMode::ServerSide, // TOS always prefers SSD
            activated: false,
            fullscreen: false,
            maximized: false,
            resizing: false,
        }
    }
}

/// Decoration mode for windows
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecorationMode {
    /// TOS provides LCARS-style window decorations (default)
    ServerSide,
    /// The application draws its own decorations (CSD)
    ClientSide,
    /// Legacy mode: app has its own titlebar, TOS adds minimal border
    Legacy,
}

/// State for an XWayland surface
#[derive(Debug, Clone, PartialEq)]
pub struct XWaylandState {
    pub window_id: u32,
    pub title: String,
    pub class: String,
    pub override_redirect: bool, // popup/tooltip that bypasses WM
    pub transient_for: Option<u32>,
}

impl Default for XWaylandState {
    fn default() -> Self {
        Self {
            window_id: 0,
            title: String::new(),
            class: String::new(),
            override_redirect: false,
            transient_for: None,
        }
    }
}

/// A tracked Wayland surface with all its state
#[derive(Debug)]
pub struct WlSurface {
    pub id: WlSurfaceId,
    pub client_id: ClientId,
    pub role: Option<SurfaceRole>,
    /// Position in compositor space (pixels)
    pub position: (i32, i32),
    /// Surface dimensions from last commit
    pub size: (u32, u32),
    /// The surface's buffer has been committed
    pub committed: bool,
    /// Which sector this surface belongs to in the TOS hierarchy
    pub tos_sector: Option<u32>,
    /// GPU texture handle for this surface's content
    pub texture_id: Option<super::gpu::TextureId>,
    /// Whether this surface should use direct scanout
    pub direct_scanout: bool,
    /// Whether input should be forwarded to this surface
    pub receives_input: bool,
    /// Creation timestamp
    pub created_at: Instant,
}

// ─── Seat (Input Device Handling) ───────────────────────

/// Input seat state for keyboard, pointer, and touch
#[derive(Debug)]
pub struct CompositorSeat {
    pub name: String,
    /// Which surface currently has keyboard focus
    pub keyboard_focus: Option<WlSurfaceId>,
    /// Which surface is under the pointer
    pub pointer_focus: Option<WlSurfaceId>,
    /// Pointer position (global compositor coords)
    pub pointer_position: (f64, f64),
    /// Currently pressed keys (keycodes)
    pub pressed_keys: Vec<u32>,
    /// Active modifiers
    pub modifiers: ModifierState,
    /// Pointer button state
    pub pointer_buttons: u32,
}

#[derive(Debug, Clone, Default)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool, // Super/Meta
}

impl CompositorSeat {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            keyboard_focus: None,
            pointer_focus: None,
            pointer_position: (0.0, 0.0),
            pressed_keys: Vec::new(),
            modifiers: ModifierState::default(),
            pointer_buttons: 0,
        }
    }
}

// ─── Compositor Backend ─────────────────────────────────

/// Events emitted by the Wayland compositor to the TOS logic layer
#[derive(Debug, Clone)]
pub enum CompositorEvent {
    /// A new client connected
    ClientConnected(ClientId),
    /// A client disconnected
    ClientDisconnected(ClientId),
    /// A new toplevel surface was created
    ToplevelCreated {
        surface_id: WlSurfaceId,
        client_id: ClientId,
        app_id: String,
        title: String,
    },
    /// A toplevel surface's title changed
    ToplevelTitleChanged {
        surface_id: WlSurfaceId,
        title: String,
    },
    /// A surface committed a new buffer (content updated)
    SurfaceCommitted {
        surface_id: WlSurfaceId,
        size: (u32, u32),
    },
    /// A surface was destroyed
    SurfaceDestroyed(WlSurfaceId),
    /// An XWayland window mapped
    XWaylandMapped {
        surface_id: WlSurfaceId,
        window_id: u32,
        class: String,
        title: String,
    },
    /// Keyboard key event
    KeyInput {
        keycode: u32,
        pressed: bool,
        modifiers: ModifierState,
    },
    /// Pointer motion
    PointerMotion {
        x: f64,
        y: f64,
    },
    /// Pointer button
    PointerButton {
        button: u32,
        pressed: bool,
    },
    /// A new output was hotplugged
    OutputConnected {
        name: String,
        width: u32,
        height: u32,
        refresh: u32,
    },
    /// An output was disconnected
    OutputDisconnected {
        name: String,
    },
}

/// The Wayland compositor backend.
/// Manages client connections, surface state, and input routing.
pub struct WaylandBackend {
    surfaces: HashMap<WlSurfaceId, WlSurface>,
    clients: HashMap<ClientId, Vec<WlSurfaceId>>,
    seat: CompositorSeat,
    /// Event queue for the TOS logic layer to consume
    event_queue: Vec<CompositorEvent>,
    /// Next surface ID
    next_surface_id: u64,
    /// Next client ID
    next_client_id: u32,
    /// Whether XWayland is enabled
    pub xwayland_enabled: bool,
    /// XWayland process PID (if running)
    xwayland_pid: Option<u32>,
    /// Wayland socket path
    pub socket_path: String,
}

impl WaylandBackend {
    pub fn new() -> Self {
        let socket_path = format!("wayland-tos-{}", std::process::id());
        println!("[Wayland] Backend initialized (socket: {})", socket_path);

        Self {
            surfaces: HashMap::new(),
            clients: HashMap::new(),
            seat: CompositorSeat::new("seat0"),
            event_queue: Vec::new(),
            next_surface_id: 1,
            next_client_id: 1,
            xwayland_enabled: true,
            xwayland_pid: None,
            socket_path,
        }
    }

    // ─── Client Management ─────────────────────────────

    /// Register a new Wayland client connection
    pub fn client_connect(&mut self) -> ClientId {
        let id = ClientId(self.next_client_id);
        self.next_client_id += 1;
        self.clients.insert(id, Vec::new());
        self.event_queue.push(CompositorEvent::ClientConnected(id));
        println!("[Wayland] Client {} connected", id.0);
        id
    }

    /// Handle client disconnection — destroy all associated surfaces
    pub fn client_disconnect(&mut self, client_id: ClientId) {
        if let Some(surface_ids) = self.clients.remove(&client_id) {
            for sid in surface_ids {
                self.surfaces.remove(&sid);
                self.event_queue.push(CompositorEvent::SurfaceDestroyed(sid));
            }
        }
        self.event_queue.push(CompositorEvent::ClientDisconnected(client_id));
        println!("[Wayland] Client {} disconnected", client_id.0);
    }

    // ─── Surface Lifecycle ─────────────────────────────

    /// Create a new wl_surface for a client
    pub fn create_surface(&mut self, client_id: ClientId) -> WlSurfaceId {
        let id = WlSurfaceId(self.next_surface_id);
        self.next_surface_id += 1;

        let surface = WlSurface {
            id,
            client_id,
            role: None,
            position: (0, 0),
            size: (0, 0),
            committed: false,
            tos_sector: None,
            texture_id: None,
            direct_scanout: false,
            receives_input: true,
            created_at: Instant::now(),
        };

        self.surfaces.insert(id, surface);
        self.clients.entry(client_id).or_default().push(id);
        id
    }

    /// Assign the xdg_toplevel role to a surface
    pub fn assign_toplevel_role(&mut self, surface_id: WlSurfaceId, app_id: &str, title: &str) {
        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            let toplevel = ToplevelState {
                title: title.to_string(),
                app_id: app_id.to_string(),
                ..ToplevelState::default()
            };
            surface.role = Some(SurfaceRole::Toplevel(toplevel));
            self.event_queue.push(CompositorEvent::ToplevelCreated {
                surface_id,
                client_id: surface.client_id,
                app_id: app_id.to_string(),
                title: title.to_string(),
            });
            println!("[Wayland] Surface {} assigned toplevel role ({})", surface_id.0, app_id);
        }
    }

    /// Handle an xdg_toplevel set_title request
    pub fn set_toplevel_title(&mut self, surface_id: WlSurfaceId, title: &str) {
        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            if let Some(SurfaceRole::Toplevel(ref mut state)) = surface.role {
                state.title = title.to_string();
                self.event_queue.push(CompositorEvent::ToplevelTitleChanged {
                    surface_id,
                    title: title.to_string(),
                });
            }
        }
    }

    /// Handle a surface commit (new buffer content available)
    pub fn commit_surface(&mut self, surface_id: WlSurfaceId, width: u32, height: u32) {
        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            surface.size = (width, height);
            surface.committed = true;
            self.event_queue.push(CompositorEvent::SurfaceCommitted {
                surface_id,
                size: (width, height),
            });
        }
    }

    /// Destroy a surface
    pub fn destroy_surface(&mut self, surface_id: WlSurfaceId) {
        if let Some(surface) = self.surfaces.remove(&surface_id) {
            // Remove from client's surface list
            if let Some(client_surfaces) = self.clients.get_mut(&surface.client_id) {
                client_surfaces.retain(|&s| s != surface_id);
            }
            // Clear focus if this was the focused surface
            if self.seat.keyboard_focus == Some(surface_id) {
                self.seat.keyboard_focus = None;
            }
            if self.seat.pointer_focus == Some(surface_id) {
                self.seat.pointer_focus = None;
            }
            self.event_queue.push(CompositorEvent::SurfaceDestroyed(surface_id));
            println!("[Wayland] Surface {} destroyed", surface_id.0);
        }
    }

    // ─── XWayland ──────────────────────────────────────

    /// Map an XWayland window to a Wayland surface
    pub fn map_xwayland_window(&mut self, surface_id: WlSurfaceId, window_id: u32, class: &str, title: &str) {
        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            let xw_state = XWaylandState {
                window_id,
                title: title.to_string(),
                class: class.to_string(),
                ..XWaylandState::default()
            };
            surface.role = Some(SurfaceRole::XWayland(xw_state));
            self.event_queue.push(CompositorEvent::XWaylandMapped {
                surface_id,
                window_id,
                class: class.to_string(),
                title: title.to_string(),
            });
            println!("[XWayland] Window {} mapped (class: {}, title: {})", window_id, class, title);
        }
    }

    // ─── Input Routing ─────────────────────────────────

    /// Set keyboard focus to a specific surface
    pub fn set_keyboard_focus(&mut self, surface_id: Option<WlSurfaceId>) {
        // Deactivate old focused toplevel
        if let Some(old_id) = self.seat.keyboard_focus {
            if let Some(surface) = self.surfaces.get_mut(&old_id) {
                if let Some(SurfaceRole::Toplevel(ref mut state)) = surface.role {
                    state.activated = false;
                }
            }
        }

        // Activate new focused toplevel
        if let Some(new_id) = surface_id {
            if let Some(surface) = self.surfaces.get_mut(&new_id) {
                if let Some(SurfaceRole::Toplevel(ref mut state)) = surface.role {
                    state.activated = true;
                }
            }
        }

        self.seat.keyboard_focus = surface_id;
    }

    /// Route a keyboard event
    pub fn handle_key_input(&mut self, keycode: u32, pressed: bool) {
        if pressed {
            self.seat.pressed_keys.push(keycode);
        } else {
            self.seat.pressed_keys.retain(|&k| k != keycode);
        }

        self.event_queue.push(CompositorEvent::KeyInput {
            keycode,
            pressed,
            modifiers: self.seat.modifiers.clone(),
        });
    }

    /// Route pointer motion
    pub fn handle_pointer_motion(&mut self, x: f64, y: f64) {
        self.seat.pointer_position = (x, y);

        // Hit-test: find which surface is under the pointer
        let hit = self.surfaces.values()
            .filter(|s| s.receives_input && s.committed)
            .find(|s| {
                let (sx, sy) = s.position;
                let (sw, sh) = s.size;
                x >= sx as f64 && x < (sx as u32 + sw) as f64 &&
                y >= sy as f64 && y < (sy as u32 + sh) as f64
            })
            .map(|s| s.id);

        if hit != self.seat.pointer_focus {
            self.seat.pointer_focus = hit;
        }

        self.event_queue.push(CompositorEvent::PointerMotion { x, y });
    }

    /// Route pointer button
    pub fn handle_pointer_button(&mut self, button: u32, pressed: bool) {
        if pressed {
            self.seat.pointer_buttons |= 1 << button;
            // Click-to-focus: if we clicked a surface, give it keyboard focus
            if let Some(focus_id) = self.seat.pointer_focus {
                self.set_keyboard_focus(Some(focus_id));
            }
        } else {
            self.seat.pointer_buttons &= !(1 << button);
        }

        self.event_queue.push(CompositorEvent::PointerButton { button, pressed });
    }

    // ─── TOS Integration ───────────────────────────────

    /// Assign a surface to a TOS sector
    pub fn assign_to_sector(&mut self, surface_id: WlSurfaceId, sector: u32) {
        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            surface.tos_sector = Some(sector);
            println!("[Wayland/TOS] Surface {} → Sector {}", surface_id.0, sector);
        }
    }

    /// Set the position and size of a surface (called by TOS layout engine)
    pub fn configure_surface(&mut self, surface_id: WlSurfaceId, x: i32, y: i32, width: u32, height: u32) {
        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            surface.position = (x, y);
            // In a real compositor, this sends a configure event to the client
            println!("[Wayland] Surface {} configured: {}x{} at ({},{})", surface_id.0, width, height, x, y);
        }
    }

    /// Get the decoration mode for a surface
    pub fn get_decoration_mode(&self, surface_id: WlSurfaceId) -> DecorationMode {
        self.surfaces.get(&surface_id)
            .and_then(|s| match &s.role {
                Some(SurfaceRole::Toplevel(state)) => Some(state.decorations),
                Some(SurfaceRole::XWayland(_)) => Some(DecorationMode::ServerSide),
                _ => None,
            })
            .unwrap_or(DecorationMode::ServerSide)
    }

    // ─── Queries ───────────────────────────────────────

    /// Get a surface by ID
    pub fn get_surface(&self, id: WlSurfaceId) -> Option<&WlSurface> {
        self.surfaces.get(&id)
    }

    /// Get all toplevel surfaces
    pub fn get_toplevels(&self) -> Vec<&WlSurface> {
        self.surfaces.values()
            .filter(|s| matches!(s.role, Some(SurfaceRole::Toplevel(_)) | Some(SurfaceRole::XWayland(_))))
            .collect()
    }

    /// Get surfaces in a specific sector
    pub fn get_surfaces_in_sector(&self, sector: u32) -> Vec<&WlSurface> {
        self.surfaces.values()
            .filter(|s| s.tos_sector == Some(sector))
            .collect()
    }

    /// Get the currently focused surface
    pub fn get_focused_surface(&self) -> Option<&WlSurface> {
        self.seat.keyboard_focus.and_then(|id| self.surfaces.get(&id))
    }

    /// Total number of surfaces
    pub fn surface_count(&self) -> usize {
        self.surfaces.len()
    }

    /// Total number of connected clients
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    // ─── Event Dispatch ────────────────────────────────

    /// Drain all pending events (consumed by the TOS logic layer)
    pub fn drain_events(&mut self) -> Vec<CompositorEvent> {
        std::mem::take(&mut self.event_queue)
    }

    /// Check if there are pending events
    pub fn has_events(&self) -> bool {
        !self.event_queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        let backend = WaylandBackend::new();
        assert_eq!(backend.surface_count(), 0);
        assert_eq!(backend.client_count(), 0);
        assert!(backend.socket_path.starts_with("wayland-tos-"));
    }

    #[test]
    fn test_client_lifecycle() {
        let mut backend = WaylandBackend::new();

        let client = backend.client_connect();
        assert_eq!(backend.client_count(), 1);

        let surface = backend.create_surface(client);
        assert_eq!(backend.surface_count(), 1);

        backend.client_disconnect(client);
        assert_eq!(backend.client_count(), 0);
        assert_eq!(backend.surface_count(), 0); // Surfaces cleaned up
    }

    #[test]
    fn test_toplevel_creation() {
        let mut backend = WaylandBackend::new();
        let client = backend.client_connect();
        let surface_id = backend.create_surface(client);

        backend.assign_toplevel_role(surface_id, "org.mozilla.firefox", "Firefox");

        let surface = backend.get_surface(surface_id).unwrap();
        match &surface.role {
            Some(SurfaceRole::Toplevel(state)) => {
                assert_eq!(state.app_id, "org.mozilla.firefox");
                assert_eq!(state.title, "Firefox");
                assert_eq!(state.decorations, DecorationMode::ServerSide);
            }
            _ => panic!("Expected Toplevel role"),
        }
    }

    #[test]
    fn test_surface_commit() {
        let mut backend = WaylandBackend::new();
        let client = backend.client_connect();
        let surface_id = backend.create_surface(client);

        backend.commit_surface(surface_id, 800, 600);

        let surface = backend.get_surface(surface_id).unwrap();
        assert!(surface.committed);
        assert_eq!(surface.size, (800, 600));
    }

    #[test]
    fn test_keyboard_focus() {
        let mut backend = WaylandBackend::new();
        let client = backend.client_connect();
        let s1 = backend.create_surface(client);
        let s2 = backend.create_surface(client);

        backend.assign_toplevel_role(s1, "app1", "App 1");
        backend.assign_toplevel_role(s2, "app2", "App 2");

        // Focus s1
        backend.set_keyboard_focus(Some(s1));
        let surf1 = backend.get_surface(s1).unwrap();
        if let Some(SurfaceRole::Toplevel(state)) = &surf1.role {
            assert!(state.activated);
        }

        // Focus s2 — s1 should be deactivated
        backend.set_keyboard_focus(Some(s2));
        let surf1 = backend.get_surface(s1).unwrap();
        if let Some(SurfaceRole::Toplevel(state)) = &surf1.role {
            assert!(!state.activated);
        }
        let surf2 = backend.get_surface(s2).unwrap();
        if let Some(SurfaceRole::Toplevel(state)) = &surf2.role {
            assert!(state.activated);
        }
    }

    #[test]
    fn test_sector_assignment() {
        let mut backend = WaylandBackend::new();
        let client = backend.client_connect();
        let s1 = backend.create_surface(client);
        let s2 = backend.create_surface(client);

        backend.assign_to_sector(s1, 0);
        backend.assign_to_sector(s2, 1);

        let sector0 = backend.get_surfaces_in_sector(0);
        assert_eq!(sector0.len(), 1);
        assert_eq!(sector0[0].id, s1);

        let sector1 = backend.get_surfaces_in_sector(1);
        assert_eq!(sector1.len(), 1);
        assert_eq!(sector1[0].id, s2);
    }

    #[test]
    fn test_xwayland_mapping() {
        let mut backend = WaylandBackend::new();
        let client = backend.client_connect();
        let surface_id = backend.create_surface(client);

        backend.map_xwayland_window(surface_id, 12345, "gimp", "GNU Image Manipulation Program");

        let surface = backend.get_surface(surface_id).unwrap();
        match &surface.role {
            Some(SurfaceRole::XWayland(state)) => {
                assert_eq!(state.window_id, 12345);
                assert_eq!(state.class, "gimp");
                assert_eq!(state.title, "GNU Image Manipulation Program");
            }
            _ => panic!("Expected XWayland role"),
        }

        // XWayland surfaces should default to SSD
        assert_eq!(backend.get_decoration_mode(surface_id), DecorationMode::ServerSide);
    }

    #[test]
    fn test_event_queue() {
        let mut backend = WaylandBackend::new();

        let client = backend.client_connect();
        let surface_id = backend.create_surface(client);
        backend.assign_toplevel_role(surface_id, "test", "Test");

        let events = backend.drain_events();
        assert!(events.len() >= 2); // At least ClientConnected + ToplevelCreated

        // Queue should be empty after drain
        assert!(!backend.has_events());
    }

    #[test]
    fn test_toplevels_query() {
        let mut backend = WaylandBackend::new();
        let client = backend.client_connect();

        let s1 = backend.create_surface(client);
        let s2 = backend.create_surface(client);
        let s3 = backend.create_surface(client);

        backend.assign_toplevel_role(s1, "app1", "App 1");
        backend.assign_toplevel_role(s2, "app2", "App 2");
        // s3 has no role

        let toplevels = backend.get_toplevels();
        assert_eq!(toplevels.len(), 2);
    }

    #[test]
    fn test_surface_destroy() {
        let mut backend = WaylandBackend::new();
        let client = backend.client_connect();
        let s1 = backend.create_surface(client);

        backend.set_keyboard_focus(Some(s1));
        assert!(backend.get_focused_surface().is_some());

        backend.destroy_surface(s1);
        assert_eq!(backend.surface_count(), 0);
        assert!(backend.get_focused_surface().is_none()); // Focus cleared
    }

    #[test]
    fn test_pointer_hit_test() {
        let mut backend = WaylandBackend::new();
        let client = backend.client_connect();
        let s1 = backend.create_surface(client);

        // Position surface at (100, 100) with size 400x300
        backend.assign_toplevel_role(s1, "test", "Test");
        backend.configure_surface(s1, 100, 100, 400, 300);
        backend.commit_surface(s1, 400, 300);

        // Move pointer inside the surface
        backend.handle_pointer_motion(200.0, 200.0);
        assert_eq!(backend.seat.pointer_focus, Some(s1));

        // Move pointer outside
        backend.handle_pointer_motion(50.0, 50.0);
        assert_eq!(backend.seat.pointer_focus, None);
    }

    #[test]
    fn test_click_to_focus() {
        let mut backend = WaylandBackend::new();
        let client = backend.client_connect();

        let s1 = backend.create_surface(client);
        let s2 = backend.create_surface(client);

        backend.assign_toplevel_role(s1, "app1", "App 1");
        backend.assign_toplevel_role(s2, "app2", "App 2");

        // Position surfaces
        backend.configure_surface(s1, 0, 0, 500, 500);
        backend.commit_surface(s1, 500, 500);
        backend.configure_surface(s2, 500, 0, 500, 500);
        backend.commit_surface(s2, 500, 500);

        // Click on s2
        backend.handle_pointer_motion(600.0, 250.0);
        backend.handle_pointer_button(0, true);

        assert_eq!(backend.seat.keyboard_focus, Some(s2));
    }

    #[test]
    fn test_title_change_event() {
        let mut backend = WaylandBackend::new();
        let client = backend.client_connect();
        let s1 = backend.create_surface(client);
        backend.assign_toplevel_role(s1, "terminal", "Terminal");

        backend.drain_events(); // Clear initial events

        backend.set_toplevel_title(s1, "Terminal — /home/user");

        let events = backend.drain_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            CompositorEvent::ToplevelTitleChanged { surface_id, title } => {
                assert_eq!(*surface_id, s1);
                assert_eq!(title, "Terminal — /home/user");
            }
            _ => panic!("Expected ToplevelTitleChanged"),
        }
    }
}
