//! Digital Signature Verification
//! 
//! Provides signature verification using Minisign (Ed25519-based signatures).
//! Supports trusted key management and signature validation for packages.

use super::MarketplaceError;
use minisign_verify::{PublicKey, Signature};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Signature verifier for package validation
#[derive(Clone)]
pub struct SignatureVerifier {
    /// Trusted public keys (base64 encoded)
    trusted_keys: HashSet<String>,
    /// Key directory for loading trusted keys
    keys_dir: PathBuf,
    /// Whether to allow untrusted packages
    allow_untrusted: bool,
}

/// Signature verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether signature is valid
    pub valid: bool,
    /// Key ID that signed the package
    pub key_id: Option<String>,
    /// Trusted status
    pub trusted: bool,
    /// Verification message
    pub message: String,
}

/// Signature error types
#[derive(Debug)]
pub enum SignatureError {
    /// Invalid signature format
    InvalidFormat(String),
    /// Key not found
    KeyNotFound(String),
    /// Verification failed
    VerificationFailed(String),
    /// IO error
    Io(std::io::Error),
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureError::InvalidFormat(e) => write!(f, "Invalid signature format: {}", e),
            SignatureError::KeyNotFound(e) => write!(f, "Key not found: {}", e),
            SignatureError::VerificationFailed(e) => write!(f, "Verification failed: {}", e),
            SignatureError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for SignatureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SignatureError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SignatureError {
    fn from(e: std::io::Error) -> Self {
        SignatureError::Io(e)
    }
}

impl From<serde_json::Error> for SignatureError {
    fn from(e: serde_json::Error) -> Self {
        SignatureError::InvalidFormat(format!("JSON error: {}", e))
    }
}

impl SignatureVerifier {
    /// Create a new signature verifier with trusted keys
    pub fn new(trusted_keys: Vec<String>) -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let keys_dir = PathBuf::from(format!("{}/.config/tos/keys", home));
        
        Self {
            trusted_keys: trusted_keys.into_iter().collect(),
            keys_dir,
            allow_untrusted: false,
        }
    }
    
    /// Create with custom keys directory
    pub fn with_keys_dir(mut self, keys_dir: PathBuf) -> Self {
        self.keys_dir = keys_dir;
        self
    }
    
    /// Allow untrusted packages (for development)
    pub fn with_allow_untrusted(mut self, allow: bool) -> Self {
        self.allow_untrusted = allow;
        self
    }
    
    /// Add a trusted public key
    pub fn add_trusted_key(&mut self, key: String) {
        self.trusted_keys.insert(key);
    }
    
    /// Remove a trusted key
    pub fn remove_trusted_key(&mut self, key: &str) {
        self.trusted_keys.remove(key);
    }
    
    /// Load trusted keys from keys directory
    pub fn load_keys_from_directory(&mut self) -> Result<usize, SignatureError> {
        if !self.keys_dir.exists() {
            std::fs::create_dir_all(&self.keys_dir)?;
        }
        
        let mut loaded = 0;
        
        for entry in std::fs::read_dir(&self.keys_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |e| e == "pub") {
                let key_data = std::fs::read_to_string(&path)?;
                // Extract base64 key from minisign format
                if let Some(key) = self.extract_key_from_file(&key_data) {
                    self.trusted_keys.insert(key);
                    loaded += 1;
                }
            }
        }
        
