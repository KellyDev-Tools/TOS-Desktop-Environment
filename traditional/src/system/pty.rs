// Real Shell PTY Integration
// Based on "Dream.md" Shell API and "File Management.md" metadata injection
//
// This module spawns a real shell process (Fish/Zsh/Bash) in a PTY,
// intercepts OSC 1337 sequences for compositor synchronization,
// and provides bidirectional communication between the shell and TOS.

use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::time::Duration;
use std::os::unix::io::RawFd;

/// Configuration for the PTY shell
#[derive(Debug, Clone)]
pub struct PtyConfig {
    /// Shell binary path (e.g. "/usr/bin/fish", "/bin/zsh", "/bin/bash")
    pub shell_path: String,
    /// Initial working directory
    pub cwd: String,
    /// Environment variables to set
    pub env_vars: Vec<(String, String)>,
    /// Terminal size (columns, rows)
    pub size: (u16, u16),
    /// Whether to inject TOS Shell API hooks on startup
    pub inject_tos_hooks: bool,
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            shell_path: Self::detect_shell(),
            cwd: std::env::var("HOME").unwrap_or_else(|_| "/".to_string()),
            env_vars: Vec::new(),
            size: (120, 40),
            inject_tos_hooks: true,
        }
    }
}

impl PtyConfig {
    /// Auto-detect the best available shell
    fn detect_shell() -> String {
        // Prefer Fish (the origin idea's reference implementation)
        for shell in &["/usr/bin/fish", "/usr/local/bin/fish", "/usr/bin/zsh", "/bin/zsh", "/bin/bash"] {
            if std::path::Path::new(shell).exists() {
                return shell.to_string();
            }
        }
        "/bin/sh".to_string()
    }

    /// Get the shell name (for selecting the right TOS module)
    pub fn shell_name(&self) -> &str {
        if self.shell_path.contains("fish") { "fish" }
        else if self.shell_path.contains("zsh") { "zsh" }
        else if self.shell_path.contains("bash") { "bash" }
        else { "sh" }
    }
}

/// Events emitted by the PTY when TOS-relevant data is received
#[derive(Debug, Clone)]
pub enum PtyEvent {
    /// Regular terminal output (display text)
    Output(String),
    /// Directory changed (from OSC 1337 CurrentDir= or shell hook)
    DirectoryChanged(String),
    /// Shell requests a zoom level change
    ZoomRequest(u8),
    /// Shell sends metadata about files (JSON payload)
    FileMetadata { path: String, metadata_json: String },
    /// Shell process exited with a status code
    ProcessExited(i32),
    /// Shell sends a layout hint
    LayoutHint(String),
    /// Shell is ready (prompt displayed)
    ShellReady,
    /// Error occurred in PTY
    Error(String),
}

/// Commands that can be sent to the PTY
#[derive(Debug, Clone)]
pub enum PtyCommand {
    /// Write raw input to the shell (keyboard data)
    Write(String),
    /// Write a line (appends newline)
    WriteLine(String),
    /// Resize the terminal
    Resize(u16, u16),
    /// Send a signal (e.g., SIGINT = 2, SIGTSTP = 20)
    Signal(i32),
    /// Inject a TOS Shell API command
    InjectOsc(String, String), // key, value
    /// Close the PTY
    Close,
}

/// OSC sequence parser state machine
struct OscParser {
    state: OscState,
    buffer: String,
}

#[derive(Debug, PartialEq)]
enum OscState {
    Normal,
    EscSeen,       // Got \x1b
    OscStart,      // Got \x1b]
    OscAccum,      // Accumulating OSC content
}

impl OscParser {
    fn new() -> Self {
        Self {
            state: OscState::Normal,
            buffer: String::new(),
        }
    }

    /// Feed raw bytes from PTY and extract any OSC 1337 commands.
    /// Returns (cleaned_output, extracted_osc_events)
    fn feed(&mut self, data: &str) -> (String, Vec<PtyEvent>) {
        let mut output = String::new();
        let mut events = Vec::new();

        for ch in data.chars() {
            match self.state {
                OscState::Normal => {
                    if ch == '\x1b' {
                        self.state = OscState::EscSeen;
                    } else {
                        output.push(ch);
                    }
                }
                OscState::EscSeen => {
                    if ch == ']' {
                        self.state = OscState::OscStart;
                        self.buffer.clear();
                    } else {
                        // Not an OSC, pass through the escape + this char
                        output.push('\x1b');
                        output.push(ch);
                        self.state = OscState::Normal;
                    }
                }
                OscState::OscStart => {
                    // Check for "1337;" prefix
                    self.buffer.push(ch);
                    if self.buffer.len() == 5 {
                        if self.buffer == "1337;" {
                            self.buffer.clear();
                            self.state = OscState::OscAccum;
                        } else {
                            // Not our OSC, pass through
                            output.push_str("\x1b]");
                            output.push_str(&self.buffer);
                            self.buffer.clear();
                            self.state = OscState::Normal;
                        }
                    }
                }
                OscState::OscAccum => {
                    if ch == '\x07' || ch == '\x1b' {
                        // BEL or ST terminates the OSC
                        if let Some(event) = self.parse_osc_payload(&self.buffer.clone()) {
                            events.push(event);
                        }
                        self.buffer.clear();
                        self.state = OscState::Normal;
                    } else {
                        self.buffer.push(ch);
                    }
                }
            }
        }

        (output, events)
    }

