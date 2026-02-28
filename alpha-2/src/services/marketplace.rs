use serde::{Deserialize, Serialize};
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
    /// Discover module in a directory.
    pub fn discover_module(mut path: PathBuf) -> anyhow::Result<ModuleManifest> {
        path.push("module.toml");
        
        let content = std::fs::read_to_string(&path)?;
        let manifest: ModuleManifest = toml::from_str(&content)?;
        Ok(manifest)
    }

    /// Retrieve the TOS public key used to verify Marketplace modules.
    /// In a production environment, this would be embedded in the binary or fetched securely.
    pub fn get_trusted_public_key() -> anyhow::Result<VerifyingKey> {
        // Mock trusted public key for testing validation
        // (This matches the mock key generated in tests/tier4_security_manifest.rs)
        let public_key_hex = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
        let pub_bytes = hex::decode(public_key_hex)?;
        
        if pub_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid public key length"));
        }
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&pub_bytes);
        
        Ok(VerifyingKey::from_bytes(&key_bytes)?)
    }

    /// Verify the cryptographic signature of a module manifest.
    pub fn verify_manifest(manifest: &ModuleManifest, public_key: &VerifyingKey) -> bool {
        let sig_hex = match &manifest.signature {
            Some(s) => s,
            None => {
                tracing::warn!("Signature missing for module: {}", manifest.name);
                return false;
            }
        };

        let sig_bytes = match hex::decode(sig_hex) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("Failed to decode signature hex: {}", e);
                return false;
            }
        };

        if sig_bytes.len() != 64 {
            tracing::warn!("Invalid signature length: {}", sig_bytes.len());
            return false;
        }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);
        let signature = Signature::from_bytes(&sig_array);

        // We hash the manifest without the signature field itself
        let payload = format!(
            "{}:{}:{}:{}:{}",
            manifest.id,
            manifest.name,
            manifest.version,
            manifest.module_type,
            manifest.author
        );

        match public_key.verify(payload.as_bytes(), &signature) {
            Ok(_) => {
                tracing::info!("Valid Marketplace signature confirmed: {}", manifest.name);
                true
            }
            Err(e) => {
                tracing::warn!("Cryptographic validation failed for {}: {}", manifest.name, e);
                false
            }
        }
    }
}
