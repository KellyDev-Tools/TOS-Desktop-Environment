//! Phase 1.7 (dynamic chips): a best-effort model of the shell's
//! current input line, fed from the same byte stream the Brain
//! forwards to the PTY.
//!
//! The chip system's type-ahead filter keys on this model: while
//! the user types, the Brain publishes the fragment of the word
//! being typed, and chip systems narrow the current listing to
//! matches. The model is deliberately conservative — it tracks
//! plain typing, backspace, Ctrl+U and Ctrl+W exactly; anything
//! it cannot model faithfully (escape sequences from arrow keys,
//! Tab completion, Ctrl+R search, history recall, cursor moves)
//! marks the line desynced, the filter clears, and the full
//! listing shows until the next submit / Ctrl+C / Ctrl+U resyncs
//! the model to a known-empty line. Keystrokes meant for a
//! full-screen program (vim, less) mostly desync the same way,
//! which is the safe failure: no filter beats a wrong filter.

/// A shell input line being modelled from raw terminal input.
#[derive(Debug)]
pub struct InputLine {
    line: String,
    synced: bool,
    utf8_pending: Vec<u8>,
}

impl Default for InputLine {
    fn default() -> Self {
        Self::new()
    }
}

impl InputLine {
    pub fn new() -> Self {
        Self {
            line: String::new(),
            synced: true,
            utf8_pending: Vec::new(),
        }
    }

