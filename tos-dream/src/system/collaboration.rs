//! Phase 12: Collaboration & RBAC Implementation
//! 
//! Handles multi-user roles, permissions, and session synchronization logic.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollaborationRole {
    CoOwner,    // Full control, can manage participants and settings
    Operator,   // Can interact with apps and shells, but not manage sector settings
    Viewer,     // Read-only access to viewports and state
}

impl CollaborationRole {
    pub fn can_interact(&self) -> bool {
        match self {
            CollaborationRole::CoOwner | CollaborationRole::Operator => true,
            CollaborationRole::Viewer => false,
        }
    }

    pub fn can_manage(&self) -> bool {
        matches!(self, CollaborationRole::CoOwner)
    }

    pub fn as_str(&self) -> &str {
        match self {
            CollaborationRole::CoOwner => "Co-owner",
            CollaborationRole::Operator => "Operator",
            CollaborationRole::Viewer => "Viewer",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    pub allow_shell_input: bool,
    pub allow_app_launch: bool,
    pub allow_sector_reset: bool,
    pub allow_participant_invite: bool,
}

impl PermissionSet {
    pub fn for_role(role: CollaborationRole) -> Self {
        match role {
            CollaborationRole::CoOwner => Self {
                allow_shell_input: true,
                allow_app_launch: true,
                allow_sector_reset: true,
                allow_participant_invite: true,
            },
            CollaborationRole::Operator => Self {
                allow_shell_input: true,
                allow_app_launch: true,
                allow_sector_reset: false,
                allow_participant_invite: false,
            },
            CollaborationRole::Viewer => Self {
                allow_shell_input: false,
                allow_app_launch: false,
                allow_sector_reset: false,
                allow_participant_invite: false,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    pub id: Uuid,
    pub sector_id: Uuid,
    pub role: CollaborationRole,
    pub token: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub used: bool,
}

#[derive(Default)]
pub struct CollaborationManager {
    /// Active invitations
    pub invitations: HashMap<String, Invitation>,
    /// Active sessions for participants
    pub sessions: HashMap<Uuid, PermissionSet>,
}

impl CollaborationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new invitation for a sector
    pub fn create_invitation(&mut self, sector_id: Uuid, role: CollaborationRole) -> String {
        let token = Uuid::new_v4().to_string();
        let invitation = Invitation {
            id: Uuid::new_v4(),
            sector_id,
            role,
            token: token.clone(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(24)),
            used: false,
        };
        self.invitations.insert(token.clone(), invitation);
        token
    }

    /// Validate an invitation token and return the associated role/sector
    pub fn redeem_invitation(&mut self, token: &str) -> Option<(Uuid, CollaborationRole)> {
        if let Some(invitation) = self.invitations.get_mut(token) {
            if !invitation.used && invitation.expires_at.map_or(true, |exp| exp > chrono::Utc::now()) {
                invitation.used = true;
                return Some((invitation.sector_id, invitation.role));
            }
        }
        None
    }

    /// Perform a security check for an action
    pub fn check_permission(&self, participant_id: Uuid, action: impl FnOnce(&PermissionSet) -> bool) -> bool {
        self.sessions.get(&participant_id).map_or(false, action)
    }
}

impl std::fmt::Debug for CollaborationManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollaborationManager")
            .field("invitations_count", &self.invitations.len())
            .field("active_sessions", &self.sessions.len())
            .finish()
    }
}
