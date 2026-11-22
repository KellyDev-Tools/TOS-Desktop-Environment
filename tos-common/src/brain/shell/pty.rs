use crate::{TerminalLine, TosState};
use chrono::Local;
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::shell::{OscEvent, OscParser};

// ---------------------------------------------------------------------
// Opt-in PTY byte trace.
//
// Rendering faults on device are hard to attribute without seeing
// the exact bytes: what a face sent, and what the shell answered.
// With `TOS_PTY_TRACE` set in the Brain's environment, every input
// write and every output chunk is logged to stderr in escaped
// form, so the two directions can be compared byte for byte. Off
// by default, and capped per direction so a long session cannot
// flood the log.
// ---------------------------------------------------------------------

static PTY_TRACE_IN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static PTY_TRACE_OUT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
const PTY_TRACE_MAX_CHUNKS: u64 = 2000;
const PTY_TRACE_MAX_BYTES: usize = 512;

fn pty_trace_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("TOS_PTY_TRACE").is_some())
}

/// Render bytes readably for the trace: printable ASCII as-is,
/// the whitespace controls as `\r` / `\n` / `\t`, everything else
/// as `\xNN`.
fn escape_trace_bytes(data: &[u8]) -> String {
    let mut text = String::with_capacity(data.len());
    for &b in data {
        match b {
            b'\r' => text.push_str("\\r"),
            b'\n' => text.push_str("\\n"),
            b'\t' => text.push_str("\\t"),
            0x20..=0x7e => text.push(b as char),
            _ => text.push_str(&format!("\\x{:02x}", b)),
        }
    }
    text
}

fn trace_pty_bytes(dir: &str, counter: &std::sync::atomic::AtomicU64, data: &[u8]) {
    if !pty_trace_enabled() {
        return;
    }
    let n = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if n >= PTY_TRACE_MAX_CHUNKS {
        return;
    }
    let shown = &data[..data.len().min(PTY_TRACE_MAX_BYTES)];
    let more = if data.len() > shown.len() {
        format!(" (+{} more bytes)", data.len() - shown.len())
    } else {
        String::new()
    };
    eprintln!(
        "[pty-trace] {} #{} {} bytes: {}{}",
        dir,
        n,
        data.len(),
        escape_trace_bytes(shown),
        more
    );
}

/// Cap on the raw scrollback the Brain retains per shell (Phase 1.5).
pub const SCROLLBACK_CAP_BYTES: usize = 256 * 1024;

/// Raw PTY output history retained by the Brain (Phase 1.5).
///
/// The shell process itself is Brain-owned and outlives any face, but
/// a terminal widget can only paint what it receives — without this
/// buffer a (re)connecting face starts from a blank screen even
/// though its session is intact. The read loop appends every chunk
/// here before broadcasting it; when a new face connects, the IPC
/// layer replays the snapshot to that client alone (see
/// `IpcHandler::replay_terminal_to`).
///
/// Trimming prefers a newline boundary so a replay does not begin
/// mid-line where avoidable; it can still begin mid escape-sequence
/// after very long unbroken output, which terminals tolerate as a
/// transient glitch at the top of the replayed history.
pub struct ScrollbackBuffer {
    cap: usize,
    data: std::collections::VecDeque<u8>,
}

impl ScrollbackBuffer {
    pub fn new(cap: usize) -> Self {
        Self {
            cap,
            data: std::collections::VecDeque::new(),
        }
    }

    pub fn append(&mut self, bytes: &[u8]) {
        self.data.extend(bytes.iter().copied());
        if self.data.len() > self.cap {
            let excess = self.data.len() - self.cap;
            let cut = (excess..self.data.len())
                .find(|&i| self.data[i] == b'\n')
                .map(|i| i + 1)
                .unwrap_or(excess);
            self.data.drain(..cut);
        }
    }