    /// Parse a TOS OSC payload like "CurrentDir=/home/user"
    fn parse_osc_payload(&self, payload: &str) -> Option<PtyEvent> {
        let eq_pos = payload.find('=')?;
        let key = &payload[..eq_pos];
        let value = &payload[eq_pos + 1..];

        match key {
            "CurrentDir" => Some(PtyEvent::DirectoryChanged(value.to_string())),
            "ZoomLevel" => {
                value.parse::<u8>().ok().map(PtyEvent::ZoomRequest)
            }
            "SetLayout" => Some(PtyEvent::LayoutHint(value.to_string())),
            "FileMetadata" => {
                // value format: "path|json"
                if let Some(pipe_pos) = value.find('|') {
                    Some(PtyEvent::FileMetadata {
                        path: value[..pipe_pos].to_string(),
                        metadata_json: value[pipe_pos + 1..].to_string(),
                    })
                } else {
                    None
                }
            }
            "ShellReady" => Some(PtyEvent::ShellReady),
            _ => {
                println!("[PTY] Unknown OSC key: {}", key);
                None
            }
        }
    }
}

/// TOS Shell API hook scripts for different shells.
/// These are injected into the shell on startup to enable
/// metadata injection and compositor synchronization.
pub struct ShellHooks;

impl ShellHooks {
    /// Fish shell hooks (the reference implementation)
    pub fn fish_init() -> &'static str {
        r#"
# TOS Shell API - Fish Module
# Injected by the TOS compositor for spatial UI synchronization

function __tos_report_cwd --on-variable PWD
    printf '\e]1337;CurrentDir=%s\a' $PWD
end

function __tos_report_ready --on-event fish_prompt
    printf '\e]1337;ShellReady=1\a'
end

# Shadow 'ls' to inject file metadata
function ls --wraps='command ls'
    command ls $argv
    # In a full implementation, this would output OSC metadata
    # for each file with MIME type, thumbnail path, etc.
end

# Report initial directory
printf '\e]1337;CurrentDir=%s\a' $PWD
printf '\e]1337;ShellReady=1\a'
"#
    }

    /// Zsh shell hooks
    pub fn zsh_init() -> &'static str {
        r#"
# TOS Shell API - Zsh Module
__tos_precmd() {
    printf '\e]1337;CurrentDir=%s\a' "$PWD"
    printf '\e]1337;ShellReady=1\a'
}
precmd_functions+=(__tos_precmd)

# Report initial directory
printf '\e]1337;CurrentDir=%s\a' "$PWD"
printf '\e]1337;ShellReady=1\a'
"#
    }

    /// Bash shell hooks
    pub fn bash_init() -> &'static str {
        r#"
# TOS Shell API - Bash Module
__tos_prompt_command() {
    printf '\e]1337;CurrentDir=%s\a' "$PWD"
    printf '\e]1337;ShellReady=1\a'
}
PROMPT_COMMAND="__tos_prompt_command;$PROMPT_COMMAND"

# Report initial directory
printf '\e]1337;CurrentDir=%s\a' "$PWD"
printf '\e]1337;ShellReady=1\a'
"#
    }

    /// Get the init script for a given shell name
    pub fn get_init(shell_name: &str) -> &'static str {
        match shell_name {
            "fish" => Self::fish_init(),
            "zsh" => Self::zsh_init(),
            "bash" => Self::bash_init(),
            _ => Self::bash_init(), // Fallback
        }
    }
}

/// A real PTY handle wrapping a forked shell process.
/// Uses Unix-specific APIs (forkpty).
pub struct PtyHandle {
    /// Channel to send commands to the PTY writer thread
    pub cmd_tx: Sender<PtyCommand>,
    /// Channel to receive events from the PTY reader thread
    pub event_rx: Receiver<PtyEvent>,
    /// The child process PID
    pub child_pid: Option<u32>,
    /// The PTY master file descriptor
    master_fd: Option<RawFd>,
    /// Configuration used
    pub config: PtyConfig,
}

