use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_registry, delegate_shm, delegate_output,
    registry::{ProvidesRegistryState, RegistryState, RegistryHandler},
    shm::{ShmHandler, Shm},
    output::{OutputHandler, OutputState},
    shell::wlr_layer::{LayerShell, LayerSurface, LayerSurfaceConfigure, LayerShellHandler, Layer},
};
use wayland_client::{Connection, QueueHandle, protocol::wl_surface};

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
    pub layer_shell: LayerShell,
    pub output_state: OutputState,
}

impl WaylandShell {
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
        let layer_shell = match LayerShell::bind(&globals, &qh) {
            Ok(s) => s,
            Err(e) => {
                tracing::debug!("LayerShell not supported on this compositor, entering headless mode: {:?}", e);
                return None;
            }
        };
        let output_state = OutputState::new(&globals, &qh);

        let state = WaylandState {
            registry_state,
            compositor_state,
            shm,
            layer_shell,
            output_state,
        };

        Some(Self {
            connection: conn,
            queue_handle: qh,
            event_queue,
            state,
        })
    }

    pub fn create_layer_surface(&mut self, _title: &str, width: u32, height: u32) {
         let surface = self.state.compositor_state.create_surface(&self.queue_handle);
         let layer_surface = self.state.layer_shell.create_layer_surface(
            &self.queue_handle,
            surface.clone(),
            Layer::Top,
            Some("tos-native"),
            None
         );

         layer_surface.set_size(width, height);
         surface.commit();
         tracing::info!("Wayland: Real Layer Surface created ({}x{})", width, height);
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
    fn runtime_add_global(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _name: u32, _interface: &str, _version: u32) {}
    fn runtime_remove_global(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _name: u32, _interface: &str) {}
}

impl RegistryHandler<WaylandState> for WaylandState {
    fn new_global(_state: &mut WaylandState, _conn: &Connection, _qh: &QueueHandle<Self>, _name: u32, _interface: &str, _version: u32) {}
    fn remove_global(_state: &mut WaylandState, _conn: &Connection, _qh: &QueueHandle<Self>, _name: u32, _interface: &str) {}
}

impl CompositorHandler for WaylandState {
    fn scale_factor_changed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _new_factor: i32) {}
    fn transform_changed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _new_transform: wayland_client::protocol::wl_output::Transform) {}
    fn frame(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _time: u32) {}
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
    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wayland_client::protocol::wl_output::WlOutput) {}
    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wayland_client::protocol::wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wayland_client::protocol::wl_output::WlOutput) {}
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

    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer_surface: &LayerSurface) {}
}

delegate_registry!(WaylandState);
delegate_compositor!(WaylandState);
delegate_shm!(WaylandState);
delegate_layer!(WaylandState);
delegate_output!(WaylandState);
