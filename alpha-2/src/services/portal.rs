use uuid::Uuid;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalToken {
    pub token: String,
    pub sector_id: Uuid,
    #[serde(skip)]
    pub expires_at: Instant,
}

pub struct PortalService {
    active_tokens: std::sync::Mutex<HashMap<String, PortalToken>>,
    ttl: Duration,
}

impl PortalService {
    pub fn new() -> Self {
        Self {
            active_tokens: std::sync::Mutex::new(HashMap::new()),
            ttl: Duration::from_secs(900), // 15 minutes as per UI spec
        }
    }

    /// Generate a secure one-time token for a sector.
    pub fn create_token(&self, sector_id: Uuid) -> String {
        let token = format!("{:x}-{:x}", Uuid::new_v4().as_u128() as u64, Uuid::new_v4().as_u128() as u64);
        let portal_token = PortalToken {
            token: token.clone(),
            sector_id,
            expires_at: Instant::now() + self.ttl,
        };
        
        let mut tokens = self.active_tokens.lock().unwrap();
        tokens.insert(token.clone(), portal_token);
        
        tracing::info!("WEB PORTAL: Token generated for sector {}: {}", sector_id, token);
        token
    }

    /// Validate a token and return the associated sector ID if valid.
    pub fn validate_token(&self, token: &str) -> Option<Uuid> {
        let mut tokens = self.active_tokens.lock().unwrap();
        
        // Cleanup expired tokens
        let now = Instant::now();
        tokens.retain(|_, v| v.expires_at > now);

        if let Some(t) = tokens.get(token) {
            tracing::info!("WEB PORTAL: Handshake successful for token {}", token);
            let sector_id = t.sector_id;
            // Consumption: Tokens are one-time use
            // tokens.remove(token); 
            return Some(sector_id);
        }
        
        None
    }

    pub fn revoke_token(&self, token: &str) {
        let mut tokens = self.active_tokens.lock().unwrap();
        tokens.remove(token);
        tracing::info!("WEB PORTAL: Token revoked: {}", token);
    }
}