impl PtyHandle {
    /// Spawn a new shell in a PTY.
    ///
    /// This forks the process:
    /// - Child process: exec's the shell
    /// - Parent process: reads/writes the PTY master fd
    ///
    /// Returns None if the fork fails.
    pub fn spawn(config: PtyConfig) -> Option<Self> {
        let (cmd_tx, cmd_rx) = channel::<PtyCommand>();
        let (event_tx, event_rx) = channel::<PtyEvent>();

        // Use libc forkpty
        let mut winsize = libc::winsize {
            ws_row: config.size.1,
            ws_col: config.size.0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        let mut master_fd: libc::c_int = 0;

        let pid = unsafe {
            libc::forkpty(
                &mut master_fd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut winsize,
            )
        };

        if pid < 0 {
            let _ = event_tx.send(PtyEvent::Error("forkpty failed".to_string()));
            return None;
        }

        if pid == 0 {
            // ─── Child Process ────────────────────────────
            // Set working directory
            let _ = std::env::set_current_dir(&config.cwd);

            // Set TOS environment variables
            std::env::set_var("TOS_SHELL_API", "1");
            std::env::set_var("TOS_VERSION", "0.1.0");
            std::env::set_var("TERM", "xterm-256color");

            for (key, value) in &config.env_vars {
                std::env::set_var(key, value);
            }

            // Exec the shell
            let shell = std::ffi::CString::new(config.shell_path.as_str()).unwrap();
            let shell_name = std::ffi::CString::new(
                format!("-{}", config.shell_path.rsplit('/').next().unwrap_or("sh"))
            ).unwrap();

            unsafe {
                libc::execl(
                    shell.as_ptr(),
                    shell_name.as_ptr(),
                    std::ptr::null::<libc::c_char>(),
                );
                // If we get here, exec failed
                libc::_exit(127);
            }
        }

        // ─── Parent Process ──────────────────────────────
        let child_pid = pid as u32;
        println!("[PTY] Shell spawned: {} (PID: {}, master_fd: {})", config.shell_path, child_pid, master_fd);

        // Set master fd to non-blocking
        unsafe {
            let flags = libc::fcntl(master_fd, libc::F_GETFL);
            libc::fcntl(master_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        let inject_hooks = config.inject_tos_hooks;
        let shell_name = config.shell_name().to_string();
        let master_fd_copy = master_fd;

        // Spawn reader thread
        let reader_event_tx = event_tx.clone();
        thread::Builder::new()
            .name("pty-reader".to_string())
            .spawn(move || {
                Self::reader_loop(master_fd_copy, reader_event_tx);
            })
            .ok()?;

        // Spawn writer thread
        let writer_master_fd = master_fd;
        thread::Builder::new()
            .name("pty-writer".to_string())
            .spawn(move || {
                Self::writer_loop(writer_master_fd, cmd_rx, child_pid);
            })
            .ok()?;

        // Inject TOS hooks after a short delay
        if inject_hooks {
            let hook_tx = cmd_tx.clone();
            let hook_shell = shell_name;
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(500));
                let init_script = ShellHooks::get_init(&hook_shell);
                // Send the init script as input to the shell
                let _ = hook_tx.send(PtyCommand::WriteLine(init_script.to_string()));
                println!("[PTY] TOS hooks injected for {}", hook_shell);
            });
        }

        Some(Self {
            cmd_tx,
            event_rx,
            child_pid: Some(child_pid),
            master_fd: Some(master_fd),
            config,
        })
    }

    /// Reader loop: reads from PTY master fd, parses OSC sequences,
    /// and emits PtyEvents
    fn reader_loop(master_fd: RawFd, event_tx: Sender<PtyEvent>) {
        let mut parser = OscParser::new();
        let mut buf = [0u8; 4096];

        loop {
            let n = unsafe {
                libc::read(
                    master_fd,
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len(),
                )
            };

            if n > 0 {
                let data = String::from_utf8_lossy(&buf[..n as usize]).to_string();
                let (clean_output, events) = parser.feed(&data);

                // Send clean terminal output
                if !clean_output.is_empty() {
                    let _ = event_tx.send(PtyEvent::Output(clean_output));
                }

                // Send parsed events
                for event in events {
                    let _ = event_tx.send(event);
                }
            } else if n == 0 {
                // EOF — shell exited
                let _ = event_tx.send(PtyEvent::ProcessExited(0));
                break;
            } else {
                let errno = unsafe { *libc::__errno_location() };
                if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
                    // Non-blocking, no data available
                    thread::sleep(Duration::from_millis(10));
                } else if errno == libc::EIO {
                    // PTY closed (child exited)
                    let _ = event_tx.send(PtyEvent::ProcessExited(0));
                    break;
                } else {
                    let _ = event_tx.send(PtyEvent::Error(format!("PTY read error: errno {}", errno)));
                    break;
                }
            }
        }