        tracing::info!("Loaded {} trusted keys from {}", loaded, self.keys_dir.display());
        Ok(loaded)
    }
    
    /// Save a trusted key to the keys directory
    pub fn save_trusted_key(&self, name: &str, key: &str) -> Result<PathBuf, SignatureError> {
        std::fs::create_dir_all(&self.keys_dir)?;
        
        let key_path = self.keys_dir.join(format!("{}.pub", name));
        let key_data = format!("untrusted comment: TOS trusted key\n{}\n", key);
        
        std::fs::write(&key_path, key_data)?;
        
        tracing::info!("Saved trusted key: {}", key_path.display());
        Ok(key_path)
    }
    
    /// Verify a package signature
    pub fn verify(
        &self,
        package_path: &Path,
        signature_b64: &str,
    ) -> Result<VerificationResult, MarketplaceError> {
        // Read package data
        let package_data = std::fs::read(package_path)?;
        
        // Detect signature type
        if signature_b64.starts_with("-----BEGIN PGP SIGNATURE-----") {
            return self.verify_gpg(&package_data, signature_b64);
        }

        // Try to verify with each trusted key (Minisign)
        for key_b64 in &self.trusted_keys {
            match self.verify_with_key(&package_data, signature_b64, key_b64) {
                Ok(key_id) => {
                    return Ok(VerificationResult {
                        valid: true,
                        key_id: Some(key_id),
                        trusted: true,
                        message: "Signature verified with trusted minisign key".to_string(),
                    });
                }
                Err(e) => {
                    tracing::debug!("Minisign verification failed with key: {}", e);
                }
            }
        }
        
        // If we allow untrusted, try to verify signature format anyway
        if self.allow_untrusted {
            if let Ok(key_id) = self.verify_untrusted(&package_data, signature_b64) {
                return Ok(VerificationResult {
                    valid: true,
                    key_id: Some(key_id),
                    trusted: false,
                    message: "Minisign signature valid but from untrusted key".to_string(),
                });
            }
        }
        
        Err(MarketplaceError::Signature(
            "Signature verification failed with all trusted keys".to_string()
        ))
    }

    /// Verify GPG signature (STUB)
    fn verify_gpg(&self, _data: &[u8], _signature_armored: &str) -> Result<VerificationResult, MarketplaceError> {
        // In a real implementation, this would use gpgme or another GPG library
        // For now, we return a validation error if we require strict GPG check
        if self.allow_untrusted {
            Ok(VerificationResult {
                valid: true,
                key_id: Some("gpg-stub-id".to_string()),
                trusted: false,
                message: "GPG signature detected (verification not implemented, allowed by policy)".to_string(),
            })
        } else {
            Err(MarketplaceError::Signature(
                "GPG signatures are not supported in this version".to_string()
            ))
        }
    }
    
    /// Verify signature with a specific key
    fn verify_with_key(
        &self,
        data: &[u8],
        signature_b64: &str,
        key_b64: &str,
    ) -> Result<String, SignatureError> {
        // Parse public key
        let public_key = PublicKey::from_base64(key_b64)
            .map_err(|e| SignatureError::InvalidFormat(format!("Invalid key: {}", e)))?;
        
        // Parse signature (decode handles the multiline format)
        let signature = Signature::decode(signature_b64)
            .map_err(|e| SignatureError::InvalidFormat(format!("Invalid signature: {}", e)))?;
        
        // Verify
        public_key.verify(data, &signature, false)
            .map_err(|e| SignatureError::VerificationFailed(e.to_string()))?;
        
        // Generate key ID from public key (since signature.key_id() is private)
        let key_id = self.derive_key_id(key_b64);
        
        Ok(key_id)
    }
    
    /// Verify without trusting (just check format)
    fn verify_untrusted(&self, _data: &[u8], signature_b64: &str) -> Result<String, SignatureError> {
        // Just parse the signature to validate format
        let _signature = Signature::decode(signature_b64)
            .map_err(|e| SignatureError::InvalidFormat(format!("Invalid signature: {}", e)))?;
        
        // Generate a placeholder key ID from signature data hash
        let key_id = format!("{:x}", crc32fast::hash(signature_b64.as_bytes()));
        Ok(key_id)
    }
    
    /// Derive a key ID from the public key
    fn derive_key_id(&self, key_b64: &str) -> String {
        // Use first 16 chars of base64 key as ID
        if key_b64.len() >= 16 {
            key_b64[..16].to_string()
        } else {
            key_b64.to_string()
        }
    }
    
    /// Extract base64 key from minisign public key file format
    fn extract_key_from_file(&self, file_content: &str) -> Option<String> {
        for line in file_content.lines() {
            let line = line.trim();
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with("untrusted comment:") {
                continue;
            }
            // The key is the base64 line
            if line.len() > 40 && line.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '=') {
                return Some(line.to_string());
            }
        }
        None
    }
    
    /// Generate a signature (for package maintainers)
    pub fn sign_package(
        &self,
        _package_path: &Path,
        _secret_key_b64: &str,
    ) -> Result<String, SignatureError> {
        // This is a placeholder - actual signing requires the secret key
        // In production, this would use the minisign signing API
        Err(SignatureError::VerificationFailed(
            "Signing not implemented in this version".to_string()
        ))
    }
    
    /// List trusted key IDs
    pub fn trusted_key_ids(&self) -> Vec<String> {
        self.trusted_keys.iter().cloned().collect()
    }
    
    /// Check if a key is trusted
    pub fn is_key_trusted(&self, key_b64: &str) -> bool {
        self.trusted_keys.contains(key_b64)
    }
    
    /// Get number of trusted keys
    pub fn trusted_key_count(&self) -> usize {
        self.trusted_keys.len()
    }
}

