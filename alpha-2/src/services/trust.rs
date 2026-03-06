use crate::common::TosState;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum CommandClass {
    PrivilegeEscalation,
    RecursiveBulk,
    ImplicitBulk,
    Standard,
}

pub struct TrustService {}

impl TrustService {
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluates a command against the Brain-side classifier.
    pub fn classify_command(&self, command: &str, cwd: &Path, bulk_threshold: usize) -> CommandClass {
        let cmd = command.trim();
        
        // Stage 1a: Privilege escalation (explicit match)
        if cmd == "su" || cmd.starts_with("sudo ") || cmd.starts_with("su ") || cmd.starts_with("doas ") || cmd.starts_with("pkexec ") {
            return CommandClass::PrivilegeEscalation;
        }

        let is_destructive = cmd.starts_with("rm ") || cmd.starts_with("cp ") || cmd.starts_with("mv ") || cmd.starts_with("chmod ") || cmd.starts_with("chown ");
        let is_recursive = cmd.contains(" -r") || cmd.contains(" -R") || cmd.contains(" --recursive");

        // Stage 1b: Recursive bulk logic
        if is_destructive && is_recursive {
            return CommandClass::RecursiveBulk;
        }

        // Stage 2: Implicit bulk detection via glob estimation
        if is_destructive && cmd.contains('*') {
            let mut total_matches = 0;
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            
            for part in parts.iter().skip(1) {
                if part.contains('*') {
                    // Turn shell glob to regex loosely for estimation
                    let regex_pattern = format!("^{}$", part.replace(".", "\\.").replace("*", ".*").replace("?", "."));
                    if let Ok(re) = regex::Regex::new(&regex_pattern) {
                        if let Ok(entries) = std::fs::read_dir(cwd) {
                            for e in entries.flatten() {
                                if let Ok(name) = e.file_name().into_string() {
                                    if re.is_match(&name) {
                                        total_matches += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if total_matches >= bulk_threshold {
                return CommandClass::ImplicitBulk;
            }
        }

        CommandClass::Standard
    }

    /// Implement the TrustService cascade resolution (Sector -> Global).
    pub fn get_trust_policy(&self, state: &TosState, sector_id: Option<&str>, class: &CommandClass) -> String {
        let class_key = match class {
            CommandClass::PrivilegeEscalation => "tos.trust.privilege_escalation",
            CommandClass::RecursiveBulk => "tos.trust.recursive_bulk",
            CommandClass::ImplicitBulk => "tos.trust.recursive_bulk",
            CommandClass::Standard => return "allow".to_string(),
        };
        state.settings.resolve(class_key, sector_id, None)
            .unwrap_or_else(|| "warn".to_string())
    }

    /// Promote a class to "allow" globally.
    pub fn promote_global(&self, state: &mut TosState, class_key: &str) {
        state.settings.global.insert(class_key.to_string(), "allow".to_string());
    }

    /// Demote a class to "block" globally.
    pub fn demote_global(&self, state: &mut TosState, class_key: &str) {
        state.settings.global.insert(class_key.to_string(), "block".to_string());
    }

    /// Promote a class to "allow" for a specific sector.
    pub fn promote_sector(&self, state: &mut TosState, sector_id: &str, class_key: &str) {
        state.settings.sectors
            .entry(sector_id.to_string())
            .or_default()
            .insert(class_key.to_string(), "allow".to_string());
    }

    /// Demote a class to "block" for a specific sector.
    pub fn demote_sector(&self, state: &mut TosState, sector_id: &str, class_key: &str) {
        state.settings.sectors
            .entry(sector_id.to_string())
            .or_default()
            .insert(class_key.to_string(), "block".to_string());
    }

    /// Clear all per-sector trust overrides for a sector.
    pub fn clear_sector(&self, state: &mut TosState, sector_id: &str) {
        state.settings.sectors.remove(sector_id);
    }

    /// Return the current trust configuration as a JSON string.
    pub fn get_config_json(&self, state: &TosState) -> String {
        let keys = [
            "tos.trust.privilege_escalation",
            "tos.trust.recursive_bulk",
            "tos.trust.bulk_threshold",
        ];
        let mut config = serde_json::Map::new();
        for key in &keys {
            let val = state.settings.global.get(*key).cloned().unwrap_or_else(|| "warn".to_string());
            config.insert(key.to_string(), serde_json::Value::String(val));
        }
        serde_json::to_string(&config).unwrap_or_else(|_| "{}".to_string())
    }
}