        println!("[PTY] Reader loop exited");
    }

    /// Writer loop: receives PtyCommands and writes to the PTY master fd
    fn writer_loop(master_fd: RawFd, cmd_rx: Receiver<PtyCommand>, child_pid: u32) {
        loop {
            match cmd_rx.recv() {
                Ok(PtyCommand::Write(data)) => {
                    let bytes = data.as_bytes();
                    unsafe {
                        libc::write(
                            master_fd,
                            bytes.as_ptr() as *const libc::c_void,
                            bytes.len(),
                        );
                    }
                }
                Ok(PtyCommand::WriteLine(data)) => {
                    let line = format!("{}\n", data);
                    unsafe {
                        libc::write(
                            master_fd,
                            line.as_ptr() as *const libc::c_void,
                            line.len(),
                        );
                    }
                }
                Ok(PtyCommand::Resize(cols, rows)) => {
                    let winsize = libc::winsize {
                        ws_row: rows,
                        ws_col: cols,
                        ws_xpixel: 0,
                        ws_ypixel: 0,
                    };
                    unsafe {
                        libc::ioctl(master_fd, libc::TIOCSWINSZ, &winsize);
                    }
                    println!("[PTY] Resized to {}x{}", cols, rows);
                }
                Ok(PtyCommand::Signal(sig)) => {
                    unsafe {
                        libc::kill(child_pid as i32, sig);
                    }
                    println!("[PTY] Sent signal {} to PID {}", sig, child_pid);
                }
                Ok(PtyCommand::InjectOsc(key, value)) => {
                    let osc = format!("\x1b]1337;{}={}\x07", key, value);
                    unsafe {
                        libc::write(
                            master_fd,
                            osc.as_ptr() as *const libc::c_void,
                            osc.len(),
                        );
                    }
                }
                Ok(PtyCommand::Close) | Err(_) => {
                    println!("[PTY] Writer loop closing");
                    unsafe {
                        libc::close(master_fd);
                    }
                    break;
                }
            }
        }
    }

    /// Send a command to the shell
    pub fn send(&self, cmd: PtyCommand) {
        let _ = self.cmd_tx.send(cmd);
    }

    /// Write raw text to the shell
    pub fn write_str(&self, s: &str) {
        self.send(PtyCommand::Write(s.to_string()));
    }

    /// Execute a command in the shell
    pub fn execute(&self, cmd: &str) {
        self.send(PtyCommand::WriteLine(cmd.to_string()));
    }

    /// Send Ctrl+C (SIGINT)
    pub fn interrupt(&self) {
        self.send(PtyCommand::Signal(libc::SIGINT));
    }

    /// Resize the PTY
    pub fn resize(&self, cols: u16, rows: u16) {
        self.send(PtyCommand::Resize(cols, rows));
    }

    /// Poll for the next event (non-blocking)
    pub fn try_recv(&self) -> Option<PtyEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Drain all pending events
    pub fn drain_events(&self) -> Vec<PtyEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.try_recv() {
            events.push(event);
        }
        events
    }

    /// Close the PTY and kill the child process
    pub fn close(&self) {
        let _ = self.cmd_tx.send(PtyCommand::Close);
        if let Some(pid) = self.child_pid {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }
    }
}

impl Drop for PtyHandle {
    fn drop(&mut self) {
        self.close();
    }
}

