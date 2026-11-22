//! Surface specifications for the ambient face.
//!
//! Pure data and helpers — no GTK — so the slice's surface contract
//! (which layer, which anchors, who may take the keyboard) is unit
//! tested without a display. `main.rs` wires these specs to real
//! layer-shell windows.

use std::path::PathBuf;

/// The three surfaces of the Phase 1 slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    /// The persistent terminal session, dimmed at rest, under everything.
    Background,
    /// The glanceable status strip. Never takes focus.
    Chips,
    /// CommandHub, summoned on demand, takes focus while in use.
    Prompt,
}

impl Surface {
    pub fn name(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Chips => "chips",
            Self::Prompt => "prompt",
        }
    }

    /// layer-shell namespace, so compositors/users can target each
    /// surface with rules (blur, animations) independently.
    pub fn namespace(self) -> &'static str {
        match self {
            Self::Background => "tos-background",
            Self::Chips => "tos-chips",
            Self::Prompt => "tos-prompt",
        }
    }

    pub fn all() -> [Surface; 3] {
        [Self::Background, Self::Chips, Self::Prompt]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerKind {
    Background,
    Top,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyboard {
    /// Never receives keyboard input.
    None,
    /// Grabs the keyboard while visible (the prompt, while summoned).
    Exclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceSpec {
    pub surface: Surface,
    pub layer: LayerKind,
    pub anchor_top: bool,
    pub anchor_bottom: bool,
    pub anchor_left: bool,
    pub anchor_right: bool,
    /// -1 = cover the whole output (background); 0 = reserve nothing.
    pub exclusive_zone: i32,
    pub keyboard: Keyboard,
    /// Transparent window background; the web UI paints what it wants
    /// (the prompt's centered card, the chips strip) and the rest of
    /// the desktop shows through.
    pub transparent: bool,
    pub start_visible: bool,
    /// 0 = sized by anchors / content.
    pub default_width: i32,
    pub default_height: i32,
}

pub fn spec(surface: Surface) -> SurfaceSpec {
    match surface {
        Surface::Background => SurfaceSpec {
            surface,
            layer: LayerKind::Background,
            anchor_top: true,
            anchor_bottom: true,
            anchor_left: true,
            anchor_right: true,
            exclusive_zone: -1,
            keyboard: Keyboard::None,
            transparent: false,
            start_visible: true,
            default_width: 0,
            default_height: 0,
        },
        Surface::Chips => SurfaceSpec {
            surface,
            layer: LayerKind::Top,
            anchor_top: true,
            anchor_bottom: false,
            anchor_left: true,
            anchor_right: true,
            exclusive_zone: 0,
            keyboard: Keyboard::None,
            transparent: true,
            start_visible: true,
            default_width: 0,
            default_height: 64,
        },
        Surface::Prompt => SurfaceSpec {
            surface,
            layer: LayerKind::Overlay,
            // Full-output surface with a transparent background; the web
            // UI centers the prompt card and paints the dim/zoom backdrop.
            // Motion (zoom/slide) is a web-side transition, per the spec.
            anchor_top: true,
            anchor_bottom: true,
            anchor_left: true,
            anchor_right: true,
            exclusive_zone: 0,
            keyboard: Keyboard::Exclusive,
            transparent: true,
            start_visible: false,
            default_width: 0,
            default_height: 0,
        },
    }
}

/// The URL a surface's web view loads: the shared face build, routed by
/// the `surface` query parameter (see `+page.svelte`), optionally
/// appending a `brain=<url>` query parameter so the web UI connects
/// directly to a configured Brain WebSocket endpoint.
pub fn ui_url_with_brain(base: &str, surface: Surface, brain_url: Option<&str>) -> String {
    let sep = if base.contains('?') { '&' } else { '?' };
    let mut url = format!("{base}{sep}surface={}", surface.name());
    if let Some(brain) = brain_url.map(str::trim).filter(|s| !s.is_empty()) {
        url.push_str("&brain=");
        url.push_str(&encode_query_component(brain));
    }
    url
}

fn encode_query_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}

/// Where the face listens for prompt commands. One face per user
/// session, so a per-user runtime path is the right scope.
pub fn socket_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir).join("tos-face.sock");
        }
    }
    PathBuf::from("/tmp/tos-face.sock")
}

