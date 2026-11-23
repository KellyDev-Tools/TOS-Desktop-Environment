//! The Brain-owned screen grid for one terminal session.
//!
//! The PTY stream is parsed exactly once, here, into cells,
//! attributes, cursor, modes, and scrollback history (Architecture
//! §26.3). Faces never see the stream: they receive snapshots of
//! this grid as display state and paint them. Input encoding also
//! lives beside this module (see `keymap`), because what a key
//! means depends on the emulation state held here.

use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::vte::ansi::{Color, NamedColor};
use alacritty_terminal::vte::ansi::Processor;
use serde::Serialize;

#[derive(Clone, Copy)]
struct Dims {
    cols: usize,
    rows: usize,
}

impl Dimensions for Dims {
    fn total_lines(&self) -> usize {
        self.rows
    }
    fn screen_lines(&self) -> usize {
        self.rows
    }
    fn columns(&self) -> usize {
        self.cols
    }
}

/// One run of consecutive cells sharing a style.
#[derive(Serialize, Clone, PartialEq)]
pub struct GridSpan {
    pub text: String,
    /// "#rrggbb", or "fg" / "bg" for the face's default colours.
    pub fg: String,
    pub bg: String,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
}

impl GridSpan {
    fn plain(text: String) -> Self {
        Self {
            text,
            fg: "fg".into(),
            bg: "bg".into(),
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            strike: false,
        }
    }

    fn same_style(&self, other: &GridSpan) -> bool {
        self.fg == other.fg
            && self.bg == other.bg
            && self.bold == other.bold
            && self.dim == other.dim
            && self.italic == other.italic
            && self.underline == other.underline
            && self.strike == other.strike
    }
}

/// The whole visible screen, as published to faces.
#[derive(Serialize, Clone)]
pub struct GridSnapshot {
    pub cols: usize,
    pub rows: usize,
    /// Monotonic counter bumped on every grid change; faces ignore
    /// stale out-of-order updates.
    pub seq: u64,
    /// Lines scrolled back into history (0 = live bottom).
    pub display_offset: usize,
    pub cursor_col: usize,
    pub cursor_row: usize,
    pub cursor_visible: bool,
    /// One entry per visible row; each row is a list of spans.
    pub lines: Vec<Vec<GridSpan>>,
}

pub struct ScreenGrid {
    term: Term<VoidListener>,
    processor: Processor,
    cols: usize,
    rows: usize,
    seq: u64,
}

impl ScreenGrid {
    pub fn new(cols: usize, rows: usize) -> Self {
        let config = Config {
            scrolling_history: 10_000,
            ..Default::default()
        };
        let term = Term::new(config, &Dims { cols, rows }, VoidListener);
        Self {
            term,
            processor: Processor::new(),
            cols,
            rows,
            seq: 0,
        }
    }

    /// Parse PTY output into the grid. Returns true if the sequence
    /// advanced (there is something new to publish).
    pub fn feed(&mut self, bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return false;
        }
        self.processor.advance(&mut self.term, bytes);
        self.seq += 1;
        true
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        if cols == self.cols && rows == self.rows {
            return;
        }
        self.cols = cols;
        self.rows = rows;
        self.term.resize(Dims { cols, rows });
        self.seq += 1;
    }

    /// Move the display window into scrollback history
    /// (positive = older). The grid itself does not change.
    pub fn scroll(&mut self, delta_lines: i32) {
        use alacritty_terminal::grid::Scroll;
        self.term.scroll_display(Scroll::Delta(delta_lines));
        self.seq += 1;
    }

    /// Snap back to the live bottom of the grid. Returns true if the
    /// viewport was actually scrolled up into history.
    pub fn scroll_to_bottom(&mut self) -> bool {
        use alacritty_terminal::grid::Scroll;
        if self.term.grid().display_offset() == 0 {
            return false;
        }
        self.term.scroll_display(Scroll::Bottom);
        self.seq += 1;
        true
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn mode(&self) -> &TermMode {
        self.term.mode()
    }

    pub fn snapshot(&self) -> GridSnapshot {
        let content = self.term.renderable_content();
        let display_offset = content.display_offset;
        let mut lines: Vec<Vec<GridSpan>> = vec![Vec::new(); self.rows];
        for indexed in content.display_iter {
            let line_idx = indexed.point.line.0;
            if line_idx < 0 || line_idx as usize >= self.rows {
                continue;
            }
            let cell = &indexed.cell;
            if cell
                .flags
                .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
            {
                continue;
            }
            let ch = if cell.c == '\0' || cell.flags.contains(Flags::HIDDEN) {
                ' '
            } else {
                cell.c
            };
            let mut span = span_for_cell(cell);
            span.text = ch.to_string();
            let row = &mut lines[line_idx as usize];
            match row.last_mut() {
                Some(last) if last.same_style(&span) => last.text.push(ch),
                _ => row.push(span),
            }
        }
        // Trim trailing blanks per row: pop wholly-blank plain
        // spans, and shave trailing spaces off a plain final span
        // (unwritten cells merge into it).
        for row in lines.iter_mut() {
            loop {
                let plain = match row.last() {
                    Some(last) => {
                        last.fg == "fg"
                            && last.bg == "bg"
                            && !last.bold
                            && !last.dim
                            && !last.italic
                            && !last.underline
                            && !last.strike
                    }
                    None => break,
                };
                if !plain {
                    break;
                }
                let last = row.last_mut().unwrap();
                let trimmed_len = last.text.trim_end().len();
                if trimmed_len == 0 {
                    row.pop();
                } else {
                    last.text.truncate(trimmed_len);
                    break;
                }
            }
        }
        let cursor = content.cursor;
        GridSnapshot {
            cols: self.cols,
            rows: self.rows,
            seq: self.seq,
            display_offset,
            cursor_col: cursor.point.column.0,
            cursor_row: cursor.point.line.0.max(0) as usize,
            cursor_visible: self.term.mode().contains(TermMode::SHOW_CURSOR)
                && cursor.shape != alacritty_terminal::vte::ansi::CursorShape::Hidden,
            lines,
        }
    }

    /// The visible rows as plain text (tests, debugging).
    pub fn visible_text(&self) -> Vec<String> {
        self.snapshot()
            .lines
            .iter()
            .map(|row| row.iter().map(|s| s.text.as_str()).collect::<String>())
            .collect()
    }
}

