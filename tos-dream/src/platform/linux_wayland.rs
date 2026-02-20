use smithay::{
    delegate_compositor, delegate_output, delegate_seat, delegate_shm, delegate_xdg_shell,
    delegate_xdg_decoration,
    reexports::{
        wayland_server::{Display, DisplayHandle, protocol::{wl_surface::WlSurface, wl_buffer, wl_seat}, Client, ListeningSocket},
        wayland_protocols::{
            xdg::shell::server::xdg_toplevel,
            xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1,
        },
    },
    wayland::{
        compositor::{CompositorHandler, CompositorState, CompositorClientState},
        output::{OutputHandler, OutputManagerState},
        shm::{ShmHandler, ShmState},
        shell::xdg::{
            XdgShellHandler, XdgShellState, ToplevelSurface, PopupSurface, PositionerState,
            decoration::XdgDecorationState,
        },
    },
    input::{
        Seat, SeatHandler, SeatState, pointer::{CursorImageStatus, MotionEvent, ButtonEvent},
        keyboard::XkbConfig,
    },
    utils::{Logical, Point, Rectangle, Serial, Transform, Physical},
    desktop::{Space, Window, space::SpaceRenderElements},
    backend::{
        renderer::{
            gles::GlesRenderer,
            damage::OutputDamageTracker,
            element::{
                Id, Kind,
                solid::SolidColorRenderElement,
                AsRenderElements,
            },
            utils::CommitCounter,
            ImportAll,
        },
        winit::WinitEvent,
        input::{InputEvent, AbsolutePositionEvent, Event, PointerButtonEvent},
    },
    output::{Output, PhysicalProperties, Subpixel, Mode, Scale},
};
use smithay::backend::input::PointerMotionEvent;

use std::sync::{Arc, Mutex};
use super::{Renderer as TosPlatformRenderer, InputSource, SurfaceConfig, SurfaceHandle};
use crate::system::input::SemanticEvent;
use crate::TosState;

// LCARS Palette (§1.1, §2.1)
const LCARS_GOLD: [f32; 4] = [1.0, 0.6, 0.0, 1.0];
const LCARS_PEACH: [f32; 4] = [1.0, 0.8, 0.6, 1.0];
const LCARS_BLUE: [f32; 4] = [0.4, 0.4, 1.0, 1.0];
const LCARS_BLACK: [f32; 4] = [0.01, 0.01, 0.04, 1.0];

pub struct TosCompositorState {
    pub display_handle: DisplayHandle,
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub output_state: OutputManagerState,
    pub seat_state: SeatState<TosCompositorState>,
    pub data_device_state: smithay::wayland::selection::data_device::DataDeviceState,
    pub primary_selection_state: smithay::wayland::selection::primary_selection::PrimarySelectionState,
    pub decoration_state: XdgDecorationState,
    pub seat: Seat<TosCompositorState>,
    pub space: Space<Window>,
    pub running: bool,
    pub tos_state: Arc<Mutex<TosState>>,
    pub pending_semantic_events: Arc<Mutex<Vec<SemanticEvent>>>,
    pub pointer_location: Point<f64, Logical>,
    pub bezel_ids: [Id; 6],
}

#[derive(Default)]
pub struct WaylandPointerData {
    pub compositor_state: CompositorClientState,
}

impl CompositorHandler for TosCompositorState {
    fn compositor_state(&mut self) -> &mut CompositorState { &mut self.compositor_state }
    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<WaylandPointerData>().unwrap().compositor_state
    }
    fn commit(&mut self, surface: &WlSurface) {
        smithay::backend::renderer::utils::on_commit_buffer_handler::<Self>(surface);
        for window in self.space.elements() {
            if window.toplevel().map(|t| t.wl_surface() == surface).unwrap_or(false) {
                window.on_commit();
            }
        }
    }
}

impl smithay::wayland::selection::SelectionHandler for TosCompositorState {
    type SelectionUserData = ();
}

impl smithay::wayland::selection::data_device::DataDeviceHandler for TosCompositorState {
    fn data_device_state(&self) -> &smithay::wayland::selection::data_device::DataDeviceState { &self.data_device_state }
}

impl smithay::wayland::selection::primary_selection::PrimarySelectionHandler for TosCompositorState {
    fn primary_selection_state(&self) -> &smithay::wayland::selection::primary_selection::PrimarySelectionState { &self.primary_selection_state }
}

impl smithay::wayland::selection::data_device::ClientDndGrabHandler for TosCompositorState {}
impl smithay::wayland::selection::data_device::ServerDndGrabHandler for TosCompositorState {}

