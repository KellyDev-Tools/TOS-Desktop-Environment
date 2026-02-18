//! Shell API Module
//! 
//! Enables bi-directional communication between the Command Hub and the underlying shell
//! via OSC (Operating System Command) escape sequences and special commands.
//! 
//! ## Shell-to-Compositor (OSC)
//! - `suggestions`: Provides command suggestions based on context
//! - `directory`: Sends directory listing
//! - `command_result`: Reports exit status and output preview
//! - `cwd`: Informs of current working directory
//! - `env`: Environment variable updates
//! - `dangerous_command`: Flags a dangerous command for confirmation
//! 
//! ## Compositor-to-Shell (via PTY)
//! - `EXEC <command>`: Execute a command
//! - `CD <path>`: Change directory
//! - `COMPLETE <partial>`: Request completions
//! - `LS <path>`: Request directory listing
//! - `SETENV <var=value>`: Set environment variable
//! 
//! ## Reference Implementation
//! Fish shell is the reference shell, with a built-in plugin.
//! Bash/Zsh plugins can be implemented using PROMPT_COMMAND and preexec hooks.

use crate::{TosState, CommandHubMode, DirectoryListing, DirectoryEntry, EntryType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// OSC sequence types for shell-to-compositor communication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OscSequence {
    /// Command suggestions
    Suggestions(Vec<CommandSuggestion>),
    /// Directory listing
    Directory(DirectoryListing),
    /// Command execution result
    CommandResult {
        command: String,
        exit_status: i32,
        output_preview: Option<String>,
    },
    /// Current working directory update
    Cwd(String),
    /// Environment variable update
    Env {
        key: String,
        value: String,
    },
    /// Dangerous command warning
    DangerousCommand {
        command: String,
        risk_level: RiskLevel,
    },
    /// Shell ready notification
    ShellReady {
        shell_type: String,
        version: String,
    },
    /// Request completion from compositor
    RequestCompletion {
        partial: String,
        cursor_pos: usize,
    },
    /// Request sector/system context metadata
    ContextRequest {
        hub_id: Uuid,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInfo {
    pub sector_name: String,
    pub sector_type: String,
    pub active_modules: Vec<String>,
    pub current_time: String,
    pub node_id: Uuid,
}

/// Risk level for dangerous commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk - informational only
    Low,
    /// Medium risk - requires confirmation
    Medium,
    /// High risk - requires tactile confirmation
    High,
    /// Critical risk - requires multi-factor confirmation
    Critical,
}

/// Command suggestion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandSuggestion {
    /// Display text
    pub text: String,
    /// Full command to insert
    pub command: String,
    /// Description/help text
    pub description: String,
    /// Icon or category
    pub category: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}


/// Compositor-to-shell command
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellCommand {
    /// Execute a command
    Exec(String),
    /// Change directory
    Cd(String),
    /// Request completions
    Complete { partial: String, cursor_pos: usize },
    /// List directory
    Ls(String),
    /// Set environment variable
    SetEnv { key: String, value: String },
    /// Clear screen
    Clear,
    /// Interrupt current command (SIGINT)
    Interrupt,
    /// Send EOF
    Eof,
}

/// Shell API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellApiConfig {
    /// Enable OSC parsing
    pub enable_osc: bool,
    /// Enable compositor-to-shell commands
    pub enable_commands: bool,
    /// Shell type (fish, bash, zsh)
    pub shell_type: String,
    /// Maximum suggestion count
    pub max_suggestions: usize,
    /// Maximum directory entries
    pub max_dir_entries: usize,
    /// Auto-execute safe commands
    pub auto_execute_safe: bool,
    /// Dangerous command patterns
    pub dangerous_patterns: Vec<DangerousPattern>,
}

impl Default for ShellApiConfig {
    fn default() -> Self {
        Self {
            enable_osc: true,
            enable_commands: true,
            shell_type: "fish".to_string(),
            max_suggestions: 10,
            max_dir_entries: 1000,
            auto_execute_safe: true,
            dangerous_patterns: Self::default_dangerous_patterns(),
        }
    }
}

