use serde::{Deserialize, Serialize};
use toml;
use std::path::PathBuf;
use ed25519_dalek::{Signature, VerifyingKey};
use ed25519_dalek::Verifier;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub module_type: String, // "Application", "TerminalOutput", "Theme", etc.
    pub author: String,
    // The Ed25519 cryptographic signature of the manifest contents
    pub signature: Option<String>,
}

pub struct MarketplaceService;

impl MarketplaceService {
    /// Discover module in a directory. Attempts to use Marketplace Daemon first.
    pub fn discover_module(mut path: PathBuf) -> anyhow::Result<ModuleManifest> {
        let path_str = path.to_string_lossy().to_string();
        if let Ok(mut stream) = std::net::TcpStream::connect_timeout(&"127.0.0.1:7004".parse().unwrap(), std::time::Duration::from_millis(100)) {
            use std::io::{Write, BufRead, BufReader};
            let _ = stream.write_all(format!("discover:{}\n", path_str).as_bytes());
            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            if let Ok(_) = reader.read_line(&mut response) {
                if let Ok(m) = serde_json::from_str(&response) {
                    return Ok(m);
                }
            }
        }

        path.push("module.toml");
        let content = std::fs::read_to_string(&path)?;
        let manifest: ModuleManifest = toml::from_str(&content)?;
        Ok(manifest)
    }

    /// Retrieve the TOS public key.
    pub fn get_trusted_public_key() -> anyhow::Result<VerifyingKey> {
        // Mock trusted public key for testing validation
        let public_key_hex = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
        let pub_bytes = hex::decode(public_key_hex)?;
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&pub_bytes);
        Ok(VerifyingKey::from_bytes(&key_bytes)?)
    }

    /// Verifies the cryptographic signature of a module manifest.
    pub fn verify_manifest(manifest: &ModuleManifest, public_key: &VerifyingKey) -> bool {
        // Attempt specialized daemon verification first
        if let Ok(mut stream) = std::net::TcpStream::connect_timeout(&"127.0.0.1:7004".parse().unwrap(), std::time::Duration::from_millis(100)) {
            use std::io::{Write, BufRead, BufReader};
            let manifest_json = serde_json::to_string(manifest).unwrap_or_default();
            let _ = stream.write_all(format!("verify:{}\n", manifest_json).as_bytes());
            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            if let Ok(_) = reader.read_line(&mut response) {
                if response.trim() == "VALID" { return true; }
                if response.trim() == "INVALID" { return false; }
            }
        }

        let sig_hex = match &manifest.signature {
            Some(s) => s,
            None => return false,
        };

        let sig_bytes = match hex::decode(sig_hex) {
            Ok(b) => b,
            Err(_) => return false,
        };

        if sig_bytes.len() != 64 { return false; }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);
        let signature = Signature::from_bytes(&sig_array);

        let payload = format!(
            "{}:{}:{}:{}:{}",
            manifest.id,
            manifest.name,
            manifest.version,
            manifest.module_type,
            manifest.author
        );

        public_key.verify(payload.as_bytes(), &signature).is_ok()
    }
}
