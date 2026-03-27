//! Tests for Shell API implementation

use tos_core::system::shell_api::{
    OscParser, OscSequence, ShellApi, ShellApiConfig, ShellCommand,
    CommandSuggestion, RiskLevel
};
use tos_core::{TosState, DirectoryListing, DirectoryEntry, EntryType};

#[test]
fn test_shell_api_config_default() {
    let config = ShellApiConfig::default();
    
    assert!(config.enable_osc);
    assert!(config.enable_commands);
    assert_eq!(config.shell_type, "fish");
    assert_eq!(config.max_suggestions, 10);
    assert_eq!(config.max_dir_entries, 1000);
    assert!(config.auto_execute_safe);
    assert!(!config.dangerous_patterns.is_empty());
}

#[test]
fn test_osc_parser_creation() {
    let parser = OscParser::new();
    assert!(parser.buffer.is_empty());
}

#[test]
fn test_parse_osc_cwd() {
    let mut parser = OscParser::new();
    let input = "\x1b]9003;/home/user/projects\x07";
    
    let (sequences, _remaining) = parser.parse(input);
    
    assert_eq!(sequences.len(), 1);
    assert!(_remaining.is_empty());
    
    if let OscSequence::Cwd(path) = &sequences[0] {
        assert_eq!(path, "/home/user/projects");
    } else {
        panic!("Expected Cwd sequence, got {:?}", sequences[0]);
    }
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
        assert_eq!(suggestions[0].command, "ls -la");
        assert_eq!(suggestions[0].description, "List all files");
        assert_eq!(suggestions[0].category, "files");
        
        assert_eq!(suggestions[1].text, "cd");
        assert_eq!(suggestions[1].category, "nav");
    } else {
        panic!("Expected Suggestions sequence");
    }
}

