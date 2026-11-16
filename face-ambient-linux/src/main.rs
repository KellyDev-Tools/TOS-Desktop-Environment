//! TOS ambient face — the Phase 1 layer-shell host.
//!
//! One process, three layer-shell surfaces, one shared Svelte build:
//!
//! - **background** — the persistent terminal, under everything
//! - **chips** — a glanceable strip on the top edge, never focused
//! - **prompt** — CommandHub on an overlay surface, hidden until summoned
//!
//! Summoning is deliberately DE-agnostic: bind any hotkey in GNOME/KDE/
//! Sway/etc. to `tos-face-ambient --toggle-prompt`, which talks to the
//! running face over `$XDG_RUNTIME_DIR/tos-face.sock`.
//!
//! The UI location comes from `--ui <url-or-path>` or `TOS_FACE_UI`,
//! falling back to the workspace's `face-svelte-ui/build` output.

mod surfaces;

use std::cell::RefCell;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use surfaces::{FaceCommand, Keyboard, LayerKind, Surface, SurfaceSpec};
use webkit6::prelude::*;
use webkit6::{NetworkSession, TLSErrorsPolicy};

const APP_ID: &str = "net.kellydev.tos.face.ambient";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Client modes first: they need no display and no GTK.
    if let Some(cmd) = client_command(&args) {
        std::process::exit(run_client(cmd));
    }

    let ui_base = resolve_ui_base(&args);
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app| build_ui(app, &ui_base));
    // Run with a fixed argv so GTK never sees --ui / TOS flags.
    app.run_with_args(&["tos-face-ambient"]);
}

fn client_command(args: &[String]) -> Option<FaceCommand> {
    args.iter().find_map(|a| match a.as_str() {
        "--toggle-prompt" => Some(FaceCommand::TogglePrompt),
        "--show-prompt" => Some(FaceCommand::ShowPrompt),
        "--hide-prompt" => Some(FaceCommand::HidePrompt),
        _ => None,
    })
}

fn run_client(cmd: FaceCommand) -> i32 {
    let path = surfaces::socket_path();
    match UnixStream::connect(&path) {
        Ok(mut stream) => {
            let _ = writeln!(stream, "{}", cmd.wire());
            0
        }
        Err(e) => {
            eprintln!(
                "tos-face-ambient: cannot reach the running face at {}: {e}",
                path.display()
            );
            1
        }
    }
}

/// Resolve the face UI base (URL or file path) from --ui / TOS_FACE_UI,
/// else the workspace build output, else a placeholder page.
fn resolve_ui_base(args: &[String]) -> String {
    let explicit = args
        .windows(2)
        .find(|w| w[0] == "--ui")
        .map(|w| w[1].clone())
        .or_else(|| std::env::var("TOS_FACE_UI").ok());
    if let Some(value) = explicit {
        return as_url(&value);
    }
    let mut candidates: Vec<PathBuf> = vec![PathBuf::from("face-svelte-ui/build/index.html")];
    if let Ok(exe) = std::env::current_exe() {
        // target/<profile>/tos-face-ambient -> workspace root
        if let Some(root) = exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
            candidates.push(root.join("face-svelte-ui/build/index.html"));
        }
    }
    for c in candidates {
        if c.exists() {
            if let Ok(abs) = c.canonicalize() {
                return format!("file://{}", abs.display());
            }
        }
    }
    "data:text/html,<h1>TOS face UI not found</h1><p>Build face-svelte-ui \
     (npm run build) or pass --ui &lt;url-or-path&gt;.</p>"
        .to_string()
}

fn as_url(value: &str) -> String {
    if value.contains("://") {
        value.to_string()
    } else if let Ok(abs) = PathBuf::from(value).canonicalize() {
        format!("file://{}", abs.display())
    } else {
        value.to_string()
    }
}