impl ShellApiConfig {
    /// Default dangerous command patterns
    fn default_dangerous_patterns() -> Vec<DangerousPattern> {
        vec![
            DangerousPattern {
                pattern: r"rm\s+-rf\s+/".to_string(),
                risk_level: RiskLevel::Critical,
                message: "This will delete the entire filesystem!".to_string(),
            },
            DangerousPattern {
                pattern: r"dd\s+if=.*\s+of=/dev/".to_string(),
                risk_level: RiskLevel::Critical,
                message: "This will overwrite a device directly!".to_string(),
            },
            DangerousPattern {
                pattern: r"mkfs\.".to_string(),
                risk_level: RiskLevel::Critical,
                message: "This will format a filesystem".to_string(),
            },
            DangerousPattern {
                pattern: r">?\s*/dev/".to_string(),
                risk_level: RiskLevel::High,
                message: "Writing to device files can damage the system".to_string(),
            },
            DangerousPattern {
                pattern: r"chmod\s+-R\s+777\s+/".to_string(),
                risk_level: RiskLevel::High,
                message: "This will make all files world-writable".to_string(),
            },
            DangerousPattern {
                pattern: r":\s*\(\s*\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:".to_string(),
                risk_level: RiskLevel::High,
                message: "This is a fork bomb that will crash the system".to_string(),
            },
            DangerousPattern {
                pattern: r"wget\s+.*\s*\|\s*(ba)?sh".to_string(),
                risk_level: RiskLevel::Medium,
                message: "Piping downloaded content directly to shell is dangerous".to_string(),
            },
            DangerousPattern {
                pattern: r"curl\s+.*\s*\|\s*(ba)?sh".to_string(),
                risk_level: RiskLevel::Medium,
                message: "Piping downloaded content directly to shell is dangerous".to_string(),
            },
        ]
    }
}

/// Dangerous command pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerousPattern {
    /// Regex pattern
    pub pattern: String,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Warning message
    pub message: String,
}

/// OSC Parser for shell escape sequences
#[derive(Debug)]
pub struct OscParser {
    /// Buffer for incomplete sequences
    pub buffer: String,
    /// Configuration
    pub config: ShellApiConfig,
}

impl OscParser {
    /// Create a new OSC parser
    pub fn new() -> Self {
        Self::with_config(ShellApiConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: ShellApiConfig) -> Self {
        Self {
            buffer: String::new(),
            config,
        }
    }

    /// Parse input for OSC sequences
    /// Returns parsed sequences and remaining text
    pub fn parse(&mut self, input: &str) -> (Vec<OscSequence>, String) {
        let mut sequences = Vec::new();
        let mut output = String::new();
        
        // OSC sequences use ESC ] (0x1B 0x5D) ... BEL (0x07) or ST (0x1B 0x5C)
        // Format: ESC ] <Ps> ; <Pt> BEL
        // Where Ps is the parameter (e.g., 133 for shell integration)
        
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Potential escape sequence
                if let Some(next) = chars.peek() {
                    if *next == ']' {
                        // OSC sequence start
                        chars.next(); // consume ']'
                        let seq = self.parse_osc_sequence(&mut chars);
                        if let Some(osc) = self.interpret_osc(&seq) {
                            sequences.push(osc);
                        }
                    } else {
                        output.push(ch);
                        output.push(*next);
                        chars.next();
                    }
                } else {
                    output.push(ch);
                }
            } else {
                output.push(ch);
            }
        }
        