/// Manager for multiple PTY sessions (one per terminal surface)
pub struct PtyManager {
    sessions: std::collections::HashMap<u32, PtyHandle>, // surface_id -> PTY
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
        }
    }

    /// Spawn a new PTY session for a given surface
    pub fn spawn_for_surface(&mut self, surface_id: u32, config: PtyConfig) -> bool {
        if let Some(handle) = PtyHandle::spawn(config) {
            println!("[PtyMgr] Session created for surface {}", surface_id);
            self.sessions.insert(surface_id, handle);
            true
        } else {
            println!("[PtyMgr] Failed to spawn session for surface {}", surface_id);
            false
        }
    }

    /// Get a PTY handle for a surface
    pub fn get(&self, surface_id: u32) -> Option<&PtyHandle> {
        self.sessions.get(&surface_id)
    }

    /// Remove and close a PTY session
    pub fn close_session(&mut self, surface_id: u32) {
        if let Some(handle) = self.sessions.remove(&surface_id) {
            handle.close();
            println!("[PtyMgr] Session closed for surface {}", surface_id);
        }
    }

    /// Drain events from all sessions, returning (surface_id, event) pairs
    pub fn drain_all_events(&self) -> Vec<(u32, PtyEvent)> {
        let mut all_events = Vec::new();
        for (&surface_id, handle) in &self.sessions {
            for event in handle.drain_events() {
                all_events.push((surface_id, event));
            }
        }
        all_events
    }

    /// Number of active sessions
    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osc_parser_basic() {
        let mut parser = OscParser::new();

        let input = "Hello \x1b]1337;CurrentDir=/home/user\x07 World";
        let (output, events) = parser.feed(input);

        assert_eq!(output, "Hello  World");
        assert_eq!(events.len(), 1);
        match &events[0] {
            PtyEvent::DirectoryChanged(dir) => assert_eq!(dir, "/home/user"),
            _ => panic!("Expected DirectoryChanged"),
        }
    }

    #[test]
    fn test_osc_parser_multiple() {
        let mut parser = OscParser::new();

        let input = "\x1b]1337;CurrentDir=/tmp\x07output\x1b]1337;ZoomLevel=3\x07end";
        let (output, events) = parser.feed(input);

        assert_eq!(output, "outputend");
        assert_eq!(events.len(), 2);

        match &events[0] {
            PtyEvent::DirectoryChanged(dir) => assert_eq!(dir, "/tmp"),
            _ => panic!("Expected DirectoryChanged"),
        }
        match &events[1] {
            PtyEvent::ZoomRequest(level) => assert_eq!(*level, 3),
            _ => panic!("Expected ZoomRequest"),
        }
    }

    #[test]
    fn test_osc_parser_no_osc() {
        let mut parser = OscParser::new();
        let (output, events) = parser.feed("just regular terminal output\r\n");
        assert_eq!(output, "just regular terminal output\r\n");
        assert!(events.is_empty());
    }

    #[test]
    fn test_osc_parser_non_tos_osc() {
        let mut parser = OscParser::new();
        // OSC 0 (set window title) — not our sequence
        let (output, events) = parser.feed("\x1b]0;My Title\x07visible text");
        // Should pass through (with the escape sequence preserved)
        assert!(events.is_empty());
        assert!(output.contains("visible text"));
    }

    #[test]
    fn test_osc_parser_split_across_reads() {
        let mut parser = OscParser::new();

        // First chunk: start of OSC
        let (out1, ev1) = parser.feed("text\x1b]1337;Current");
        assert_eq!(out1, "text");
        assert!(ev1.is_empty());

        // Second chunk: rest of OSC
        let (out2, ev2) = parser.feed("Dir=/home\x07more");
        assert_eq!(out2, "more");
        assert_eq!(ev2.len(), 1);
        match &ev2[0] {
            PtyEvent::DirectoryChanged(dir) => assert_eq!(dir, "/home"),
            _ => panic!("Expected DirectoryChanged"),
        }
    }

    #[test]
    fn test_osc_file_metadata() {
        let mut parser = OscParser::new();
        let input = "\x1b]1337;FileMetadata=/home/doc.txt|{\"mime\":\"text/plain\",\"size\":1024}\x07";
        let (_, events) = parser.feed(input);

        assert_eq!(events.len(), 1);
        match &events[0] {
            PtyEvent::FileMetadata { path, metadata_json } => {
                assert_eq!(path, "/home/doc.txt");
                assert!(metadata_json.contains("text/plain"));
            }
            _ => panic!("Expected FileMetadata"),
        }
    }

    #[test]
    fn test_pty_config_defaults() {
        let config = PtyConfig::default();
        assert!(!config.shell_path.is_empty());
        assert_eq!(config.size, (120, 40));
        assert!(config.inject_tos_hooks);
    }

    #[test]
    fn test_shell_hooks_exist() {
        let fish = ShellHooks::fish_init();
        assert!(fish.contains("__tos_report_cwd"));
        assert!(fish.contains("CurrentDir"));

        let zsh = ShellHooks::zsh_init();
        assert!(zsh.contains("__tos_precmd"));

        let bash = ShellHooks::bash_init();
        assert!(bash.contains("PROMPT_COMMAND"));
    }

    #[test]
    fn test_pty_manager() {
        let mgr = PtyManager::new();
        assert_eq!(mgr.active_count(), 0);
    }
}