fn span_for_cell(cell: &Cell) -> GridSpan {
    let flags = cell.flags;
    let mut fg = color_string(cell.fg, true);
    let mut bg = color_string(cell.bg, false);
    if flags.contains(Flags::INVERSE) {
        std::mem::swap(&mut fg, &mut bg);
    }
    GridSpan {
        text: String::new(),
        fg,
        bg,
        bold: flags.contains(Flags::BOLD),
        dim: flags.contains(Flags::DIM),
        italic: flags.contains(Flags::ITALIC),
        underline: flags.contains(Flags::UNDERLINE),
        strike: flags.contains(Flags::STRIKEOUT),
    }
}

fn color_string(color: Color, is_fg: bool) -> String {
    match color {
        Color::Named(NamedColor::Foreground) => "fg".into(),
        Color::Named(NamedColor::Background) => "bg".into(),
        Color::Named(NamedColor::Cursor) => "cursor".into(),
        Color::Named(named) => {
            let idx = named as usize;
            palette_hex(idx).unwrap_or_else(|| if is_fg { "fg".into() } else { "bg".into() })
        }
        Color::Spec(rgb) => format!("#{:02x}{:02x}{:02x}", rgb.r, rgb.g, rgb.b),
        Color::Indexed(i) => palette_hex(i as usize).unwrap_or_else(|| "fg".into()),
    }
}

/// The standard xterm 256-colour palette as hex strings.
fn palette_hex(idx: usize) -> Option<String> {
    const BASE16: [&str; 16] = [
        "#000000", "#cc0000", "#00cc00", "#cccc00", "#0000cc", "#cc00cc", "#00cccc", "#cccccc",
        "#555555", "#ff5555", "#55ff55", "#ffff55", "#5555ff", "#ff55ff", "#55ffff", "#ffffff",
    ];
    match idx {
        0..=15 => Some(BASE16[idx].to_string()),
        16..=231 => {
            let n = idx - 16;
            let conv = |v: usize| if v == 0 { 0 } else { 55 + v * 40 };
            Some(format!(
                "#{:02x}{:02x}{:02x}",
                conv(n / 36),
                conv((n / 6) % 6),
                conv(n % 6)
            ))
        }
        232..=255 => {
            let v = 8 + (idx - 232) * 10;
            Some(format!("#{:02x}{:02x}{:02x}", v, v, v))
        }
        _ => None,
    }
}

// Referenced by snapshot building for line/column typing.
#[allow(dead_code)]
fn _dims_check(_: Line, _: Column) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_and_redraw_stay_on_one_row() {
        // The shell's line editor redraws the input line in place:
        // carriage return + rewrite, never a newline. The grid must
        // show exactly one row of text — this is the staircase
        // fault, expressed against the Brain's grid.
        let mut grid = ScreenGrid::new(40, 5);
        grid.feed(b"$ ");
        grid.feed(b"\r$ e");
        grid.feed(b"\r$ ec");
        grid.feed(b"\r$ ech");
        grid.feed(b"\r$ echo");
        grid.feed(b"\r$ echo hello\r\n");
        grid.feed(b"hello\r\n$ ");
        let text = grid.visible_text();
        let non_empty: Vec<&String> = text.iter().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(non_empty.len(), 3, "rows were: {:?}", text);
        assert_eq!(non_empty[0].trim(), "$ echo hello");
        assert_eq!(non_empty[1].trim(), "hello");
    }

    #[test]
    fn cursor_and_modes_are_tracked() {
        let mut grid = ScreenGrid::new(20, 3);
        grid.feed(b"ab");
        let snap = grid.snapshot();
        assert_eq!(snap.cursor_col, 2);
        assert!(snap.cursor_visible);
        // Application cursor keys mode (DECCKM) set by the app.
        grid.feed(b"\x1b[?1h");
        assert!(grid.mode().contains(TermMode::APP_CURSOR));
        grid.feed(b"\x1b[?2004h");
        assert!(grid.mode().contains(TermMode::BRACKETED_PASTE));
    }

    #[test]
    fn styles_become_spans() {
        let mut grid = ScreenGrid::new(20, 3);
        grid.feed(b"\x1b[1;31mERR\x1b[0m plain");
        let snap = grid.snapshot();
        let row = &snap.lines[0];
        assert_eq!(row.len(), 2);
        assert!(row[0].bold);
        assert_eq!(row[0].fg, "#cc0000");
        assert_eq!(row[0].text, "ERR");
        assert_eq!(row[1].text, " plain");
    }

    #[test]
    fn scroll_to_bottom_only_bumps_when_scrolled() {
        let mut grid = ScreenGrid::new(20, 3);
        for i in 0..10 {
            grid.feed(format!("line {}\r\n", i).as_bytes());
        }
        let seq_before = grid.seq();
        assert!(!grid.scroll_to_bottom());
        assert_eq!(grid.seq(), seq_before);

        grid.scroll(3);
        assert!(grid.snapshot().display_offset > 0);
        assert!(grid.scroll_to_bottom());
        assert_eq!(grid.snapshot().display_offset, 0);
    }
}
