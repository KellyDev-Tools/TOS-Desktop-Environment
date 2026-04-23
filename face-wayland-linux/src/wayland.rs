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
use wayland_protocols::wp::linux_dmabuf::zv1::client::{zwp_linux_dmabuf_v1, zwp_linux_buffer_params_v1};
use wayland_client::{protocol::{wl_surface, wl_shm_pool, wl_buffer}, Connection, QueueHandle};

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
    pub dmabuf: Option<zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1>,
    pub output_state: OutputState,
}

impl WaylandShell {
    pub fn can_connect() -> bool {
        match std::env::var("WAYLAND_DISPLAY") {
            Ok(_) => {
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
        let dmabuf = globals.bind::<zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1, _, _>(&qh, 1..=4, ()).ok();

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
            dmabuf,
            output_state,
        };

        Some(Self {
            connection: conn,
            queue_handle: qh,
            event_queue,
            state,
        })
    }

    pub fn create_layer_surface(&mut self, title: &str, width: u32, height: u32) -> wl_surface::WlSurface {
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
            window.set_app_id("org.tos.native-shell".to_string());
            surface.commit();
            tracing::info!("Wayland: Fallback XDG Window created ({}x{})", width, height);
        }
        surface
    }

    pub fn attach_buffer(&mut self, surface: &wl_surface::WlSurface, fd: std::os::unix::io::RawFd, width: i32, height: i32) {
        use wayland_client::protocol::wl_shm;
        use std::os::unix::io::BorrowedFd;
        
        let size = width * height * 4;
        
        // Prefer DMABUF for Stage 5.2 performance requirements
        let buffer = if let Some(dmabuf_buffer) = self.create_dmabuf_buffer(fd, width, height) {
            tracing::debug!("Wayland: Using DMABUF for surface attachment");
            dmabuf_buffer
        } else {
            // Fallback to SHM (Stage 5.1 path)
            tracing::debug!("Wayland: DMABUF unavailable, falling back to SHM");
            let borrowed_fd = unsafe { BorrowedFd::borrow_raw(fd) };
            let pool = self.state.shm.wl_shm().create_pool(borrowed_fd, size, &self.queue_handle, ());
            pool.create_buffer(0, width, height, width * 4, wl_shm::Format::Argb8888, &self.queue_handle, ())
        };

        // Attach, damage, and commit
        surface.attach(Some(&buffer), 0, 0);
        surface.damage(0, 0, width, height);
        surface.commit();
        
        tracing::debug!("Wayland: Buffer attached and committed ({}x{})", width, height);
    }

    pub fn create_dmabuf_buffer(&mut self, fd: std::os::unix::io::RawFd, width: i32, height: i32) -> Option<wl_buffer::WlBuffer> {
        use std::os::unix::io::BorrowedFd;
        
        let dmabuf = self.state.dmabuf.as_ref()?;
        let params = dmabuf.create_params(&self.queue_handle, ());
        
        // DRM_FORMAT_ARGB8888 = 0x34325241
        let format = 0x34325241;
        
        // SAFETY: FD is managed by the caller (LinuxRenderer)
        let borrowed_fd = unsafe { BorrowedFd::borrow_raw(fd) };

        params.add(
            borrowed_fd,
            0, // plane 0
            0, // offset
            (width * 4) as u32, // stride
            0, // modifier_hi
            0, // modifier_lo
        );
        
        let buffer = params.create_immed(
            width,
            height,
            format,
            zwp_linux_buffer_params_v1::Flags::empty(),
            &self.queue_handle,
            (),
        );
        
        Some(buffer)
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
    fn configure(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer_surface: &LayerSurface, _configure: LayerSurfaceConfigure, _serial: u32) {}
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer_surface: &LayerSurface) {}
}

impl wayland_client::Dispatch<wl_shm_pool::WlShmPool, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl wayland_client::Dispatch<wl_buffer::WlBuffer, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_buffer::WlBuffer,
        _event: wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // In a production renderer, we'd handle the Release event to recycle buffers.
    }
}

impl wayland_client::Dispatch<zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1,
        _event: zwp_linux_dmabuf_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // We'd receive modifiers and formats here
    }
}

impl wayland_client::Dispatch<zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1,
        _event: zwp_linux_buffer_params_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // We'd receive created/failed events here
    }
}

impl XdgWindowHandler for WaylandState {
    fn configure(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _window: &XdgWindow, _configure: XdgWindowConfigure, _serial: u32) {}
    fn request_close(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _window: &XdgWindow) {}
}

delegate_registry!(WaylandState);
delegate_compositor!(WaylandState);
delegate_shm!(WaylandState);
delegate_xdg_shell!(WaylandState);
delegate_xdg_window!(WaylandState);
delegate_layer!(WaylandState);
delegate_output!(WaylandState);
