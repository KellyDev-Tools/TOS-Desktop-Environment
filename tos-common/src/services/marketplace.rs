use ed25519_dalek::Verifier;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use toml;
use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub module_type: String, // "Application", "TerminalOutput", "Theme", "Shell", "AI", etc.
    pub author: String,
    pub description: Option<String>,
    pub icon: Option<String>,

    // §1.7: Shell Specifics
    pub executable: Option<ExecutableConfig>,
    pub integration: Option<crate::modules::ShellIntegration>,

    // §1.6: Theme Specifics
    pub assets: Option<crate::ThemeAssetDefinition>,

    // §1.3: AI Specifics
    pub capabilities: Option<Vec<String>>,
    /// LLM provider identifier: "openai", "anthropic", "ollama", or "module".
    pub provider: Option<String>,
    /// HTTP base URL for LLM API calls (e.g. "https://api.openai.com/v1").
    pub endpoint: Option<String>,
    /// Latency profile hint: "low" (<300ms p95), "medium" (<1s p95), "high" (>1s p95).
    pub latency_profile: Option<String>,

    // §1.4: AI Skill Specifics
    pub tool_bundle: Option<ToolBundleConfig>,

    // The Ed25519 cryptographic signature of the manifest contents
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolBundleConfig {
    pub allowed_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutableConfig {
    pub path: String,
    pub args: Vec<String>,
}

pub struct MarketplaceService {
    registry: Arc<Mutex<crate::services::registry::ServiceRegistry>>,
}

impl MarketplaceService {
    pub fn new(registry: Arc<Mutex<crate::services::registry::ServiceRegistry>>) -> Self {
        Self { registry }
    }

    fn get_port(&self) -> u16 {
        self.registry
            .lock()
            .unwrap()
            .port_of("tos-marketplaced")
            .unwrap_or(7004)
    }
    /// Discover module in a directory. Attempts to use Marketplace Daemon first.
    pub fn discover_module(&self, path: PathBuf) -> anyhow::Result<ModuleManifest> {
        let path_str = path.to_string_lossy().to_string();
        let port = self.get_port();
        if let Ok(mut stream) = std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{}", port).parse().unwrap(),
            std::time::Duration::from_millis(100),
        ) {
            use std::io::{BufRead, BufReader, Write};
            let _ = stream.write_all(format!("discover:{}\n", path_str).as_bytes());
            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            if reader.read_line(&mut response).is_ok() {
                if let Ok(m) = serde_json::from_str(&response) {
                    return Ok(m);
                }
            }
        }
        Self::discover_module_local(path)
    }

    /// Discover module in a directory, bypassing daemon check.
    pub fn discover_module_local(mut path: PathBuf) -> anyhow::Result<ModuleManifest> {
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
    pub fn verify_manifest(&self, manifest: &ModuleManifest, public_key: &VerifyingKey) -> bool {
        let port = self.get_port();
        if let Ok(mut stream) = std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{}", port).parse().unwrap(),
            std::time::Duration::from_millis(100),
        ) {
            use std::io::{BufRead, BufReader, Write};
            let manifest_json = serde_json::to_string(manifest).unwrap_or_default();
            let _ = stream.write_all(format!("verify:{}\n", manifest_json).as_bytes());
            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            if reader.read_line(&mut response).is_ok() {
                if response.trim() == "VALID" {
                    return true;
                }
                if response.trim() == "INVALID" {
                    return false;
                }
            }
        }
        Self::verify_manifest_local(manifest, public_key)
    }

    /// Verifies the cryptographic signature of a module manifest, bypassing daemon check.
    pub fn verify_manifest_local(manifest: &ModuleManifest, public_key: &VerifyingKey) -> bool {
        let sig_hex = match &manifest.signature {
            Some(s) => s,
            None => return false,
        };

        let sig_bytes = match hex::decode(sig_hex) {
            Ok(b) => b,
            Err(_) => return false,
        };

        if sig_bytes.len() != 64 {
            return false;
        }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);
        let signature = Signature::from_bytes(&sig_array);

        let payload = format!(
            "{}:{}:{}:{}:{}",
            manifest.id, manifest.name, manifest.version, manifest.module_type, manifest.author
        );

        public_key.verify(payload.as_bytes(), &signature).is_ok()
    }

    /// Lists terminal modules installed in the system modules directory.
    pub fn list_terminal_modules() -> Vec<crate::TerminalOutputModuleMeta> {
        let mut modules = Vec::new();
        let mut base_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        base_path.push(".config/tos/modules/terminal");

        if let Ok(entries) = std::fs::read_dir(base_path) {
            for entry in entries.flatten() {
                if let Ok(manifest) = Self::discover_module_local(entry.path()) {
                    if manifest.module_type == "TerminalOutput" {
                        modules.push(crate::TerminalOutputModuleMeta {
                            id: manifest.id.clone(),
                            name: manifest.name,
                            version: manifest.version,
                            layout: match manifest.id.as_str() {
                                id if id.contains("cinematic") => {
                                    crate::TerminalLayoutType::Cinematic
                                }
                                _ => crate::TerminalLayoutType::Rectangular,
                            },
                            supports_high_contrast: true,
                            supports_reduced_motion: true,
                        });
                    }
                }
            }
        }
        modules
    }

    /// Lists theme modules installed in the system modules directory.
    pub fn list_theme_modules() -> Vec<crate::ThemeModule> {
        let mut themes = Vec::new();
        let mut base_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        base_path.push(".config/tos/modules/themes");

        if let Ok(entries) = std::fs::read_dir(base_path) {
            for entry in entries.flatten() {
                if let Ok(manifest) = Self::discover_module_local(entry.path()) {
                    if manifest.module_type == "Theme" {
                        if let Some(assets) = manifest.assets {
                            themes.push(crate::ThemeModule {
                                id: manifest.id,
                                name: manifest.name,
                                version: manifest.version,
                                author: manifest.author,
                                description: manifest.description.unwrap_or_default(),
                                assets,
                            });
                        }
                    }
                }
            }
        }
        themes
    }

    /// Lists AI modules installed in the system modules directory.
    pub fn list_ai_modules() -> Vec<crate::AiModuleMetadata> {
        let mut modules = Vec::new();
        let mut base_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        base_path.push(".config/tos/modules/ai");

        if let Ok(entries) = std::fs::read_dir(base_path) {
            for entry in entries.flatten() {
                if let Ok(manifest) = Self::discover_module_local(entry.path()) {
                    if manifest.module_type == "AI" || manifest.module_type == "ai" {
                        modules.push(crate::AiModuleMetadata {
                            id: manifest.id,
                            name: manifest.name,
                            version: manifest.version,
                            author: manifest.author,
                            capabilities: manifest.capabilities.unwrap_or_default(),
                        });
                    }
                }
            }
        }
        modules
    }

    pub fn get_home(&self) -> anyhow::Result<MarketplaceHome> {
        let resp = self.remote_call("marketplace_home", "")?;
        serde_json::from_str(&resp).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn get_category(&self, cat_id: &str) -> anyhow::Result<Vec<MarketplaceModuleSummary>> {
        let resp = self.remote_call("marketplace_category", cat_id)?;
        serde_json::from_str(&resp).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn get_detail(&self, mod_id: &str) -> anyhow::Result<MarketplaceModuleDetail> {
        let resp = self.remote_call("marketplace_detail", mod_id)?;
        serde_json::from_str(&resp).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn install(&self, mod_id: &str) -> anyhow::Result<String> {
        self.remote_call("marketplace_install", mod_id)
    }

    pub fn get_status(&self, mod_id: &str) -> anyhow::Result<InstallProgress> {
        let resp = self.remote_call("marketplace_status", mod_id)?;
        serde_json::from_str(&resp).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn search_ai(&self, query: &str) -> anyhow::Result<Vec<MarketplaceModuleSummary>> {
        let resp = self.remote_call("marketplace_search_ai", query)?;
        serde_json::from_str(&resp).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn cancel_install(&self, mod_id: &str) -> anyhow::Result<String> {
        self.remote_call("marketplace_install_cancel", mod_id)
    }

    fn remote_call(&self, cmd: &str, payload: &str) -> anyhow::Result<String> {
        use std::io::{BufRead, BufReader, Write};
        let port = self.get_port();
        let mut stream = std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{}", port).parse().unwrap(),
            std::time::Duration::from_millis(100),
        )?;
        stream.write_all(format!("{}:{}\n", cmd, payload).as_bytes())?;
        let mut reader = BufReader::new(&stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;
        Ok(response.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand_core::OsRng;

    #[test]
    fn test_manifest_verification() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let mut manifest = ModuleManifest {
            id: "com.tos.test".to_string(),
            name: "Test Module".to_string(),
            version: "1.0.0".to_string(),
            module_type: "Theme".to_string(),
            author: "TOS Team".to_string(),
            description: None,
            icon: None,
            executable: None,
            integration: None,
            assets: None,
            capabilities: None,
            provider: None,
            endpoint: None,
            latency_profile: None,
            tool_bundle: None,
            signature: None,
        };

        let payload = format!(
            "{}:{}:{}:{}:{}",
            manifest.id, manifest.name, manifest.version, manifest.module_type, manifest.author
        );
        let signature = signing_key.sign(payload.as_bytes());
        manifest.signature = Some(hex::encode(signature.to_bytes()));

        assert!(MarketplaceService::verify_manifest_local(
            &manifest,
            &verifying_key
        ));

        // Tamper with version
        let mut tampered = manifest.clone();
        tampered.version = "1.0.1".to_string();
        assert!(!MarketplaceService::verify_manifest_local(
            &tampered,
            &verifying_key
        ));

        // Tamper with signature
        let mut tampered_sig = manifest.clone();
        tampered_sig.signature = Some("f".repeat(128));
        assert!(!MarketplaceService::verify_manifest_local(
            &tampered_sig,
            &verifying_key
        ));
    }
}
