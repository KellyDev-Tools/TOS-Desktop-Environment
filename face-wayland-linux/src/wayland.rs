use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryHandler, RegistryState},
    shell::{
        wlr_layer::{Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
        xdg::{
            window::{Window as XdgWindow, WindowConfigure as XdgWindowConfigure, WindowHandler as XdgWindowHandler, WindowDecorations},
            XdgShell,
        },
    },
    shm::{Shm, ShmHandler},
};
use wayland_client::{protocol::wl_surface, Connection, QueueHandle};

pub struct WaylandShell {
    pub connection: Connection,
    pub queue_handle: QueueHandle<WaylandState>,
    pub event_queue: wayland_client::EventQueue<WaylandState>,
    pub state: WaylandState,
}

pub struct WaylandState {
    pub registry_state: RegistryState,
    pub compositor_state: CompositorState,
    pub shm: Shm,
    pub layer_shell: Option<LayerShell>,
    pub xdg_shell: Option<XdgShell>,
    pub output_state: OutputState,
}

impl WaylandShell {
    /// Check if Wayland compositor is reachable without blocking.
    /// Returns true only if connection can be established immediately.
    pub fn can_connect() -> bool {
        match std::env::var("WAYLAND_DISPLAY") {
            Ok(_) => {
                // Attempt connection with short timeout/non-blocking if possible.
                // Sctk doesn't have a direct "timeout" but we can try to connect_to_env
                // which fails quickly if not found.
                match Connection::connect_to_env() {
                    Ok(_) => true,
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    pub fn new() -> Option<Self> {
        let conn = match Connection::connect_to_env() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Wayland Connection failed: {:?}", e);
                return None;
            }
        };
        let (globals, event_queue) = match wayland_client::globals::registry_queue_init(&conn) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Wayland Registry queue init failed: {:?}", e);
                return None;
            }
        };
        let qh = event_queue.handle();

        let registry_state = RegistryState::new(&globals);
        let compositor_state = match CompositorState::bind(&globals, &qh) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Wayland CompositorState bind failed: {:?}", e);
                return None;
            }
        };
        let shm = match Shm::bind(&globals, &qh) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Wayland Shm bind failed: {:?}", e);
                return None;
            }
        };
        let layer_shell = LayerShell::bind(&globals, &qh).ok();
        let xdg_shell = XdgShell::bind(&globals, &qh).ok();

        if layer_shell.is_none() && xdg_shell.is_none() {
            tracing::warn!("Neither LayerShell nor XdgShell supported on this compositor.");
            return None;
        }

        let output_state = OutputState::new(&globals, &qh);

        let state = WaylandState {
            registry_state,
            compositor_state,
            shm,
            layer_shell,
            xdg_shell,
            output_state,
        };

        Some(Self {
            connection: conn,
            queue_handle: qh,
            event_queue,
            state,
        })
    }

    pub fn create_layer_surface(&mut self, title: &str, width: u32, height: u32) {
        let surface = self
            .state
            .compositor_state
            .create_surface(&self.queue_handle);

        if let Some(ref layer_shell) = self.state.layer_shell {
            let layer_surface = layer_shell.create_layer_surface(
                &self.queue_handle,
                surface.clone(),
                Layer::Top,
                Some("tos-native"),
                None,
            );
            layer_surface.set_size(width, height);
            surface.commit();
            tracing::info!("Wayland: Real Layer Surface created ({}x{})", width, height);
        } else if let Some(ref xdg_shell) = self.state.xdg_shell {
            let window = xdg_shell.create_window(surface.clone(), WindowDecorations::RequestServer, &self.queue_handle);
            window.set_title(title.to_string());
            // Standard windows might need an app_id for some DEs
            window.set_app_id("org.tos.native-shell".to_string());
            surface.commit();
            tracing::info!("Wayland: Fallback XDG Window created ({}x{})", width, height);
        }
    }

    pub fn dispatch(&mut self) {
        let _ = self.event_queue.dispatch_pending(&mut self.state);
        let _ = self.connection.flush();
    }
}

impl ProvidesRegistryState for WaylandState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    fn runtime_add_global(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _name: u32,
        _interface: &str,
        _version: u32,
    ) {
    }
    fn runtime_remove_global(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _name: u32,
        _interface: &str,
    ) {
    }
}

impl RegistryHandler<WaylandState> for WaylandState {
    fn new_global(
        _state: &mut WaylandState,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _name: u32,
        _interface: &str,
        _version: u32,
    ) {
    }
    fn remove_global(
        _state: &mut WaylandState,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _name: u32,
        _interface: &str,
    ) {
    }
}

impl CompositorHandler for WaylandState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }
    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wayland_client::protocol::wl_output::Transform,
    ) {
    }
    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
    }
}

impl ShmHandler for WaylandState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl OutputHandler for WaylandState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }
    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }
    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for WaylandState {
    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer_surface: &LayerSurface,
        _configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
    }

    fn closed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer_surface: &LayerSurface,
    ) {
    }
}


impl XdgWindowHandler for WaylandState {
    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _window: &XdgWindow,
        _configure: XdgWindowConfigure,
        _serial: u32,
    ) {
    }

    fn request_close(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _window: &XdgWindow) {
    }
}

delegate_registry!(WaylandState);
delegate_compositor!(WaylandState);
delegate_shm!(WaylandState);
delegate_xdg_shell!(WaylandState);
delegate_xdg_window!(WaylandState);
delegate_layer!(WaylandState);
delegate_output!(WaylandState);