impl smithay::wayland::buffer::BufferHandler for TosCompositorState {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl XdgShellHandler for TosCompositorState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState { &mut self.xdg_shell_state }
    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface);
        
        let mut width = 1200;
        let mut height = 728;
        if let Some(output) = self.space.outputs().next() {
            if let Some(mode) = output.current_mode() {
                width = mode.size.w - 80;
                height = mode.size.h - 68;
            }
        }
        
        if let Some(toplevel) = window.toplevel() {
            toplevel.with_pending_state(|state| {
                state.size = Some((width, height).into());
                state.states.set(xdg_toplevel::State::Activated);
                state.states.set(xdg_toplevel::State::Maximized);
            });
            toplevel.send_configure();
        }
        self.space.map_element(window, (76, 36), true);
    }
    fn new_popup(&mut self, _surface: PopupSurface, _state: PositionerState) {}
    fn reposition_request(&mut self, _surface: PopupSurface, _state: PositionerState, _token: u32) {}
    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}
}

impl ShmHandler for TosCompositorState {
    fn shm_state(&self) -> &ShmState { &self.shm_state }
}

impl SeatHandler for TosCompositorState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;
    fn seat_state(&mut self) -> &mut SeatState<Self> { &mut self.seat_state }
    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&Self::KeyboardFocus>) {}
    fn cursor_image(&mut self, _seat: &Seat<Self>, _cursor: CursorImageStatus) {}
}

impl OutputHandler for TosCompositorState {}
delegate_compositor!(TosCompositorState);
delegate_xdg_shell!(TosCompositorState);
delegate_shm!(TosCompositorState);
delegate_seat!(TosCompositorState);
delegate_output!(TosCompositorState);
delegate_xdg_decoration!(TosCompositorState);
smithay::delegate_data_device!(TosCompositorState);
smithay::delegate_primary_selection!(TosCompositorState);

impl smithay::wayland::shell::xdg::decoration::XdgDecorationHandler for TosCompositorState {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(zxdg_toplevel_decoration_v1::Mode::ServerSide);
        });
        toplevel.send_configure();
    }
    fn request_mode(&mut self, toplevel: ToplevelSurface, _mode: zxdg_toplevel_decoration_v1::Mode) {
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(zxdg_toplevel_decoration_v1::Mode::ServerSide);
        });
        toplevel.send_configure();
    }
    fn unset_mode(&mut self, _toplevel: ToplevelSurface) {}
}

smithay::backend::renderer::element::render_elements! {
    pub TosRenderElements<'a, R, E, C> where
        R: ImportAll;
    Space=E,
    Custom=&'a C,
}

pub struct WaylandRenderer {
    pub display: Display<TosCompositorState>,
    pub state: Option<Arc<Mutex<TosCompositorState>>>,
}

impl WaylandRenderer {
    pub fn new() -> Self {
        let display = Display::<TosCompositorState>::new().unwrap();
        Self { display, state: None }
    }

    pub fn run_event_loop(&mut self, tos_state: Arc<Mutex<TosState>>) {
        let dh = self.display.handle();
        let compositor_state = CompositorState::new::<TosCompositorState>(&dh);
        let xdg_shell_state = XdgShellState::new::<TosCompositorState>(&dh);
        let shm_state = ShmState::new::<TosCompositorState>(&dh, Vec::new());
        let output_state = OutputManagerState::new_with_xdg_output::<TosCompositorState>(&dh);
        let decoration_state = XdgDecorationState::new::<TosCompositorState>(&dh);
        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&dh, "seat0"); 
        seat.add_pointer();
        seat.add_keyboard(XkbConfig::default(), 200, 25).unwrap();
        
        let data_device_state = smithay::wayland::selection::data_device::DataDeviceState::new::<TosCompositorState>(&dh);
        let primary_selection_state = smithay::wayland::selection::primary_selection::PrimarySelectionState::new::<TosCompositorState>(&dh);

        let state = Arc::new(Mutex::new(TosCompositorState {
            display_handle: dh.clone(),
            compositor_state, xdg_shell_state, shm_state, output_state, seat_state, decoration_state,
            data_device_state, primary_selection_state,
            seat, space: Space::default(), running: true, tos_state,
            pending_semantic_events: Arc::new(Mutex::new(Vec::new())),
            pointer_location: (0.0, 0.0).into(),
            bezel_ids: [Id::new(), Id::new(), Id::new(), Id::new(), Id::new(), Id::new()],
        }));
        self.state = Some(state.clone());

        println!("Wayland compositor initialized. Starting socket...");
        let socket = ListeningSocket::bind_auto("wayland", 1..32).unwrap();
        let socket_name = socket.socket_name().unwrap().to_string_lossy().into_owned();
        println!("Listening on socket: {}", socket_name);