fn build_ui(app: &Application, ui_base: &str) {
    let (tx, rx) = mpsc::channel::<FaceCommand>();
    spawn_command_listener(tx);

    // The Brain serves its face socket (7001) over TLS with a
    // self-signed certificate it generates itself (rcgen) — there is no
    // CA-issued cert for a loopback service, so the system trust store
    // can never validate it and WebKit would refuse the connection.
    // This face only ever loads the local UI bundle and talks to that
    // loopback Brain, so TLS errors are ignored on the (default)
    // network session its web views use. Revisit in Roadmap Phase 2,
    // where remote Brains move to mesh-VPN transport instead of public
    // TLS with self-signed certs.
    if let Some(session) = NetworkSession::default() {
        session.set_tls_errors_policy(TLSErrorsPolicy::Ignore);
    }

    let mut prompt_window: Option<ApplicationWindow> = None;
    for surface in Surface::all() {
        let spec = surfaces::spec(surface);
        let window = make_surface_window(app, &spec, ui_base);
        if surface == Surface::Prompt {
            prompt_window = Some(window);
        }
    }
    let prompt_window = Rc::new(RefCell::new(prompt_window));

    // Drain face commands on the GTK main loop.
    let prompt = prompt_window.clone();
    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(120), move || {
        while let Ok(cmd) = rx.try_recv() {
            if let Some(win) = prompt.borrow().as_ref() {
                apply_command(win, cmd);
            }
        }
        gtk4::glib::ControlFlow::Continue
    });
}

fn apply_command(win: &ApplicationWindow, cmd: FaceCommand) {
    match cmd {
        FaceCommand::ShowPrompt => show_prompt(win),
        FaceCommand::HidePrompt => hide_prompt(win),
        FaceCommand::TogglePrompt => {
            if win.is_visible() {
                hide_prompt(win);
            } else {
                show_prompt(win);
            }
        }
    }
}

fn show_prompt(win: &ApplicationWindow) {
    win.set_keyboard_mode(KeyboardMode::Exclusive);
    win.present();
}

fn hide_prompt(win: &ApplicationWindow) {
    // Release the keyboard grab before disappearing so focus returns
    // to the host DE cleanly.
    win.set_keyboard_mode(KeyboardMode::None);
    win.set_visible(false);
}

fn spawn_command_listener(tx: mpsc::Sender<FaceCommand>) {
    std::thread::spawn(move || {
        let path = surfaces::socket_path();
        let _ = std::fs::remove_file(&path);
        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[tos-face] cannot bind {}: {e}", path.display());
                return;
            }
        };
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).is_ok() {
                        if let Some(cmd) = FaceCommand::parse(&line) {
                            let _ = tx.send(cmd);
                        }
                    }
                }
                Err(_) => continue,
            }
        }
    });
}

fn make_surface_window(app: &Application, spec: &SurfaceSpec, ui_base: &str) -> ApplicationWindow {
    let window = ApplicationWindow::builder().application(app).build();

    window.init_layer_shell();
    window.set_namespace(Some(spec.surface.namespace()));
    window.set_layer(match spec.layer {
        LayerKind::Background => Layer::Background,
        LayerKind::Top => Layer::Top,
        LayerKind::Overlay => Layer::Overlay,
    });
    for (edge, on) in [
        (Edge::Top, spec.anchor_top),
        (Edge::Bottom, spec.anchor_bottom),
        (Edge::Left, spec.anchor_left),
        (Edge::Right, spec.anchor_right),
    ] {
        window.set_anchor(edge, on);
    }
    window.set_exclusive_zone(spec.exclusive_zone);
    window.set_keyboard_mode(match spec.keyboard {
        Keyboard::Exclusive => KeyboardMode::Exclusive,
        Keyboard::None => KeyboardMode::None,
    });
    if spec.default_width > 0 || spec.default_height > 0 {
        window.set_default_size(spec.default_width, spec.default_height);
    }

    let view = webkit6::WebView::new();
    if spec.transparent {
        view.set_background_color(&gtk4::gdk::RGBA::new(0.0, 0.0, 0.0, 0.0));
    }
    view.load_uri(&surfaces::ui_url(ui_base, spec.surface));
    window.set_child(Some(&view));

    if spec.start_visible {
        window.present();
    }
    window
}