#[test]
fn test_parse_osc_directory() {
    let mut parser = OscParser::new();
    let input = "\x1b]9001;/home/user;/home;3;1;0\nfile1.txt;f;1024;-rw-r--r--;2024-01-15\ndocuments;d;4096;drwxr-xr-x;2024-01-14\n.hidden;f;512;-rw-r--r--;2024-01-13\x07";
    
    let (sequences, _remaining) = parser.parse(input);
    
    assert_eq!(sequences.len(), 1);
    
    if let OscSequence::Directory(listing) = &sequences[0] {
        assert_eq!(listing.path, "/home/user");
        assert_eq!(listing.parent, Some("/home".to_string()));
        assert_eq!(listing.total_count, 3);
        assert_eq!(listing.hidden_count, 1);
        assert_eq!(listing.selected_count, 0);
        assert_eq!(listing.entries.len(), 3);
        
        assert_eq!(listing.entries[0].name, "file1.txt");
        assert_eq!(listing.entries[0].entry_type, EntryType::File);
        assert_eq!(listing.entries[0].size, 1024);
        
        assert_eq!(listing.entries[1].name, "documents");
        assert_eq!(listing.entries[1].entry_type, EntryType::Directory);
        
        assert!(listing.entries[2].is_hidden);
    } else {
        panic!("Expected Directory sequence");
    }
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
fn test_parse_osc_env() {
    let mut parser = OscParser::new();
    let input = "\x1b]9004;PATH=/usr/bin:/bin\x07";
    
    let (sequences, _remaining) = parser.parse(input);
    
    assert_eq!(sequences.len(), 1);
    
    if let OscSequence::Env { key, value } = &sequences[0] {
        assert_eq!(key, "PATH");
        assert_eq!(value, "/usr/bin:/bin");
    } else {
        panic!("Expected Env sequence");
    }
}

#[test]
fn test_parse_osc_dangerous_command() {
    let mut parser = OscParser::new();
    let input = "\x1b]9005;critical;rm -rf /\x07";
    
    let (sequences, _remaining) = parser.parse(input);
    
    assert_eq!(sequences.len(), 1);
    
    if let OscSequence::DangerousCommand { command, risk_level } = &sequences[0] {
        assert_eq!(command, "rm -rf /");
        assert_eq!(*risk_level, RiskLevel::Critical);
    } else {
        panic!("Expected DangerousCommand sequence");
    }
}

#[test]
fn test_parse_osc_shell_ready() {
    let mut parser = OscParser::new();
    let input = "\x1b]9006;fish;3.6.0\x07";
    
    let (sequences, _remaining) = parser.parse(input);
    
    assert_eq!(sequences.len(), 1);
    
    if let OscSequence::ShellReady { shell_type, version } = &sequences[0] {
        assert_eq!(shell_type, "fish");
        assert_eq!(version, "3.6.0");
    } else {
        panic!("Expected ShellReady sequence");
    }
}

#[test]
fn test_parse_osc_request_completion() {
    let mut parser = OscParser::new();
    let input = "\x1b]9007;git com;5\x07";
    
    let (sequences, _remaining) = parser.parse(input);
    
    assert_eq!(sequences.len(), 1);
    
    if let OscSequence::RequestCompletion { partial, cursor_pos } = &sequences[0] {
        assert_eq!(partial, "git com");
        assert_eq!(*cursor_pos, 5);
    } else {
        panic!("Expected RequestCompletion sequence");
    }
}

#[test]
fn test_parse_multiple_sequences() {
    let mut parser = OscParser::new();
    let input = "\x1b]9003;/home/user\x07\x1b]9004;EDITOR=vim\x07";
    
    let (sequences, _remaining) = parser.parse(input);
    
    assert_eq!(sequences.len(), 2);
    assert!(matches!(sequences[0], OscSequence::Cwd(_)));
    assert!(matches!(sequences[1], OscSequence::Env { .. }));
}

#[test]
fn test_parse_with_remaining_text() {
    let mut parser = OscParser::new();
    let input = "Hello \x1b]9003;/home/user\x07 World";
    
    let (sequences, _remaining) = parser.parse(input);
    
    assert_eq!(sequences.len(), 1);
    assert_eq!(_remaining, "Hello  World");
}

#[test]
fn test_parse_unknown_osc() {
    let mut parser = OscParser::new();
    let input = "\x1b]9999;unknown data\x07";
    
    let (sequences, _remaining) = parser.parse(input);
    
    // Unknown OSC codes should be ignored
    assert!(sequences.is_empty());
    assert!(_remaining.is_empty());
}

#[test]
fn test_check_dangerous_commands() {
    let parser = OscParser::new();
    
    // Critical commands
    let result = parser.check_dangerous("rm -rf /");
    assert!(result.is_some());
    let (risk, msg) = result.unwrap();
    assert_eq!(risk, RiskLevel::Critical);
    assert!(msg.contains("filesystem"));
    
    let result = parser.check_dangerous("dd if=/dev/zero of=/dev/sda");
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::Critical);
    
    let result = parser.check_dangerous("mkfs.ext4 /dev/sdb1");
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::Critical);
    
    // High risk commands
    let result = parser.check_dangerous("echo data > /dev/sda");
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::High);
    
    let result = parser.check_dangerous("chmod -R 777 /");
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::High);
    
    let result = parser.check_dangerous(":(){ :|:& };:");
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::High);
    
    // Medium risk commands
    let result = parser.check_dangerous("wget http://example.com/script.sh | bash");
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::Medium);
    
    let result = parser.check_dangerous("curl -s http://example.com | sh");
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::Medium);
    
    // Safe commands
    let result = parser.check_dangerous("ls -la");
    assert!(result.is_none());
    
    let result = parser.check_dangerous("cd /home/user");
    assert!(result.is_none());
    
    let result = parser.check_dangerous("cat file.txt");
    assert!(result.is_none());
}

#[test]
fn test_format_shell_command() {
    let parser = OscParser::new();
    
    let cmd = ShellCommand::Exec("ls -la".to_string());
    assert_eq!(parser.format_shell_command(&cmd), "ls -la\n");
    
    let cmd = ShellCommand::Cd("/home/user".to_string());
    assert_eq!(parser.format_shell_command(&cmd), "cd /home/user\n");
    
    let cmd = ShellCommand::Ls("/tmp".to_string());
    assert_eq!(parser.format_shell_command(&cmd), "ls -la /tmp\n");
    
    let cmd = ShellCommand::SetEnv { 
        key: "EDITOR".to_string(), 
        value: "vim".to_string() 
    };
    assert_eq!(parser.format_shell_command(&cmd), "export EDITOR=vim\n");
    
    let cmd = ShellCommand::Clear;
    assert_eq!(parser.format_shell_command(&cmd), "clear\n");
    
    let cmd = ShellCommand::Interrupt;
    assert_eq!(parser.format_shell_command(&cmd), "\x03");
    
    let cmd = ShellCommand::Eof;
    assert_eq!(parser.format_shell_command(&cmd), "\x04");
    
    let cmd = ShellCommand::Complete { 
        partial: "git com".to_string(), 
        cursor_pos: 5 
    };
    assert!(parser.format_shell_command(&cmd).contains("9008"));
    assert!(parser.format_shell_command(&cmd).contains("git com"));
}