    pub fn snapshot(&self) -> Vec<u8> {
        self.data.iter().copied().collect()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Input/output activity shared between the PTY writer, the read
/// loop, and the command-completion watcher (Phase 1.6).
pub struct PtyActivity {
    /// Last write that submitted a command (contained CR or LF).
    last_input_ms: std::sync::atomic::AtomicU64,
    /// Last chunk read from the PTY.
    last_output_ms: std::sync::atomic::AtomicU64,
    /// A command was submitted and has not yet been seen to finish.
    command_pending: std::sync::atomic::AtomicBool,
}

/// Monotonic milliseconds for activity timestamps.
fn activity_now_ms() -> u64 {
    static EPOCH: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    EPOCH.get_or_init(std::time::Instant::now).elapsed().as_millis() as u64
}

/// Phase 1.6: command-completion watcher. When a command has been
/// submitted (its input contained Enter) and the PTY then goes
/// quiet, the command has finished — whatever shell this is, with
/// no OSC integration required. On completion, sync the hub's cwd
/// from the shell process itself (Linux /proc/<pid>/cwd: exact for
/// any shell) and re-issue the directory listing as a new
/// generation, so chip systems see fresh chips after every command
/// — `ls`, `pwd`, anything.
fn completion_watcher(
    state: Arc<Mutex<TosState>>,
    sector_id: uuid::Uuid,
    hub_id: uuid::Uuid,
    child_pid: Option<u32>,
    activity: Arc<PtyActivity>,
    alive: Arc<std::sync::atomic::AtomicBool>,
) {
    use std::sync::atomic::Ordering;
    while alive.load(Ordering::Relaxed) {
        thread::sleep(std::time::Duration::from_millis(100));
        if !activity.command_pending.load(Ordering::SeqCst) {
            continue;
        }
        let now = activity_now_ms();
        let quiet_out = now.saturating_sub(activity.last_output_ms.load(Ordering::SeqCst));
        let quiet_in = now.saturating_sub(activity.last_input_ms.load(Ordering::SeqCst));
        if quiet_out < 300 || quiet_in < 300 {
            continue;
        }
        activity.command_pending.store(false, Ordering::SeqCst);

        let mut state_lock = state.lock().unwrap();
        if let Some(pid) = child_pid {
            if let Ok(real_cwd) = std::fs::read_link(format!("/proc/{pid}/cwd")) {
                if let Some(sector) = state_lock.sectors.iter_mut().find(|s| s.id == sector_id) {
                    if let Some(hub) = sector.hubs.iter_mut().find(|h| h.id == hub_id) {
                        if hub.current_directory != real_cwd {
                            hub.current_directory = real_cwd;
                        }
                    }
                }
            }
        }
        crate::brain::sector::SectorManager::refresh_listing_for(
            &mut state_lock,
            sector_id,
            hub_id,
        );
    }
}

/// §15.2: PTY-backed Shell Implementation (Desktop Only)
pub struct PtyShell {
    _state: Arc<Mutex<TosState>>,
    _sector_id: uuid::Uuid,
    _hub_id: uuid::Uuid,
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    _child: Box<dyn Child + Send + Sync>,
    ipc: Arc<Mutex<Option<std::sync::Weak<crate::brain::ipc_handler::IpcHandler>>>>,
    /// The session's screen grid: the one emulation of this
    /// terminal, fed by the read loop from the PTY stream.
    grid: Arc<Mutex<super::grid::ScreenGrid>>,
    activity: Arc<PtyActivity>,
    alive: Arc<std::sync::atomic::AtomicBool>,
    /// Phase 1.7 (dynamic chips): model of the shell's input
    /// line, fed from the same bytes write() forwards to the PTY.
    input_line: super::input_line::InputLine,
    /// The chip filter last published to state, so write() only
    /// touches state (and bumps versions) when it changes.
    chip_filter_sent: Option<String>,
}

impl Drop for PtyShell {
    fn drop(&mut self) {
        // Stop the command-completion watcher thread.
        self.alive.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

impl PtyShell {
    pub fn new(
        state: Arc<Mutex<TosState>>,
        modules: Arc<crate::brain::module_manager::ModuleManager>,
        ai: Arc<crate::services::AiService>,
        heuristic: Arc<crate::services::HeuristicService>,
        sector_id: uuid::Uuid,
        hub_id: uuid::Uuid,
    ) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let (shell_id, cwd) = {
            let lock = state.lock().unwrap();
            let hub_opt = lock
                .sectors
                .iter()
                .find(|s| s.id == sector_id)
                .and_then(|s| s.hubs.iter().find(|h| h.id == hub_id));

            let id = hub_opt
                .and_then(|h| h.shell_module.clone())
                .unwrap_or_else(|| "tos-shell-fish".to_string());
            let current_dir = hub_opt
                .map(|h| h.current_directory.clone())
                .unwrap_or_else(|| {
                    dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"))
                });

            (id, current_dir)
        };

        let resolve_from_env = || -> (String, Vec<String>) {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
            let verified = if std::path::Path::new(&shell).exists() {
                shell
            } else {
                let fallbacks = [
                    "/bin/bash",
                    "/bin/zsh",
                    "/bin/sh",
                    "/usr/bin/bash",
                    "/usr/bin/sh",
                ];
                fallbacks
                    .iter()
                    .find(|&&path| std::path::Path::new(path).exists())
                    .map(|&s| s.to_string())
                    .unwrap_or_else(|| "sh".to_string())
            };
            (verified, vec!["--login".to_string()])
        };

        let (verified_shell, args) = if let Ok(shell_mod) = modules.load_shell(&shell_id) {
            let path = shell_mod
                .get_executable_path()
                .to_string_lossy()
                .to_string();
            let args = shell_mod.get_default_args().to_vec();
            if std::path::Path::new(&path).exists() {
                (path, args)
            } else {
                resolve_from_env()
            }
        } else {
            resolve_from_env()
        };

        let mut cmd = CommandBuilder::new(verified_shell);
        cmd.args(args);
        cmd.cwd(cwd);
        let child = pair.slave.spawn_command(cmd)?;
        let child_pid = child.process_id();

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let ipc = Arc::new(Mutex::new(None));
        let ipc_clone = ipc.clone();
        let grid = Arc::new(Mutex::new(super::grid::ScreenGrid::new(80, 24)));
        let grid_clone = grid.clone();
        let session_id = format!("{}:{}", sector_id, hub_id);
        let activity = Arc::new(PtyActivity {
            last_input_ms: std::sync::atomic::AtomicU64::new(0),
            last_output_ms: std::sync::atomic::AtomicU64::new(0),
            command_pending: std::sync::atomic::AtomicBool::new(false),
        });
        let activity_clone = activity.clone();
        let alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let alive_clone = alive.clone();

        let state_clone = state.clone();
        let ai_clone = ai.clone();
        let heuristic_clone = heuristic.clone();
        let sid_clone = sector_id;
        let hid_clone = hub_id;
        thread::spawn(move || {
            read_loop(
                reader,
                state_clone,
                ai_clone,
                heuristic_clone,
                sid_clone,
                hid_clone,
                ipc_clone,
                grid_clone,
                session_id,
                activity_clone,
            );
        });

        // Phase 1.6: watch for command completion so the directory
        // listing (and the chips fed by it) refreshes after every
        // command, in any shell.
        {
            let watcher_state = state.clone();
            let watcher_activity = activity.clone();
            thread::spawn(move || {
                completion_watcher(
                    watcher_state,
                    sector_id,
                    hub_id,
                    child_pid,
                    watcher_activity,
                    alive_clone,
                );
            });
        }

        Ok(Self {
            _state: state,
            _sector_id: sector_id,
            _hub_id: hub_id,
            writer,
            master: pair.master,
            _child: child,
            ipc,
            grid,
            activity,
            alive,
            input_line: super::input_line::InputLine::new(),
            chip_filter_sent: None,
        })
    }

    /// This session's identity. Grids, input routing, and display
    /// subscriptions are addressed by it, so a Brain holding more
    /// than one session is an addition, not a redesign.
    pub fn session_id(&self) -> String {
        format!("{}:{}", self._sector_id, self._hub_id)
    }

    /// The current display state of this session's grid.
    pub fn display_snapshot(&self) -> super::grid::GridSnapshot {
        self.grid.lock().unwrap().snapshot()
    }

    /// The grid's current sequence number (cheap; no snapshot).
    pub fn display_seq(&self) -> u64 {
        self.grid.lock().unwrap().seq()
    }

    /// Encode one raw key event against the grid's emulation
    /// state and write the resulting bytes through the one input
    /// funnel. An event that carries no terminal input (a bare
    /// modifier press) writes nothing.
    pub fn write_key_event(&mut self, ev: &super::keymap::KeyEvent) -> anyhow::Result<()> {
        let bytes = {
            let grid = self.grid.lock().unwrap();
            super::keymap::encode_key(ev, grid.mode())
        };
        match bytes {
            Some(bytes) => self.write(&bytes),
            None => Ok(()),
        }
    }

    /// Deliver literal text (committed input, a chip tap's
    /// typing), encoded by the Brain's text encoding — as
    /// typed input, byte for byte.
    pub fn write_text(&mut self, text: &str) -> anyhow::Result<()> {
        let bytes = {
            let grid = self.grid.lock().unwrap();
            super::keymap::encode_text(text, grid.mode())
        };
        self.write(&bytes)
    }

    /// Move this session's display window into its scrollback
    /// history (positive delta = older lines).
    pub fn grid_scroll(&self, delta_lines: i32) {
        self.grid.lock().unwrap().scroll(delta_lines);
    }

    pub fn set_ipc(&self, ipc: &Arc<crate::brain::ipc_handler::IpcHandler>) {
        *self.ipc.lock().unwrap() = Some(Arc::downgrade(ipc));
    }

    pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        trace_pty_bytes("in", &PTY_TRACE_IN, data);
        // Phase 1.6: a write containing Enter submits a command —
        // the completion watcher uses this (plus output quiescence)
        // to know when to re-issue the directory listing.
        if data.contains(&b'\r') || data.contains(&b'\n') {
            self.activity
                .last_input_ms
                .store(activity_now_ms(), std::sync::atomic::Ordering::SeqCst);
            self.activity
                .command_pending
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        self.writer.write_all(data)?;
        self.writer.flush()?;
        // Phase 1.7 (dynamic chips): model the shell's input line
        // from the same bytes, so the chip set can filter against
        // the word being typed. Only the derived filter is
        // published, and only when it changes.
        self.input_line.feed(data);
        let filter = self.input_line.fragment();
        if filter != self.chip_filter_sent {
            self.chip_filter_sent = filter.clone();
            let mut state = self._state.lock().unwrap();
            crate::brain::sector::SectorManager::set_chip_filter(
                &mut state,
                self._sector_id,
                self._hub_id,
                filter,
            );
        }
        Ok(())
    }

    pub fn resize(&self, rows: u16, cols: u16) -> anyhow::Result<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        self.grid
            .lock()
            .unwrap()
            .resize(cols as usize, rows as usize);
        Ok(())
    }

    pub fn send_signal(&mut self, signal: &str) -> anyhow::Result<()> {
        let sig = match signal {
            "INT" | "SIGINT" => libc::SIGINT,
            "TERM" | "SIGTERM" => libc::SIGTERM,
            "KILL" | "SIGKILL" => libc::SIGKILL,
            "QUIT" | "SIGQUIT" => libc::SIGQUIT,
            "HUP" | "SIGHUP" => libc::SIGHUP,
            _ => return Err(anyhow::anyhow!("Unsupported signal: {}", signal)),
        };

        // We attempt to signal the process group of the shell.
        // In portable-pty, the child process ID is often the PGID leader.
        if let Some(pid) = self._child.process_id() {
            unsafe {
                // Negative PID sends signal to the process group
                if libc::kill(-(pid as libc::pid_t), sig) != 0 {
                    // Fallback to signaling just the shell process if PGID kill fails
                    libc::kill(pid as libc::pid_t, sig);
                }
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Could not retrieve PID for signaling"))
        }
    }

    pub fn force_kill(&mut self) -> anyhow::Result<()> {
        self._child.kill()?;
        Ok(())
    }

    /// Run a command in an isolated PTY with a transient sandbox (§7.7).
    pub fn exec_isolated(command: &str, cwd: std::path::PathBuf) -> anyhow::Result<String> {
        let (output, sandbox) = Self::exec_sandboxed(command, cwd)?;
        let _ = sandbox.cleanup();
        Ok(output)
    }

    /// Run a command in a transient filesystem sandbox (§7.7).
    /// Returns the terminal output and the sandbox handle for later merge logic.
    pub fn exec_sandboxed(
        command: &str,
        cwd: std::path::PathBuf,
    ) -> anyhow::Result<(String, crate::modules::sandbox::OverlaySandbox)> {
        let sandbox = crate::modules::sandbox::SandboxManager::create_overlay_sandbox(&cwd)?;

        let lower = sandbox.lower.to_string_lossy();
        let upper = sandbox.upper.to_string_lossy();
        let work = sandbox.work.to_string_lossy();
        let merged = sandbox.merged.to_string_lossy();

        // Escape the command for bash -c (basic escaping)
        let escaped_command = command.replace('"', "\\\"");

        let sandbox_script = format!(
            "mount -t overlay overlay -o lowerdir='{}',upperdir='{}',workdir='{}' '{}' && cd '{}' && sh -c \"{}\"",
            lower, upper, work, merged, merged, escaped_command
        );

        let mut cmd = CommandBuilder::new("unshare");
        cmd.args(["-m", "-r", "bash", "-c", &sandbox_script]);

        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let _child = pair.slave.spawn_command(cmd)?;

        let mut reader = pair.master.try_clone_reader()?;
        let mut output = String::new();
        let mut buffer = [0u8; 1024];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    output.push_str(&String::from_utf8_lossy(&buffer[..n]));
                }
                Err(_) => break,
            }
            if output.len() > 1_000_000 {
                break;
            } // 1MB safety cap
        }

        Ok((output, sandbox))
    }
}