    /// Feed one chunk of input bytes (as forwarded to the PTY).
    pub fn feed(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.feed_byte(b);
        }
    }

    fn feed_byte(&mut self, b: u8) {
        // Complete a multi-byte UTF-8 sequence first: continuation
        // bytes are 0x80..=0xBF.
        if !self.utf8_pending.is_empty() {
            if (0x80..=0xBF).contains(&b) {
                self.utf8_pending.push(b);
                let complete = std::str::from_utf8(&self.utf8_pending)
                    .ok()
                    .and_then(|s| {
                        let mut chars = s.chars();
                        match (chars.next(), chars.next()) {
                            (Some(c), None) => Some(c),
                            _ => None,
                        }
                    });
                match complete {
                    Some(c) => {
                        if self.synced {
                            self.line.push(c);
                        }
                        self.utf8_pending.clear();
                    }
                    None => {
                        if self.utf8_pending.len() > 4 {
                            // Not a valid sequence; drop it.
                            self.utf8_pending.clear();
                        }
                    }
                }
                return;
            }
            // Not a continuation byte: drop the incomplete sequence
            // and handle this byte on its own below.
            self.utf8_pending.clear();
        }

        match b {
            // Submit: the shell starts a fresh line afterwards.
            b'\r' | b'\n' => {
                self.line.clear();
                self.synced = true;
            }
            // Ctrl+C: the shell abandons the line.
            0x03 => {
                self.line.clear();
                self.synced = true;
            }
            // Ctrl+U: kill the whole line — known-empty afterwards
            // even if the model had desynced.
            0x15 => {
                self.line.clear();
                self.synced = true;
            }
            // Ctrl+W: kill the word before the cursor (plus any
            // spaces between it and the cursor).
            0x17 => {
                if self.synced {
                    let trimmed = self.line.trim_end_matches(' ');
                    let cut = trimmed.rfind(' ').map(|i| i + 1).unwrap_or(0);
                    self.line.truncate(cut);
                }
            }
            // Backspace.
            0x7F | 0x08 => {
                if self.synced {
                    self.line.pop();
                }
            }
            // No effect on the line's content: Ctrl+D (delete char
            // under cursor / EOF), Ctrl+L (redraw), Ctrl+Z
            // (suspend), NUL.
            0x04 | 0x0C | 0x1A | 0x00 => {}
            // UTF-8 lead byte: collect the sequence.
            0xC2..=0xF4 => {
                self.utf8_pending.push(b);
            }
            // Printable ASCII.
            0x20..=0x7E => {
                if self.synced {
                    self.line.push(b as char);
                }
            }
            // Everything else — ESC (arrow keys and other escape
            // sequences), Tab, Ctrl+R/P/N/A/E/B/F/K/T/Y and stray
            // control or continuation bytes — changes the shell's
            // line in ways this model does not track. Desync: the
            // filter clears until a resync point.
            _ => {
                self.synced = false;
            }
        }
    }

    /// The fragment of the word currently being typed: the text
    /// after the last space, then after the last '/' within it
    /// (so `cd ~/Doc` filters on `Doc`). None when the model is
    /// desynced or the fragment is empty — both mean "unfiltered".
    pub fn fragment(&self) -> Option<String> {
        if !self.synced {
            return None;
        }
        let word = self.line.rsplit(' ').next().unwrap_or("");
        let frag = word.rsplit('/').next().unwrap_or("");
        if frag.is_empty() {
            None
        } else {
            Some(frag.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fed(bytes: &[u8]) -> InputLine {
        let mut m = InputLine::new();
        m.feed(bytes);
        m
    }

    #[test]
    fn plain_typing_builds_the_fragment() {
        assert_eq!(fed(b"l").fragment().as_deref(), Some("l"));
        assert_eq!(fed(b"ls").fragment().as_deref(), Some("ls"));
        assert_eq!(fed(b"cd she").fragment().as_deref(), Some("she"));
        assert_eq!(fed(b"cd ").fragment(), None, "fresh word: unfiltered");
        assert_eq!(fed(b"").fragment(), None);
    }

    #[test]
    fn fragment_is_the_tail_after_a_path_separator() {
        assert_eq!(fed(b"cat ../READ").fragment().as_deref(), Some("READ"));
        assert_eq!(fed(b"cd ~/Doc").fragment().as_deref(), Some("Doc"));
        assert_eq!(fed(b"cd ~/").fragment(), None);
    }

    #[test]
    fn backspace_edits_the_model() {
        let mut m = fed(b"lsx");
        m.feed(&[0x7F]);
        assert_eq!(m.fragment().as_deref(), Some("ls"));
        m.feed(&[0x7F, 0x7F]);
        assert_eq!(m.fragment(), None);
        // Backspace past the start is a no-op, not a desync.
        m.feed(&[0x7F]);
        m.feed(b"ab");
        assert_eq!(m.fragment().as_deref(), Some("ab"));
    }

    #[test]
    fn submit_and_ctrl_c_clear_and_resync() {
        let mut m = fed(b"ls -la\r");
        assert_eq!(m.fragment(), None);
        m.feed(b"pw");
        assert_eq!(m.fragment().as_deref(), Some("pw"));

        let mut m = fed(b"half-typed");
        m.feed(&[0x03]);
        assert_eq!(m.fragment(), None);
        m.feed(b"ok");
        assert_eq!(m.fragment().as_deref(), Some("ok"));
    }

    #[test]
    fn ctrl_u_kills_the_line_and_resyncs() {
        let mut m = fed(b"garbage");
        m.feed(&[0x15]);
        assert_eq!(m.fragment(), None);
        m.feed(b"fresh");
        assert_eq!(m.fragment().as_deref(), Some("fresh"));
    }

    #[test]
    fn ctrl_w_kills_one_word() {
        // Killing the word lands on a fresh word boundary (the
        // line keeps its trailing space), so the filter clears —
        // and the next word typed filters from there.
        let mut m = fed(b"cd shell nix");
        m.feed(&[0x17]);
        assert_eq!(m.fragment(), None, "word killed: fresh word, unfiltered");
        m.feed(b"sh");
        assert_eq!(m.fragment().as_deref(), Some("sh"));

        let mut m = fed(b"cd she");
        m.feed(&[0x17]);
        assert_eq!(m.fragment(), None);
        m.feed(&[0x17]);
        assert_eq!(m.fragment(), None, "killing the command word: empty line");
    }

    #[test]
    fn unmodelable_input_desyncs_until_a_resync_point() {
        // Up-arrow (history recall): an escape sequence.
        let mut m = fed(b"ls");
        m.feed(&[0x1B, b'[', b'A']);
        assert_eq!(m.fragment(), None, "desynced: filter clears");
        // Plain typing does not resync on its own.
        m.feed(b"x");
        assert_eq!(m.fragment(), None);
        // Ctrl+U resyncs to a known-empty line.
        m.feed(&[0x15]);
        m.feed(b"ab");
        assert_eq!(m.fragment().as_deref(), Some("ab"));

        // Tab completion desyncs too.
        let mut m = fed(b"cd sh");
        m.feed(&[0x09]);
        assert_eq!(m.fragment(), None);
        // And Enter resyncs.
        m.feed(b"\r");
        m.feed(b"pw");
        assert_eq!(m.fragment().as_deref(), Some("pw"));
    }

    #[test]
    fn utf8_chars_are_modelled() {
        let mut m = InputLine::new();
        m.feed("café".as_bytes());
        assert_eq!(m.fragment().as_deref(), Some("café"));
        m.feed(&[0x7F]);
        assert_eq!(m.fragment().as_deref(), Some("caf"));
    }
}