        println!("Initializing Winit backend...");
        let (mut backend, mut winit_loop) = smithay::backend::winit::init::<GlesRenderer>().unwrap();
        println!("Winit backend initialized on host display.");
        let output = Output::new("tos-display".into(), PhysicalProperties { size: (0,0).into(), subpixel: Subpixel::Unknown, make: "KD".into(), model: "TOS".into() });
        let _global = output.create_global::<TosCompositorState>(&dh);
        
        output.change_current_state(Some(Mode { size: backend.window_size(), refresh: 60_000 }), Some(Transform::Flipped180), Some(Scale::Fractional(1.0)), Some((0,0).into()));
        state.lock().unwrap().space.map_output(&output, (0,0));
        let mut tracker = OutputDamageTracker::from_output(&output);

        // Client spawning thread
        let socket_name_clone = socket_name.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(2000));
            println!("Spawning internal client...");
            let mut cmd = std::process::Command::new(std::env::current_exe().unwrap());
            cmd.env("WAYLAND_DISPLAY", &socket_name_clone);
            cmd.spawn().ok();
        });

        // Socket acceptance thread
        let state_for_socket = state.clone();
        std::thread::spawn(move || {
            while let Ok(stream) = socket.accept() {
                if let Some(stream) = stream {
                    let mut sl = state_for_socket.lock().unwrap();
                    println!("Accepted new client connection.");
                    sl.display_handle.insert_client(stream, Arc::new(WaylandPointerData::default())).ok();
                }
            }
        });

        let mut prev_fb_size = backend.window_size();
        let mut serial = 0u32;
        while state.lock().unwrap().running {
            {
                let mut sl = state.lock().unwrap();
                self.display.dispatch_clients(&mut *sl).ok();
            }

            let mut running = true;
            winit_loop.dispatch_new_events(|event| {
                match event {
                    WinitEvent::CloseRequested => { running = false; }
                    WinitEvent::Input(input) => {
                        let mut sl = state.lock().unwrap();
                        match input {
                            InputEvent::PointerMotionAbsolute { event, .. } => {
                                let size = backend.window_size();
                                let x = event.x_transformed(size.w);
                                let y = event.y_transformed(size.h);
                                let phys_pos: Point<f64, Physical> = Point::from((x, y));
                                let pos = phys_pos.to_logical(1.0);
                                sl.pointer_location = pos;
                                serial += 1;
                                
                                let focus = sl.space.element_under(pos).map(|(window, origin)| {
                                    (window.toplevel().unwrap().wl_surface().clone(), origin.to_f64())
                                });
                                
                                sl.seat.get_pointer().unwrap().motion(&mut *sl, focus, &MotionEvent {
                                    location: pos,
                                    serial: Serial::from(serial),
                                    time: event.time() as u32,
                                });
                            }
                            InputEvent::PointerMotion { event, .. } => {
                                let mut pos = sl.pointer_location;
                                pos.x += <smithay::backend::input::UnusedEvent as PointerMotionEvent<smithay::backend::winit::WinitInput>>::delta_x(&event);
                                pos.y += <smithay::backend::input::UnusedEvent as PointerMotionEvent<smithay::backend::winit::WinitInput>>::delta_y(&event);
                                
                                let size = backend.window_size();
                                pos.x = pos.x.clamp(0.0, size.w as f64);
                                pos.y = pos.y.clamp(0.0, size.h as f64);
                                
                                sl.pointer_location = pos;
                                serial += 1;
                                
                                let focus = sl.space.element_under(pos).map(|(window, origin)| {
                                    (window.toplevel().unwrap().wl_surface().clone(), origin.to_f64())
                                });
                                
                                sl.seat.get_pointer().unwrap().motion(&mut *sl, focus, &MotionEvent {
                                    location: pos,
                                    serial: Serial::from(serial),
                                    time: <smithay::backend::input::UnusedEvent as smithay::backend::input::Event<smithay::backend::winit::WinitInput>>::time(&event) as u32,
                                });
                            }
                            InputEvent::PointerButton { event, .. } => {
                                serial += 1;
                                sl.seat.get_pointer().unwrap().button(&mut *sl, &ButtonEvent {
                                    button: event.button_code(),
                                    state: event.state(),
                                    serial: Serial::from(serial),
                                    time: event.time() as u32,
                                });
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            });
            if !running { state.lock().unwrap().running = false; }

            let (age, fb_size, bezel_ids) = {
                let sl = state.lock().unwrap();
                (backend.buffer_age().unwrap_or(0), backend.window_size(), sl.bezel_ids.clone())
            };

            if prev_fb_size != fb_size {
                prev_fb_size = fb_size;
                output.change_current_state(
                    Some(Mode { size: fb_size, refresh: 60_000 }),
                    None, None, None
                );
                
                let sl = state.lock().unwrap();
                let width = if fb_size.w > 80 { fb_size.w - 80 } else { fb_size.w };
                let height = if fb_size.h > 68 { fb_size.h - 68 } else { fb_size.h };
                for window in sl.space.elements() {
                    if let Some(toplevel) = window.toplevel() {
                        toplevel.with_pending_state(|state| {
                            state.size = Some((width, height).into());
                        });
                        toplevel.send_configure();
                    }
                }
            }

            let (backend_renderer, mut framebuffer) = backend.bind().unwrap();
            
            let scale = 1.0;
            let (sw, sh) = (fb_size.w, fb_size.h);
            let mut bezel = Vec::new();
            bezel.push(SolidColorRenderElement::new(bezel_ids[0].clone(), Rectangle::new((72, 4).into(), (sw-76, 28).into()).to_f64().to_physical(scale).to_i32_round(), CommitCounter::default(), LCARS_GOLD, Kind::Unspecified));
            bezel.push(SolidColorRenderElement::new(bezel_ids[1].clone(), Rectangle::new((4, 4).into(), (64, 28).into()).to_f64().to_physical(scale).to_i32_round(), CommitCounter::default(), LCARS_BLUE, Kind::Unspecified));
            bezel.push(SolidColorRenderElement::new(bezel_ids[2].clone(), Rectangle::new((4, 36).into(), (48, sh-72).into()).to_f64().to_physical(scale).to_i32_round(), CommitCounter::default(), LCARS_BLUE, Kind::Unspecified));
            bezel.push(SolidColorRenderElement::new(bezel_ids[3].clone(), Rectangle::new((72, sh-32).into(), (sw-76, 28).into()).to_f64().to_physical(scale).to_i32_round(), CommitCounter::default(), LCARS_PEACH, Kind::Unspecified));
            bezel.push(SolidColorRenderElement::new(bezel_ids[4].clone(), Rectangle::new((4, sh-32).into(), (64, 28).into()).to_f64().to_physical(scale).to_i32_round(), CommitCounter::default(), LCARS_BLUE, Kind::Unspecified));

            {
                let pos = state.lock().unwrap().pointer_location;
                let cx = pos.x as i32;
                let cy = pos.y as i32;
                bezel.push(SolidColorRenderElement::new(bezel_ids[5].clone(), Rectangle::new((cx, cy).into(), (12, 12).into()).to_f64().to_physical(scale).to_i32_round(), CommitCounter::default(), [1.0, 1.0, 1.0, 1.0], Kind::Unspecified));
            }

            let mut elements: Vec<TosRenderElements<GlesRenderer, SpaceRenderElements<GlesRenderer, <Window as AsRenderElements<GlesRenderer>>::RenderElement>, SolidColorRenderElement>> = {
                let sl = state.lock().unwrap();
                smithay::desktop::space::space_render_elements::<GlesRenderer, Window, _>(backend_renderer, [&sl.space], &output, 1.0)
                    .unwrap_or_default()
                    .into_iter()
                    .map(TosRenderElements::Space)
                    .collect()
            };
            elements.extend(bezel.iter().map(TosRenderElements::Custom));

            let damage = {
                let render_result = tracker.render_output(backend_renderer, &mut framebuffer, age, &elements, LCARS_BLACK).unwrap();
                render_result.damage.cloned()
            };
            drop(framebuffer);
            backend.submit(damage.as_deref()).ok();

            {
                let mut sl = state.lock().unwrap();
                sl.display_handle.flush_clients().ok();
                sl.space.refresh();

                let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
                sl.space.elements().for_each(|w| w.send_frame(&output, time, None, |_, _| Some(output.clone())));
            }
            
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
}

pub struct WaylandInputSource {
    pub event_queue: Arc<Mutex<Vec<SemanticEvent>>>,
}

impl InputSource for WaylandInputSource {
    fn poll_events(&mut self) -> Vec<SemanticEvent> {
        let mut queue = self.event_queue.lock().unwrap();
        let events = queue.clone();
        queue.clear();
        events
    }
}

impl TosPlatformRenderer for WaylandRenderer {
    fn create_surface(&mut self, _config: SurfaceConfig) -> SurfaceHandle { SurfaceHandle(0) }
    fn update_surface(&mut self, _handle: SurfaceHandle) {}
    fn composite(&mut self) {}
    fn set_bezel_visible(&mut self, _handle: SurfaceHandle, _visible: bool) {}
}

impl smithay::reexports::wayland_server::backend::ClientData for WaylandPointerData {}