/// Commands a hotkey (or any script) can send the running face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceCommand {
    TogglePrompt,
    ShowPrompt,
    HidePrompt,
}

impl FaceCommand {
    pub fn wire(self) -> &'static str {
        match self {
            Self::TogglePrompt => "toggle-prompt",
            Self::ShowPrompt => "show-prompt",
            Self::HidePrompt => "hide-prompt",
        }
    }

    pub fn parse(line: &str) -> Option<Self> {
        match line.trim() {
            "toggle" | "toggle-prompt" => Some(Self::TogglePrompt),
            "show" | "show-prompt" => Some(Self::ShowPrompt),
            "hide" | "hide-prompt" => Some(Self::HidePrompt),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_prompt_takes_the_keyboard() {
        for s in Surface::all() {
            let want = if s == Surface::Prompt {
                Keyboard::Exclusive
            } else {
                Keyboard::None
            };
            assert_eq!(spec(s).keyboard, want, "{s:?}");
        }
    }

    #[test]
    fn only_the_prompt_starts_hidden() {
        for s in Surface::all() {
            assert_eq!(spec(s).start_visible, s != Surface::Prompt, "{s:?}");
        }
    }

    #[test]
    fn background_covers_the_output_and_sits_under_everything() {
        let b = spec(Surface::Background);
        assert_eq!(b.layer, LayerKind::Background);
        assert!(b.anchor_top && b.anchor_bottom && b.anchor_left && b.anchor_right);
        assert_eq!(b.exclusive_zone, -1);
    }

    #[test]
    fn chips_are_a_top_edge_strip_that_reserves_nothing() {
        let c = spec(Surface::Chips);
        assert_eq!(c.layer, LayerKind::Top);
        assert!(c.anchor_top && c.anchor_left && c.anchor_right && !c.anchor_bottom);
        assert_eq!(c.exclusive_zone, 0);
    }

    #[test]
    fn namespaces_are_distinct() {
        let mut ns: Vec<&str> = Surface::all().iter().map(|s| s.namespace()).collect();
        ns.sort();
        ns.dedup();
        assert_eq!(ns.len(), 3);
    }

    #[test]
    fn ui_url_appends_the_surface_route() {
        assert_eq!(
            ui_url_with_brain("file:///ui/index.html", Surface::Prompt, None),
            "file:///ui/index.html?surface=prompt"
        );
        assert_eq!(
            ui_url_with_brain("http://localhost:5173/?dev=1", Surface::Chips, None),
            "http://localhost:5173/?dev=1&surface=chips"
        );
    }

    #[test]
    fn ui_url_with_brain_encodes_remote_endpoint() {
        assert_eq!(
            ui_url_with_brain(
                "http://127.0.0.1:18701/",
                Surface::Background,
                Some("wss://100.64.0.5:7001")
            ),
            "http://127.0.0.1:18701/?surface=background&brain=wss%3A%2F%2F100.64.0.5%3A7001"
        );
        assert_eq!(
            ui_url_with_brain("http://127.0.0.1:18701/", Surface::Chips, Some("   ")),
            "http://127.0.0.1:18701/?surface=chips"
        );
    }

    #[test]
    fn chips_height_accommodates_touch_targets() {
        assert!(spec(Surface::Chips).default_height >= 64);
    }

    #[test]
    fn commands_round_trip_and_reject_garbage() {
        for c in [
            FaceCommand::TogglePrompt,
            FaceCommand::ShowPrompt,
            FaceCommand::HidePrompt,
        ] {
            assert_eq!(FaceCommand::parse(c.wire()), Some(c));
        }
        assert_eq!(FaceCommand::parse("toggle\n"), Some(FaceCommand::TogglePrompt));
        assert_eq!(FaceCommand::parse("self-destruct"), None);
        assert_eq!(FaceCommand::parse(""), None);
    }

    #[test]
    fn socket_path_lives_in_a_user_runtime_dir() {
        let p = socket_path();
        assert!(p.ends_with("tos-face.sock"));
    }
}