fn read_loop(
    mut reader: Box<dyn Read + Send>,
    state: Arc<Mutex<TosState>>,
    ai: Arc<crate::services::AiService>,
    _heuristic: Arc<crate::services::HeuristicService>,
    sector_id: uuid::Uuid,
    hub_id: uuid::Uuid,
    ipc_weak_container: Arc<Mutex<Option<std::sync::Weak<crate::brain::ipc_handler::IpcHandler>>>>,
    grid: Arc<Mutex<super::grid::ScreenGrid>>,
    session_id: String,
    activity: Arc<PtyActivity>,
) {
    let mut osc_parser = OscParser::new();
    let mut line_buffer = String::new();
    let mut buffer = [0u8; 4096];

    loop {
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                let data = &buffer[..n];
                trace_pty_bytes("out", &PTY_TRACE_OUT, data);

                activity
                    .last_output_ms
                    .store(activity_now_ms(), std::sync::atomic::Ordering::SeqCst);

                // Parse the chunk into the session's grid — the one
                // emulation of this terminal — and let the IPC layer
                // publish the new display state to faces. The raw
                // stream itself goes no further than this loop.
                grid.lock().unwrap().feed(data);
                if let Some(ipc_weak) = &*ipc_weak_container.lock().unwrap() {
                    if let Some(ipc) = ipc_weak.upgrade() {
                        ipc.publish_display(&session_id);
                    }
                }

                let text = String::from_utf8_lossy(data);
                line_buffer.push_str(&text);

                while let Some(pos) = line_buffer.find('\n') {
                    let mut line = line_buffer.drain(..=pos).collect::<String>();
                    line = line.trim_end_matches(['\r', '\n']).to_string();

                    let (clean_text, events) = osc_parser.process(&line);
                    tracing::debug!("[PTY READ] Line: {:?}, Events: {}, Clean: {:?}", line, events.len(), clean_text);

                    let mut state_lock = state.lock().unwrap();
                    let mut line_priority_override: Option<u8> = None;
                    for event in events {
                        match event {
                            OscEvent::Priority(p) => osc_parser.current_priority = p,
                            OscEvent::LinePriority(p) => {
                                // §27.4: Override priority for this line only.
                                line_priority_override = Some(p);
                            }
                            OscEvent::Cwd(path) => {
                                let path_buf = std::path::PathBuf::from(&path);
                                if let Some(sector) =
                                    state_lock.sectors.iter_mut().find(|s| s.id == sector_id)
                                {
                                    if let Some(hub) =
                                        sector.hubs.iter_mut().find(|h| h.id == hub_id)
                                    {
                                        hub.current_directory = path_buf.clone();
                                    }

                                    // §31.3: Dynamic sector labeling from cwd changes.
                                    // Only auto-relabel sectors with default/auto-generated names
                                    // to avoid overwriting explicit user renames.
                                    let auto_labels = [
                                        "Primary", "New Sector", "Detached", "Untitled",
                                    ];
                                    let is_auto_name = auto_labels
                                        .iter()
                                        .any(|prefix| sector.name == *prefix || sector.name.starts_with("Detached"));
                                    if is_auto_name {
                                        let home = dirs::home_dir();
                                        let label = if home.as_ref() == Some(&path_buf) {
                                            "~".to_string()
                                        } else {
                                            path_buf
                                                .file_name()
                                                .map(|n| n.to_string_lossy().to_string())
                                                .unwrap_or_else(|| "/".to_string())
                                        };
                                        sector.name = label;
                                    }

                                    // §4.7: Automatic skill activation from CWD signals
                                    ai.check_context_signals(&mut state_lock, &path_buf);
                                }

                                // Phase 1.6: the accelerator chips track the
                                // shell's directory — refresh the listing
                                // for the new cwd.
                                crate::brain::sector::SectorManager::refresh_listing_for(
                                    &mut state_lock,
                                    sector_id,
                                    hub_id,
                                );
                            }
                            OscEvent::DirectoryListing(mut listing) => {
                                if let Some(sector) =
                                    state_lock.sectors.iter_mut().find(|s| s.id == sector_id)
                                {
                                    if let Some(hub) =
                                        sector.hubs.iter_mut().find(|h| h.id == hub_id)
                                    {
                                        // A pushed listing is a new chip
                                        // issuance too — stamp the next
                                        // generation (Phase 1.6).
                                        listing.generation = hub
                                            .shell_listing
                                            .as_ref()
                                            .map_or(1, |l| l.generation + 1);
                                        hub.shell_listing = Some(listing);
                                    }
                                }
                            }
                            OscEvent::CommandResult {
                                command: _,
                                status,
                                output: _,
                            } => {
                                if let Some(sector) =
                                    state_lock.sectors.iter_mut().find(|s| s.id == sector_id)
                                {
                                    if let Some(hub) =
                                        sector.hubs.iter_mut().find(|h| h.id == hub_id)
                                    {
                                        hub.last_exit_status = Some(status);
                                        hub.is_running = false;
                                    }
                                }

                                // Phase 1.6: a completed command may have
                                // changed the directory's contents — keep
                                // the accelerator chips' listing fresh.
                                crate::brain::sector::SectorManager::refresh_listing_for(
                                    &mut state_lock,
                                    sector_id,
                                    hub_id,
                                );
                            }
                            OscEvent::JsonContext(json) => {
                                if let Some(sector) =
                                    state_lock.sectors.iter_mut().find(|s| s.id == sector_id)
                                {
                                    if let Some(hub) =
                                        sector.hubs.iter_mut().find(|h| h.id == hub_id)
                                    {
                                        hub.json_context = Some(json);
                                    }
                                }
                            }
                        }
                    }

                    // Use line-specific priority override if present, else the persistent parser priority.
                    let mut effective_priority = line_priority_override.unwrap_or(osc_parser.current_priority);

                    // §31.4: Automatic error detection (PTY error highlighting)
                    if line_priority_override.is_none() {
                        let text_lower = clean_text.to_lowercase();
                        if text_lower.contains("error:") || text_lower.contains("failed") || text_lower.contains("command not found") {
                            effective_priority = 3; // HIGH priority
                        }
                    }

                    if !clean_text.is_empty() {
                        if let Some(sector) =
                            state_lock.sectors.iter_mut().find(|s| s.id == sector_id)
                        {
                            if let Some(hub) = sector.hubs.iter_mut().find(|h| h.id == hub_id) {
                                hub.terminal_output.push(TerminalLine {
                                    text: clean_text.to_string(),
                                    priority: effective_priority,
                                    timestamp: Local::now(),
                                });
                                if hub.terminal_output.len() > hub.buffer_limit {
                                    hub.terminal_output.remove(0);
                                }
                                hub.version += 1;
                                state_lock.version += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrollback_round_trips_appended_bytes() {
        let mut buf = ScrollbackBuffer::new(1024);
        assert!(buf.is_empty());
        buf.append(b"hello ");
        buf.append(b"world\n");
        assert_eq!(buf.snapshot(), b"hello world\n");
        assert_eq!(buf.len(), 12);
    }

    #[test]
    fn scrollback_trims_to_cap_keeping_the_tail() {
        let mut buf = ScrollbackBuffer::new(8);
        buf.append(b"0123456789");
        assert!(buf.len() <= 8);
        assert_eq!(buf.snapshot(), b"23456789");
    }

    #[test]
    fn scrollback_trim_prefers_a_newline_boundary() {
        let mut buf = ScrollbackBuffer::new(10);
        // 16 bytes total; cap 10 -> excess 6; the first newline at or
        // after index 6 is at index 10, so the cut lands just past it
        // and the replay starts at a line boundary.
        buf.append(b"aaaa\nbbbbb\nccccc");
        assert_eq!(buf.snapshot(), b"ccccc");
    }

    #[test]
    fn trace_escaping_shows_controls_and_hides_raw_bytes() {
        assert_eq!(
            escape_trace_bytes(b"cd '/home/tim'\r\n"),
            "cd '/home/tim'\\r\\n"
        );
        assert_eq!(escape_trace_bytes(b"a\tb\x1b[0m"), "a\\tb\\x1b[0m");
        assert_eq!(escape_trace_bytes(b""), "");
    }

    #[test]
    fn scrollback_handles_chunks_larger_than_cap() {
        let mut buf = ScrollbackBuffer::new(4);
        buf.append(b"abcdefghij");
        assert_eq!(buf.snapshot(), b"ghij");
    }
}