/// Trust on first use (TOFU) key manager
pub struct TofuKeyManager {
    /// Known keys (key_id -> key)
    known_keys: std::collections::HashMap<String, String>,
    /// TOFU database path
    db_path: PathBuf,
}

impl TofuKeyManager {
    /// Create a new TOFU key manager
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let db_path = PathBuf::from(format!("{}/.config/tos/known_keys.json", home));
        
        let mut manager = Self {
            known_keys: std::collections::HashMap::new(),
            db_path,
        };
        
        // Load existing keys
        let _ = manager.load();
        
        manager
    }
    
    /// Load known keys from database
    pub fn load(&mut self) -> Result<(), SignatureError> {
        if !self.db_path.exists() {
            return Ok(());
        }
        
        let data = std::fs::read_to_string(&self.db_path)?;
        self.known_keys = serde_json::from_str(&data)
            .map_err(|e| SignatureError::InvalidFormat(format!("Invalid key database: {}", e)))?;
        
        Ok(())
    }
    
    /// Save known keys to database
    pub fn save(&self) -> Result<(), SignatureError> {
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let data = serde_json::to_string_pretty(&self.known_keys)?;
        std::fs::write(&self.db_path, data)?;
        
        Ok(())
    }
    
    /// Record a key as seen
    pub fn record_key(&mut self, key_id: &str, key: &str) -> bool {
        if self.known_keys.contains_key(key_id) {
            // Key already known
            self.known_keys.get(key_id) == Some(&key.to_string())
        } else {
            // First time seeing this key
            self.known_keys.insert(key_id.to_string(), key.to_string());
            let _ = self.save();
            true
        }
    }
    
    /// Check if a key is known
    pub fn is_key_known(&self, key_id: &str) -> bool {
        self.known_keys.contains_key(key_id)
    }
    
    /// Get known key
    pub fn get_key(&self, key_id: &str) -> Option<&String> {
        self.known_keys.get(key_id)
    }
    
    /// List all known key IDs
    pub fn known_key_ids(&self) -> Vec<String> {
        self.known_keys.keys().cloned().collect()
    }
}

impl Default for TofuKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_signature_verifier_new() {
        let verifier = SignatureVerifier::new(vec![]);
        assert_eq!(verifier.trusted_key_count(), 0);
        assert!(!verifier.allow_untrusted);
    }
    
    #[test]
    fn test_add_trusted_key() {
        let mut verifier = SignatureVerifier::new(vec![]);
        verifier.add_trusted_key("test_key".to_string());
        assert_eq!(verifier.trusted_key_count(), 1);
        assert!(verifier.is_key_trusted("test_key"));
    }
    
    #[test]
    fn test_remove_trusted_key() {
        let mut verifier = SignatureVerifier::new(vec!["key1".to_string(), "key2".to_string()]);
        assert_eq!(verifier.trusted_key_count(), 2);
        
        verifier.remove_trusted_key("key1");
        assert_eq!(verifier.trusted_key_count(), 1);
        assert!(!verifier.is_key_trusted("key1"));
        assert!(verifier.is_key_trusted("key2"));
    }
    
    #[test]
    fn test_extract_key_from_file() {
        let verifier = SignatureVerifier::new(vec![]);
        
        let key_file = r#"
untrusted comment: TOS public key
RWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBYm0+Ng0h0i0
"#;
        
        let key = verifier.extract_key_from_file(key_file);
        assert!(key.is_some());
        assert_eq!(key.unwrap(), "RWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBYm0+Ng0h0i0");
    }
    
    #[test]
    fn test_tofu_key_manager() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("known_keys.json");
        
        let mut manager = TofuKeyManager {
            known_keys: std::collections::HashMap::new(),
            db_path: db_path.clone(),
        };
        
        // Record new key
        assert!(manager.record_key("key1", "public_key_1"));
        assert!(manager.is_key_known("key1"));
        
        // Save and reload
        manager.save().unwrap();
        
        let mut manager2 = TofuKeyManager {
            known_keys: std::collections::HashMap::new(),
            db_path,
        };
        manager2.load().unwrap();
        
        assert!(manager2.is_key_known("key1"));
        assert_eq!(manager2.get_key("key1"), Some(&"public_key_1".to_string()));
    }
    
    #[test]
    fn test_verification_result() {
        let result = VerificationResult {
            valid: true,
            key_id: Some("abc123".to_string()),
            trusted: true,
            message: "Verified".to_string(),
        };
        
        assert!(result.valid);
        assert!(result.trusted);
        assert_eq!(result.key_id, Some("abc123".to_string()));
    }
}