        (sequences, output)
    }

    /// Parse an OSC sequence from character iterator
    fn parse_osc_sequence<I>(&self, chars: &mut std::iter::Peekable<I>) -> String
    where
        I: Iterator<Item = char>,
    {
        let mut seq = String::new();
        
        while let Some(ch) = chars.next() {
            if ch == '\x07' {
                // BEL - end of sequence
                break;
            } else if ch == '\x1b' {
                if let Some(next) = chars.peek() {
                    if *next == '\\' {
                        // ST - end of sequence
                        chars.next();
                        break;
                    } else {
                        seq.push(ch);
                    }
                } else {
                    seq.push(ch);
                }
            } else {
                seq.push(ch);
            }
        }
        
        seq
    }

    /// Interpret an OSC sequence
    fn interpret_osc(&self, seq: &str) -> Option<OscSequence> {
        // Parse parameter and text
        let parts: Vec<&str> = seq.splitn(2, ';').collect();
        if parts.is_empty() {
            return None;
        }

        let ps: u32 = parts[0].parse().ok()?;
        let pt = parts.get(1).map(|s| s.to_string()).unwrap_or_default();

        match ps {
            // TOS-specific OSC codes (using private range 9000-9099)
            9000 => {
                // Suggestions
                let suggestions = self.parse_suggestions(&pt);
                Some(OscSequence::Suggestions(suggestions))
            }
            9001 => {
                // Directory listing
                let listing = self.parse_directory(&pt)?;
                Some(OscSequence::Directory(listing))
            }
            9002 => {
                // Command result
                let result = self.parse_command_result(&pt)?;
                Some(OscSequence::CommandResult {
                    command: result.0,
                    exit_status: result.1,
                    output_preview: result.2,
                })
            }
            9003 | 7 => {
                // CWD update
                let mut path = pt;
                if path.starts_with("file://") {
                    // Strip file://hostname/ prefix
                    if let Some(slash_idx) = path[7..].find('/') {
                        path = path[7+slash_idx..].to_string();
                    }
                }
                Some(OscSequence::Cwd(path))
            }
            9004 => {
                // Environment variable
                let parts: Vec<&str> = pt.splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some(OscSequence::Env {
                        key: parts[0].to_string(),
                        value: parts[1].to_string(),
                    })
                } else {
                    None
                }
            }
            9005 => {
                // Dangerous command
                let parts: Vec<&str> = pt.splitn(2, ';').collect();
                if parts.len() == 2 {
                    let risk = match parts[0] {
                        "low" => RiskLevel::Low,
                        "medium" => RiskLevel::Medium,
                        "high" => RiskLevel::High,
                        "critical" => RiskLevel::Critical,
                        _ => RiskLevel::Medium,
                    };
                    Some(OscSequence::DangerousCommand {
                        command: parts[1].to_string(),
                        risk_level: risk,
                    })
                } else {
                    None
                }
            }
            9006 => {
                // Shell ready
                let parts: Vec<&str> = pt.splitn(2, ';').collect();
                Some(OscSequence::ShellReady {
                    shell_type: parts.get(0).unwrap_or(&"unknown").to_string(),
                    version: parts.get(1).unwrap_or(&"unknown").to_string(),
                })
            }
            9007 => {
                // Request completion
                let parts: Vec<&str> = pt.splitn(2, ';').collect();
                if parts.len() == 2 {
                    let cursor_pos = parts[1].parse().unwrap_or(0);
                    Some(OscSequence::RequestCompletion {
                        partial: parts[0].to_string(),
                        cursor_pos,
                    })
                } else {
                    None
                }
            }
            9008 => {
                // Explicit completion response from shell (Pt = text;command;desc;cat|...)
                let suggestions = self.parse_suggestions(&pt);
                Some(OscSequence::Suggestions(suggestions))
            }
            9010 => {
                // Context request
                let hub_id = Uuid::parse_str(&pt).ok()?;
                Some(OscSequence::ContextRequest { hub_id })
            }
            _ => None,
        }
    }

    /// Parse suggestions from OSC text
    fn parse_suggestions(&self, text: &str) -> Vec<CommandSuggestion> {
        text.split('|')
            .filter_map(|s| {
                let parts: Vec<&str> = s.splitn(4, ';').collect();
                if parts.len() >= 2 {
                    Some(CommandSuggestion {
                        text: parts[0].to_string(),
                        command: parts.get(1).unwrap_or(&parts[0]).to_string(),
                        description: parts.get(2).unwrap_or(&"").to_string(),
                        category: parts.get(3).unwrap_or(&"general").to_string(),
                        confidence: 0.8,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Parse directory listing from OSC text
    fn parse_directory(&self, text: &str) -> Option<DirectoryListing> {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return None;
        }

        // First line: path;parent;total;hidden;selected
        let header: Vec<&str> = lines[0].split(';').collect();
        if header.len() < 2 {
            return None;
        }

        let path = header[0].to_string();
        let parent = if header[1].is_empty() { None } else { Some(header[1].to_string()) };
        let total_count = header.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        let hidden_count = header.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
        let selected_count = header.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);

        // Remaining lines: entries
        let entries: Vec<DirectoryEntry> = lines.iter().skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.split(';').collect();
                if parts.len() >= 4 {
                    Some(DirectoryEntry {
                        name: parts[0].to_string(),
                        entry_type: match parts[1] {
                            "f" => EntryType::File,
                            "d" => EntryType::Directory,
                            "l" => EntryType::Symlink,
                            _ => EntryType::Unknown,
                        },
                        size: parts[2].parse().unwrap_or(0),
                        permissions: parts.get(3).unwrap_or(&"???").to_string(),
                        modified: parts.get(4).unwrap_or(&"").to_string(),
                        is_hidden: parts[0].starts_with('.'),
                        is_selected: parts.get(5).map(|s| *s == "1").unwrap_or(false),
                    })
                } else {
                    None
                }
            })
            .collect();

        Some(DirectoryListing {
            path,
            parent,
            entries,
            total_count,
            hidden_count,
            selected_count,
        })
    }

    /// Parse command result from OSC text
    fn parse_command_result(&self, text: &str) -> Option<(String, i32, Option<String>)> {
        let parts: Vec<&str> = text.splitn(3, ';').collect();
        if parts.len() >= 2 {
            let command = parts[0].to_string();
            let exit_status = parts[1].parse().ok()?;
            let output_preview = parts.get(2).map(|s| s.to_string());
            Some((command, exit_status, output_preview))
        } else {
            None
        }
    }

    /// Check if a command matches dangerous patterns
    pub fn check_dangerous(&self, command: &str) -> Option<(RiskLevel, String)> {
        for pattern in &self.config.dangerous_patterns {
            if let Ok(regex) = regex::Regex::new(&pattern.pattern) {
                if regex.is_match(command) {
                    return Some((pattern.risk_level, pattern.message.clone()));
                }
            }
        }
        None
    }

    /// Format a shell command for sending to PTY
    pub fn format_shell_command(&self, cmd: &ShellCommand) -> String {
        match cmd {
            ShellCommand::Exec(command) => format!("{}\n", command),
            ShellCommand::Cd(path) => format!("cd {}\n", shell_escape(path)),
            ShellCommand::Complete { partial, cursor_pos } => {
                format!("\x1b]9008;{};{}\x07", partial, cursor_pos)
            }
            ShellCommand::Ls(path) => format!("ls -la {}\n", shell_escape(path)),
            ShellCommand::SetEnv { key, value } => format!("export {}={}\n", key, shell_escape(value)),
            ShellCommand::Clear => "clear\n".to_string(),
            ShellCommand::Interrupt => "\x03".to_string(), // Ctrl+C
            ShellCommand::Eof => "\x04".to_string(), // Ctrl+D
        }
    }

    /// Generate Fish shell integration script
    pub fn generate_fish_integration(&self) -> String {
        r#"# TOS Shell Integration for Fish
# Add this to your config.fish or source it

function __tos_send_osc
    printf "\033]9003;%s\007" $PWD
end

function __tos_preexec --on-event fish_preexec
    # Send command about to execute
    printf "\033]9002;%s;running\007" $argv[1]
end

function __tos_postexec --on-event fish_postexec
    # Send command result
    set -l status $status
    printf "\033]9002;%s;%d\007" (history --max=1) $status
end

# Send CWD on directory change
function __tos_cwd --on-variable PWD
    __tos_send_osc
end

# Send directory listing on request
function __tos_ls
    set -l path $argv[1]
    set -l entries (ls -la $path 2>/dev/null | tail -n +2)
    set -l total (count $entries)
    set -l hidden (count (string match -r '^\\.' -- $entries))
    
    printf "\033]9001;%s;%s;%d;%d;0\007" $path (dirname $path) $total $hidden
    
    for entry in $entries
        set -l parts (string split -m 1 ' ' -- $entry)
        # Format: name;type;size;perms;modified;selected
        printf "\033]9001;%s;%s;%s;%s;%s;0\007" \
            $parts[2] \
            (if test -d $entry; echo d; else; echo f; end) \
            0 \
            $parts[1] \
            ""
    end
end

# Send shell ready notification
printf "\033]9006;fish;%s\007" $FISH_VERSION

# Hook into fish
__tos_send_osc
"#.to_string()
    }

    /// Generate Bash shell integration script
    pub fn generate_bash_integration(&self) -> String {
        r#"# TOS Shell Integration for Bash
# Add this to your .bashrc or source it

__tos_send_osc() {
    printf "\033]9003;%s\007" "$PWD"
}

__tos_preexec() {
    printf "\033]9002;%s;running\007" "$1"
}

__tos_postexec() {
    local status=$?
    printf "\033]9002;%s;%d\007" "$(history 1 | sed 's/^[ ]*[0-9]*[ ]*//')" $status
}

# Hook into bash
export PROMPT_COMMAND='__tos_send_osc'

# Trap DEBUG for preexec
trap '__tos_preexec "$BASH_COMMAND"' DEBUG

# Postexec via PROMPT_COMMAND
__tos_original_prompt_command="$PROMPT_COMMAND"
PROMPT_COMMAND='__tos_postexec; __tos_original_prompt_command'

# Send shell ready
printf "\033]9006;bash;%s\007" "${BASH_VERSION}"
"#.to_string()
    }
}

/// Escape a string for shell usage
pub fn shell_escape(s: &str) -> String {
    // Simple escaping - wrap in quotes and escape internal quotes
    if s.contains(' ') || s.contains('\\') || s.contains('\"') || s.contains('\'') {
        format!("'{}'", s.replace('\'', "'\\\"'\\\"'"))
    } else {
        s.to_string()
    }
}

/// Shell API manager for integration with TosState
#[derive(Debug)]
pub struct ShellApi {
    /// OSC parser
    pub parser: OscParser,
    /// Pending completions
    pub pending_completions: HashMap<String, Vec<CommandSuggestion>>,
}

impl Default for ShellApi {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellApi {
    /// Create a new shell API
    pub fn new() -> Self {
        Self {
            parser: OscParser::new(),
            pending_completions: HashMap::new(),
        }
    }

    /// Process OSC sequences from shell output
    pub fn process_output(&mut self, output: &str, state: &mut TosState) -> String {
        let (sequences, clean_output) = self.parser.parse(output);
        
        for seq in sequences {
            self.handle_sequence(seq, state);
        }
        
        clean_output
    }

    /// Handle an OSC sequence
    fn handle_sequence(&mut self, seq: OscSequence, state: &mut TosState) {
        match seq {
            OscSequence::Suggestions(suggestions) => {
                // Update command hub with suggestions
                let viewport = &state.viewports[state.active_viewport_index];
                let sector = &mut state.sectors[viewport.sector_index];
                let _hub = &mut sector.hubs[viewport.hub_index];
                
                // Store suggestions for display
                // In real implementation, would update UI
                tracing::info!("Received {} command suggestions", suggestions.len());
            }
            OscSequence::Directory(listing) => {
                // Update directory mode with listing
                let viewport = &state.viewports[state.active_viewport_index];
                let sector = &mut state.sectors[viewport.sector_index];
                let hub = &mut sector.hubs[viewport.hub_index];
                
                // Store listing in hub for UI to pick up
                hub.shell_listing = Some(listing.clone());
                
                // Log it
                tracing::info!("Received directory listing: {} entries", listing.entries.len());
            }
            OscSequence::CommandResult { command, exit_status, output_preview } => {
                // Update terminal output
                let viewport = &state.viewports[state.active_viewport_index];
                let sector = &mut state.sectors[viewport.sector_index];
                let hub = &mut sector.hubs[viewport.hub_index];
                
                let status_str = if exit_status == 0 {
                    "✓".to_string()
                } else {
                    format!("✗ ({})", exit_status)
                };
                
                hub.terminal_output.push(format!(
                    "[{}] {} {}",
                    status_str,
                    command,
                    output_preview.map(|s| format!("| {}", s)).unwrap_or_default()
                ));
                
                // Keep only last 100 lines
                if hub.terminal_output.len() > 100 {
                    hub.terminal_output.remove(0);
                }
            }
            OscSequence::Cwd(path) => {
                let viewport = &state.viewports[state.active_viewport_index];
                let sector = &mut state.sectors[viewport.sector_index];
                let hub = &mut sector.hubs[viewport.hub_index];
                hub.current_directory = std::path::PathBuf::from(path);
            }
            OscSequence::Env { key, value } => {
                // Update environment
                tracing::debug!("Shell env: {}={}", key, value);
            }
            OscSequence::DangerousCommand { command, risk_level } => {
                // Trigger dangerous command handling
                let viewport = &state.viewports[state.active_viewport_index];
                let sector = &mut state.sectors[viewport.sector_index];
                let hub = &mut sector.hubs[viewport.hub_index];
                
                hub.confirmation_required = Some(format!(
                    "DANGEROUS: {} (Risk: {:?})",
                    command, risk_level
                ));
            }
            OscSequence::ShellReady { shell_type, version } => {
                tracing::info!("Shell ready: {} v{}", shell_type, version);
            }
            OscSequence::RequestCompletion { partial, cursor_pos } => {
                // Generate completions
                let completions = self.generate_completions(&partial, cursor_pos, state);
                self.pending_completions.insert(partial, completions);
            }
            OscSequence::ContextRequest { hub_id: _ } => {
                // In a real implementation, we would send back a ContextInfo packet to the shell
                // This would be done via PTY: format!("\x1b]9011;{}\x07", serde_json::to_string(&info))
                tracing::info!("Shell requested context metadata");
            }
        }
    }

    /// Generate command completions
    fn generate_completions(&self, partial: &str, _cursor_pos: usize, state: &TosState) -> Vec<CommandSuggestion> {
        let mut suggestions = Vec::new();
        
        let viewport = &state.viewports[state.active_viewport_index];
        let sector = &state.sectors[viewport.sector_index];
        let hub = &sector.hubs[viewport.hub_index];
        
        // 1. Sector-specific completions
        if sector.name.contains("Stellar") || sector.name.contains("Science") {
             suggestions.push(CommandSuggestion {
                text: "scan ".to_string(),
                command: "scan ".to_string(),
                description: "Initiate sensor scan".to_string(),
                category: "science".to_string(),
                confidence: 0.95,
            });
        }

        // 2. Mode-specific completions
        match hub.mode {
            CommandHubMode::Command => {
                let builtins = vec![
                    ("cd ", "Change directory"),
                    ("ls ", "List directory"),
                    ("rm ", "Remove file"),
                    ("mkdir ", "Create directory"),
                    ("cat ", "View file"),
                    ("grep ", "Search in files"),
                ];

                for (cmd, desc) in builtins {
                    if cmd.starts_with(partial) {
                        suggestions.push(CommandSuggestion {
                            text: cmd.to_string(),
                            command: cmd.to_string(),
                            description: desc.to_string(),
                            category: "builtin".to_string(),
                            confidence: 0.9,
                        });
                    }
                }
            }
            CommandHubMode::Directory => {
                // Add file suggestions from the hub's applications (mocking filesystem)
                for app in &hub.applications {
                    if app.title.to_lowercase().contains(&partial.to_lowercase()) {
                        suggestions.push(CommandSuggestion {
                            text: app.title.clone(),
                            command: app.app_class.clone(),
                            description: format!("Launch {}", app.title),
                            category: "application".to_string(),
                            confidence: 0.85,
                        });
                    }
                }
            }
            CommandHubMode::Activity => {
                suggestions.push(CommandSuggestion {
                    text: "ps ".to_string(),
                    command: "ps ".to_string(),
                    description: "List processes".to_string(),
                    category: "activity".to_string(),
                    confidence: 0.8,
                });
            }
        }

        // 3. Security context completions
        if state.security.is_tactical_lockdown() {
            suggestions.push(CommandSuggestion {
                text: "unlock ".to_string(),
                command: "unlock ".to_string(),
                description: "Authorize system release".to_string(),
                category: "security".to_string(),
                confidence: 1.0,
            });
        }
        
        // 4. Filesystem completions (§13.3)
        let search_term = if partial.contains(' ') {
            partial.split_whitespace().last().unwrap_or("")
        } else {
            partial
        };

        if let Ok(entries) = std::fs::read_dir(&hub.current_directory) {
            for entry in entries.flatten() {
                 let name = entry.file_name().to_string_lossy().to_string();
                 if name.to_lowercase().starts_with(&search_term.to_lowercase()) {
                     let is_dir = entry.path().is_dir();
                     let suffix = if is_dir { "/" } else { "" };
                     suggestions.push(CommandSuggestion {
                         text: format!("{}{}", name, suffix),
                         command: format!("{}{}", name, suffix),
                         description: if is_dir { "Directory".to_string() } else { "File".to_string() },
                         category: "path".to_string(),
                         confidence: 0.9,
                     });
                 }
            }
        }
        
        suggestions
    }

    /// Send a command to the shell
    pub fn send_command(&self, cmd: ShellCommand) -> String {
        self.parser.format_shell_command(&cmd)
    }

    /// Check if a command is dangerous
    pub fn check_command(&self, command: &str) -> Option<(RiskLevel, String)> {
        self.parser.check_dangerous(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_api_config_default() {
        let config = ShellApiConfig::default();
        assert!(config.enable_osc);
        assert!(config.enable_commands);
        assert_eq!(config.shell_type, "fish");
        assert!(!config.dangerous_patterns.is_empty());
    }

    #[test]
    fn test_osc_parser_new() {
        let parser = OscParser::new();
        assert!(parser.buffer.is_empty());
    }

    #[test]
    fn test_parse_osc_suggestions() {
        let mut parser = OscParser::new();
        let input = "\x1b]9000;ls;ls -la;List all files;files|cd;cd;Change directory;nav\x07";
        
        let (sequences, _remaining) = parser.parse(input);
        
        assert_eq!(sequences.len(), 1);
        assert!(_remaining.is_empty());
        
        if let OscSequence::Suggestions(suggestions) = &sequences[0] {
            assert_eq!(suggestions.len(), 2);
            assert_eq!(suggestions[0].text, "ls");
            assert_eq!(suggestions[1].text, "cd");
        } else {
            panic!("Expected Suggestions sequence");
        }
    }

    #[test]
    fn test_parse_osc_cwd() {
        let mut parser = OscParser::new();
        let input = "\x1b]9003;/home/user/projects\x07";
        
        let (sequences, _remaining) = parser.parse(input);
        
        assert_eq!(sequences.len(), 1);
        assert!(matches!(sequences[0], OscSequence::Cwd(ref path) if path == "/home/user/projects"));
    }

    #[test]
    fn test_parse_osc_command_result() {
        let mut parser = OscParser::new();
        let input = "\x1b]9002;ls -la;0;total 128\x07";
        
        let (sequences, _remaining) = parser.parse(input);
        
        assert_eq!(sequences.len(), 1);
        if let OscSequence::CommandResult { command, exit_status, output_preview } = &sequences[0] {
            assert_eq!(command, "ls -la");
            assert_eq!(*exit_status, 0);
            assert_eq!(output_preview.as_ref().unwrap(), "total 128");
        } else {
            panic!("Expected CommandResult sequence");
        }
    }

    #[test]
    fn test_parse_osc_dangerous_command() {
        let mut parser = OscParser::new();
        let input = "\x1b]9005;high;rm -rf /\x07";
        
        let (sequences, _) = parser.parse(input);
        
        if let OscSequence::DangerousCommand { command, risk_level } = &sequences[0] {
            assert_eq!(command, "rm -rf /");
            assert_eq!(*risk_level, RiskLevel::High);
        } else {
            panic!("Expected DangerousCommand sequence");
        }
    }

    #[test]
    fn test_check_dangerous() {
        let parser = OscParser::new();
        
        let result = parser.check_dangerous("rm -rf /");
        assert!(result.is_some());
        let (risk, _) = result.unwrap();
        assert_eq!(risk, RiskLevel::Critical);
        
        let result = parser.check_dangerous("ls -la");
        assert!(result.is_none());
    }

    #[test]
    fn test_format_shell_command() {
        let parser = OscParser::new();
        
        let cmd = ShellCommand::Exec("ls -la".to_string());
        assert_eq!(parser.format_shell_command(&cmd), "ls -la\n");
        
        let cmd = ShellCommand::Cd("/home/user".to_string());
        assert_eq!(parser.format_shell_command(&cmd), "cd /home/user\n");
        
        let cmd = ShellCommand::Clear;
        assert_eq!(parser.format_shell_command(&cmd), "clear\n");
    }

    #[test]
    fn test_shell_escape() {
        assert_eq!(shell_escape("simple"), "simple");
        assert_eq!(shell_escape("with space"), "'with space'");
        assert_eq!(shell_escape("with'quote"), "'with'\\\"'\\\"'quote'");
    }

    #[test]
    fn test_generate_fish_integration() {
        let parser = OscParser::new();
        let script = parser.generate_fish_integration();
        
        assert!(script.contains("TOS Shell Integration for Fish"));
        assert!(script.contains("9003")); // CWD OSC
        assert!(script.contains("9002")); // Command result OSC
    }

    #[test]
    fn test_generate_bash_integration() {
        let parser = OscParser::new();
        let script = parser.generate_bash_integration();
        
        assert!(script.contains("TOS Shell Integration for Bash"));
        assert!(script.contains("PROMPT_COMMAND"));
    }

    #[test]
    fn test_shell_api_new() {
        let api = ShellApi::new();
        assert!(api.pending_completions.is_empty());
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn test_entry_type_variants() {
        let types = vec![
            EntryType::File,
            EntryType::Directory,
            EntryType::Symlink,
            EntryType::BlockDevice,
            EntryType::CharDevice,
            EntryType::Fifo,
            EntryType::Socket,
            EntryType::Unknown,
        ];
        
        for t in types {
            let _ = format!("{:?}", t);
        }
    }
}