#[test]
fn test_shell_escape() {
    // Simple strings don't need escaping
    assert_eq!(tos_core::system::shell_api::shell_escape("simple"), "simple");
    
    // Strings with spaces need quotes
    assert_eq!(tos_core::system::shell_api::shell_escape("with space"), "'with space'");
    
    // Strings with quotes need special handling
    assert_eq!(tos_core::system::shell_api::shell_escape("with'quote"), "'with'\\\"'\\\"'quote'");
}

#[test]
fn test_generate_fish_integration() {
    let parser = OscParser::new();
    let script = parser.generate_fish_integration();
    
    assert!(script.contains("TOS Shell Integration for Fish"));
    assert!(script.contains("9003")); // CWD OSC
    assert!(script.contains("9002")); // Command result OSC
    assert!(script.contains("9001")); // Directory listing OSC
    assert!(script.contains("fish_preexec"));
    assert!(script.contains("fish_postexec"));
    assert!(script.contains("__tos_send_osc"));
    assert!(script.contains("__tos_ls"));
}

#[test]
fn test_generate_bash_integration() {
    let parser = OscParser::new();
    let script = parser.generate_bash_integration();
    
    assert!(script.contains("TOS Shell Integration for Bash"));
    assert!(script.contains("PROMPT_COMMAND"));
    assert!(script.contains("9003")); // CWD OSC
    assert!(script.contains("9002")); // Command result OSC
    assert!(script.contains("trap"));
    assert!(script.contains("DEBUG"));
}

#[test]
fn test_shell_api_creation() {
    let api = ShellApi::new();
    assert!(api.pending_completions.is_empty());
}

#[test]
fn test_shell_api_process_output() {
    let mut api = ShellApi::new();
    let mut state = TosState::new();
    
    let output = "Output\x1b]9002;ls;0;file1 file2\x07More output";
    let clean = api.process_output(output, &mut state);
    
    assert_eq!(clean, "OutputMore output");
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

#[test]
fn test_command_suggestion_creation() {
    let suggestion = CommandSuggestion {
        text: "ls".to_string(),
        command: "ls -la".to_string(),
        description: "List all files".to_string(),
        category: "files".to_string(),
        confidence: 0.9,
    };
    
    assert_eq!(suggestion.text, "ls");
    assert_eq!(suggestion.confidence, 0.9);
}

#[test]
fn test_directory_listing_creation() {
    let entry = DirectoryEntry {
        name: "test.txt".to_string(),
        entry_type: EntryType::File,
        size: 1024,
        permissions: "-rw-r--r--".to_string(),
        modified: "2024-01-15".to_string(),
        is_hidden: false,
        is_selected: false,
    };
    
    let listing = DirectoryListing {
        path: "/home/user".to_string(),
        parent: Some("/home".to_string()),
        entries: vec![entry],
        total_count: 1,
        hidden_count: 0,
        selected_count: 0,
    };
    
    assert_eq!(listing.path, "/home/user");
    assert_eq!(listing.entries.len(), 1);
    assert_eq!(listing.entries[0].name, "test.txt");
}

#[test]
fn test_osc_with_st_terminator() {
    let mut parser = OscParser::new();
    // ST (String Terminator) is ESC \
    let input = "\x1b]9003;/home/user\x1b\\";
    
    let (sequences, _remaining) = parser.parse(input);
    
    assert_eq!(sequences.len(), 1);
    if let OscSequence::Cwd(path) = &sequences[0] {
        assert_eq!(path, "/home/user");
    } else {
        panic!("Expected Cwd sequence");
    }
}

#[test]
fn test_empty_osc() {
    let mut parser = OscParser::new();
    let input = "\x1b]\x07";
    
    let (sequences, _remaining) = parser.parse(input);
    
    // Empty OSC should be ignored
    assert!(sequences.is_empty());
    assert!(_remaining.is_empty());
}

#[test]
fn test_shell_api_config_customization() {
    let mut config = ShellApiConfig::default();
    
    config.shell_type = "zsh".to_string();
    config.max_suggestions = 20;
    config.max_dir_entries = 500;
    config.auto_execute_safe = false;
    
    assert_eq!(config.shell_type, "zsh");
    assert_eq!(config.max_suggestions, 20);
    assert_eq!(config.max_dir_entries, 500);
    assert!(!config.auto_execute_safe);
}
