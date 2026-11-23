//! Brain-side encoding of raw key events into PTY bytes.
//!
//! Faces capture keys and send them uninterpreted (Architecture
//! §26.3: no pre-Brain parsing). What a key sends depends on
//! emulation state — cursor-key mode changes the arrow keys,
//! bracketed-paste mode changes text delivery — and that state
//! lives in the Brain's grid, so the encoding happens here,
//! beside it. The event vocabulary is the capturing platform's
//! own terms (the web platform's `KeyboardEvent.key` / `.code`
//! for the current faces); the encoder is the one place that
//! vocabulary is understood.

use alacritty_terminal::term::TermMode;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct KeyEvent {
    /// The key's identity, in the capturer's terms (e.g. "a",
    /// "Enter", "ArrowUp", "F5", ";"). A one-character value is
    /// the character itself.
    pub key: String,
    /// The physical-key code, when the capturer provides one
    /// (used for the function row, whose `key` varies by layout).
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub shift: bool,
}

/// xterm modifier parameter: 1 + shift(1) + alt(2) + ctrl(4).
fn modifier_param(ev: &KeyEvent) -> u8 {
    1 + (ev.shift as u8) + (ev.alt as u8) * 2 + (ev.ctrl as u8) * 4
}

fn with_alt(mut bytes: Vec<u8>, alt: bool) -> Vec<u8> {
    if alt {
        let mut out = vec![0x1b];
        out.append(&mut bytes);
        out
    } else {
        bytes
    }
}

/// Encode one key event to the bytes the PTY should receive, or
/// None when the event carries no terminal input (modifier-only
/// presses, unidentified keys — text from those arrives through
/// the text path instead).
pub fn encode_key(ev: &KeyEvent, mode: &TermMode) -> Option<Vec<u8>> {
    let key = ev.key.as_str();
    let m = modifier_param(ev);

    // Named keys first.
    match key {
        "Enter" => return Some(with_alt(vec![b'\r'], ev.alt)),
        "Backspace" => return Some(vec![if ev.ctrl { 0x08 } else { 0x7f }]),
        "Tab" => {
            if ev.shift {
                return Some(b"\x1b[Z".to_vec());
            }
            return Some(with_alt(vec![b'\t'], ev.alt));
        }
        "Escape" | "Esc" => return Some(vec![0x1b]),
        "ArrowUp" | "ArrowDown" | "ArrowRight" | "ArrowLeft" => {
            let fin = match key {
                "ArrowUp" => b'A',
                "ArrowDown" => b'B',
                "ArrowRight" => b'C',
                _ => b'D',
            };
            if m == 1 {
                if mode.contains(TermMode::APP_CURSOR) {
                    return Some(vec![0x1b, b'O', fin]);
                }
                return Some(vec![0x1b, b'[', fin]);
            }
            return Some(format!("\x1b[1;{}{}", m, fin as char).into_bytes());
        }
        "Home" | "End" => {
            let fin = if key == "Home" { b'H' } else { b'F' };
            if m == 1 {
                if mode.contains(TermMode::APP_CURSOR) {
                    return Some(vec![0x1b, b'O', fin]);
                }
                return Some(vec![0x1b, b'[', fin]);
            }
            return Some(format!("\x1b[1;{}{}", m, fin as char).into_bytes());
        }
        "PageUp" | "PageDown" | "Insert" | "Delete" => {
            let n = match key {
                "Insert" => 2,
                "Delete" => 3,
                "PageUp" => 5,
                _ => 6,
            };
            if m == 1 {
                return Some(format!("\x1b[{}~", n).into_bytes());
            }
            return Some(format!("\x1b[{};{}~", n, m).into_bytes());
        }
        "Control" | "Shift" | "Alt" | "Meta" | "CapsLock" | "NumLock" | "ScrollLock"
        | "Dead" | "Compose" | "Unidentified" | "" => return None,
        _ => {}
    }

    // Function row, by physical code (the `key` value for F-keys
    // is layout-stable on the web platform too, but code is the
    // capturer's unambiguous term).
    let fnum: Option<u8> = match (key, ev.code.as_str()) {
        ("F1", _) | (_, "F1") => Some(1),
        ("F2", _) | (_, "F2") => Some(2),
        ("F3", _) | (_, "F3") => Some(3),
        ("F4", _) | (_, "F4") => Some(4),
        ("F5", _) | (_, "F5") => Some(5),
        ("F6", _) | (_, "F6") => Some(6),
        ("F7", _) | (_, "F7") => Some(7),
        ("F8", _) | (_, "F8") => Some(8),
        ("F9", _) | (_, "F9") => Some(9),
        ("F10", _) | (_, "F10") => Some(10),
        ("F11", _) | (_, "F11") => Some(11),
        ("F12", _) | (_, "F12") => Some(12),
        _ => None,
    };
    if let Some(n) = fnum {
        if n <= 4 {
            let fin = [b'P', b'Q', b'R', b'S'][(n - 1) as usize];
            if m == 1 {
                return Some(vec![0x1b, b'O', fin]);
            }
            return Some(format!("\x1b[1;{}{}", m, fin as char).into_bytes());
        }
        let code = match n {
            5 => 15,
            6 => 17,
            7 => 18,
            8 => 19,
            9 => 20,
            10 => 21,
            11 => 23,
            _ => 24,
        };
        if m == 1 {
            return Some(format!("\x1b[{}~", code).into_bytes());
        }
        return Some(format!("\x1b[{};{}~", code, m).into_bytes());
    }

    // A single character: the key itself, with control and alt
    // semantics applied here, in the Brain.
    let mut chars = key.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        if ev.ctrl {
            let code = control_code(c)?;
            return Some(with_alt(vec![code], ev.alt));
        }
        let mut buf = [0u8; 4];
        let encoded = c.encode_utf8(&mut buf);
        return Some(with_alt(encoded.as_bytes().to_vec(), ev.alt));
    }

    None
}

