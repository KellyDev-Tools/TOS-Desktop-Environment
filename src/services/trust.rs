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

    /// Verifies the cryptographic signature of a service registration request (§4.1).
    pub fn verify_service_signature(&self, req: &crate::common::ipc::ServiceRegister) -> bool {
        use ed25519_dalek::{VerifyingKey, Signature, Verifier};
        use std::convert::TryInto;

        let pk_bytes = match hex::decode(&req.public_key) {
            Ok(b) => b,
            Err(_) => return false,
        };
        let sig_bytes = match hex::decode(&req.signature) {
            Ok(b) => b,
            Err(_) => return false,
        };

        let pk_arr: [u8; 32] = match pk_bytes.try_into() {
            Ok(a) => a,
            Err(_) => return false,
        };
        let sig_arr: [u8; 64] = match sig_bytes.try_into() {
            Ok(a) => a,
            Err(_) => return false,
        };

        let verifying_key = match VerifyingKey::from_bytes(&pk_arr) {
            Ok(k) => k,
            Err(_) => return false,
        };
        let signature = Signature::from_bytes(&sig_arr);

        // Payload for signing is "name:port" (consistent with daemon logic)
        let payload = format!("{}:{}", req.name, req.port);
        verifying_key.verify(payload.as_bytes(), &signature).is_ok()
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
            .unwrap_or_else(|| "confirm".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::ipc::ServiceRegister;
    use ed25519_dalek::{SigningKey, Signer};
    use rand_core::OsRng;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_classify_standard() {
        let trust = TrustService::new();
        let cwd = Path::new("/");
        assert_eq!(trust.classify_command("ls -la", cwd, 10), CommandClass::Standard);
        assert_eq!(trust.classify_command("cat file.txt", cwd, 10), CommandClass::Standard);
    }

    #[test]
    fn test_classify_privilege_escalation() {
        let trust = TrustService::new();
        let cwd = Path::new("/");
        assert_eq!(trust.classify_command("sudo apt update", cwd, 10), CommandClass::PrivilegeEscalation);
        assert_eq!(trust.classify_command("su -", cwd, 10), CommandClass::PrivilegeEscalation);
        assert_eq!(trust.classify_command("pkexec reboot", cwd, 10), CommandClass::PrivilegeEscalation);
    }

    #[test]
    fn test_classify_recursive_bulk() {
        let trust = TrustService::new();
        let cwd = Path::new("/");
        assert_eq!(trust.classify_command("rm -rf /tmp/foo", cwd, 10), CommandClass::RecursiveBulk);
        assert_eq!(trust.classify_command("cp -R a b", cwd, 10), CommandClass::RecursiveBulk);
    }

    #[test]
    fn test_classify_implicit_bulk_glob() {
        let trust = TrustService::new();
        let dir = tempdir().unwrap();
        let path = dir.path();
        
        // Create 5 files
        for i in 0..5 {
            File::create(path.join(format!("test_{}.txt", i))).unwrap();
        }

        // Threshold 3 -> Should be ImplicitBulk
        assert_eq!(trust.classify_command("rm *.txt", path, 3), CommandClass::ImplicitBulk);
        
        // Threshold 10 -> Should be Standard
        assert_eq!(trust.classify_command("rm *.txt", path, 10), CommandClass::Standard);
    }

    #[test]
    fn test_verify_service_signature() {
        let trust = TrustService::new();
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        
        let name = "test-service";
        let port = 12345;
        let payload = format!("{}:{}", name, port);
        let signature = signing_key.sign(payload.as_bytes());

        let req = ServiceRegister {
            name: name.to_string(),
            port,
            public_key: hex::encode(verifying_key.to_bytes()),
            signature: hex::encode(signature.to_bytes()),
        };

        assert!(trust.verify_service_signature(&req));

        // Tamper with port
        let mut tampered_req = req.clone();
        tampered_req.port = 54321;
        assert!(!trust.verify_service_signature(&tampered_req));

        // Tamper with signature
        let mut tampered_sig_req = req.clone();
        tampered_sig_req.signature = "0".repeat(128);
        assert!(!trust.verify_service_signature(&tampered_sig_req));
    }
}

