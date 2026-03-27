use uuid::Uuid;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalToken {
    pub token: String,
    pub sector_id: Uuid,
    pub expires_at_ms: u64,
}

pub struct PortalService {
    active_tokens: std::sync::Mutex<HashMap<String, PortalToken>>,
    ttl_ms: u64,
}

impl PortalService {
    pub fn new() -> Self {
        Self {
            active_tokens: std::sync::Mutex::new(HashMap::new()),
            ttl_ms: 900 * 1000, // 15 minutes as per UI spec
        }
    }

    /// Generate a secure one-time token for a sector.
    pub fn create_token(&self, sector_id: Uuid) -> String {
        let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let token = format!("{:x}-{:x}", Uuid::new_v4().as_u128() as u64, Uuid::new_v4().as_u128() as u64);
        let portal_token = PortalToken {
            token: token.clone(),
            sector_id,
            expires_at_ms: now_ms + self.ttl_ms,
        };
        
        let mut tokens = self.active_tokens.lock().unwrap();
        tokens.insert(token.clone(), portal_token);
        
        tracing::info!("WEB PORTAL: Token generated for sector {}: {}", sector_id, token);
        token
    }

    /// Validate a token and return the associated sector ID if valid.
    pub fn validate_token(&self, token: &str) -> Option<Uuid> {
        let mut tokens = self.active_tokens.lock().unwrap();
        let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        
        // Cleanup expired tokens
        tokens.retain(|_, v| v.expires_at_ms > now_ms);

        if let Some(t) = tokens.get(token) {
            tracing::info!("WEB PORTAL: Handshake successful for token {}", token);
            return Some(t.sector_id);
        }
        
        None
    }

    pub fn revoke_token(&self, token: &str) {
        let mut tokens = self.active_tokens.lock().unwrap();
        tokens.remove(token);
        tracing::info!("WEB PORTAL: Token revoked: {}", token);
    }
}