/// Ctrl+character → its control code, for the characters that
/// have one.
fn control_code(c: char) -> Option<u8> {
    match c {
        ' ' => Some(0x00),
        'a'..='z' => Some(c as u8 - b'a' + 1),
        'A'..='Z' => Some(c as u8 - b'A' + 1),
        '@' => Some(0x00),
        '[' => Some(0x1b),
        '\\' => Some(0x1c),
        ']' => Some(0x1d),
        '^' => Some(0x1e),
        '_' => Some(0x1f),
        '?' => Some(0x7f),
        '2' => Some(0x00),
        '3' => Some(0x1b),
        '4' => Some(0x1c),
        '5' => Some(0x1d),
        '6' => Some(0x1e),
        '7' => Some(0x1f),
        '8' => Some(0x7f),
        _ => None,
    }
}

/// Encode literal text for delivery to the PTY: as typed input,
/// byte for byte. Every text caller means typing — a chip tap
/// types, the prompt types, committed keyboard text is typing —
/// and the shell must treat it as typing. Deliberately NOT
/// wrapped as a bracketed paste: a paste wrapper changes what
/// the shell does with the bytes and is fragile in exactly the
/// situations typing must survive (text that arrives before the
/// shell reaches its prompt, a submit that follows the text
/// immediately). CI evidence, 2026-11-22: with the wrapper, a
/// chip tap's command and its Enter interleaved wrongly in
/// bash, and type-ahead text lost its head; as typed input both
/// are exact. The mode argument stays in the signature beside
/// `encode_key`: text encoding is a Brain decision, and this is
/// where it would change.
pub fn encode_text(text: &str, _mode: &TermMode) -> Vec<u8> {
    text.as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(key: &str) -> KeyEvent {
        KeyEvent {
            key: key.into(),
            code: String::new(),
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    fn mode_default() -> TermMode {
        TermMode::SHOW_CURSOR
    }

    #[test]
    fn plain_characters_pass_through() {
        let m = mode_default();
        assert_eq!(encode_key(&ev("a"), &m), Some(b"a".to_vec()));
        assert_eq!(encode_key(&ev(";"), &m), Some(b";".to_vec()));
        assert_eq!(encode_key(&ev(" "), &m), Some(b" ".to_vec()));
    }

    #[test]
    fn control_characters_encode_in_the_brain() {
        let m = mode_default();
        let mut c = ev("c");
        c.ctrl = true;
        assert_eq!(encode_key(&c, &m), Some(vec![0x03]));
        let mut u = ev("u");
        u.ctrl = true;
        assert_eq!(encode_key(&u, &m), Some(vec![0x15]));
    }

    #[test]
    fn arrows_follow_cursor_key_mode() {
        let plain = mode_default();
        assert_eq!(
            encode_key(&ev("ArrowUp"), &plain),
            Some(b"\x1b[A".to_vec())
        );
        let app = TermMode::SHOW_CURSOR | TermMode::APP_CURSOR;
        assert_eq!(encode_key(&ev("ArrowUp"), &app), Some(b"\x1bOA".to_vec()));
        let mut ctrl = ev("ArrowRight");
        ctrl.ctrl = true;
        assert_eq!(
            encode_key(&ctrl, &plain),
            Some(b"\x1b[1;5C".to_vec())
        );
    }

    #[test]
    fn editing_and_function_keys() {
        let m = mode_default();
        assert_eq!(encode_key(&ev("Enter"), &m), Some(b"\r".to_vec()));
        assert_eq!(encode_key(&ev("Backspace"), &m), Some(vec![0x7f]));
        assert_eq!(encode_key(&ev("Delete"), &m), Some(b"\x1b[3~".to_vec()));
        let mut shift_tab = ev("Tab");
        shift_tab.shift = true;
        assert_eq!(encode_key(&shift_tab, &m), Some(b"\x1b[Z".to_vec()));
        assert_eq!(encode_key(&ev("F5"), &m), Some(b"\x1b[15~".to_vec()));
        assert_eq!(encode_key(&ev("Control"), &m), None);
    }

    #[test]
    fn alt_prefixes_escape() {
        let m = mode_default();
        let mut a = ev("b");
        a.alt = true;
        assert_eq!(encode_key(&a, &m), Some(b"\x1bb".to_vec()));
    }

    #[test]
    fn text_is_delivered_as_typed_input() {
        // Whatever the shell's modes, text goes as typing: a chip
        // tap's text must be indistinguishable from keystrokes.
        let off = mode_default();
        assert_eq!(encode_text("ls -la", &off), b"ls -la".to_vec());
        let on = TermMode::SHOW_CURSOR | TermMode::BRACKETED_PASTE;
        assert_eq!(encode_text("ls -la", &on), b"ls -la".to_vec());
    }
}
