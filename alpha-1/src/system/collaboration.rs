//! Collaboration & RBAC Implementation
//! 
//! Handles multi-user roles, permissions, and session synchronization logic.
//! Added role enforcement, following mode, and view synchronization.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use crate::{HierarchyLevel, TosState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollaborationRole {
    CoOwner,    // Full control, can manage participants and settings
    Operator,   // Can interact with apps and shells, but not manage sector settings
    Viewer,     // Read-only access to viewports and state
}

impl std::fmt::Display for CollaborationRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollaborationRole::CoOwner => write!(f, "Co-Owner"),
            CollaborationRole::Operator => write!(f, "Operator"),
            CollaborationRole::Viewer => write!(f, "Viewer"),
        }
    }
}

impl std::str::FromStr for CollaborationRole {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace("-", "").as_str() {
            "coowner" | "owner" => Ok(CollaborationRole::CoOwner),
            "operator" => Ok(CollaborationRole::Operator),
            "viewer" => Ok(CollaborationRole::Viewer),
            _ => Err(()),
        }
    }
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
    pub allow_viewport_control: bool,
    pub allow_mode_switch: bool,
    pub allow_app_calibrate: bool,
    pub allow_portal_manage: bool,
}

impl PermissionSet {
    pub fn for_role(role: CollaborationRole) -> Self {
        match role {
            CollaborationRole::CoOwner => Self {
                allow_shell_input: true,
                allow_app_launch: true,
                allow_sector_reset: true,
                allow_participant_invite: true,
                allow_viewport_control: true,
                allow_mode_switch: true,
                allow_app_calibrate: true,
                allow_portal_manage: true,
            },
            CollaborationRole::Operator => Self {
                allow_shell_input: true,
                allow_app_launch: true,
                allow_sector_reset: false,
                allow_participant_invite: false,
                allow_viewport_control: true,
                allow_mode_switch: true,
                allow_app_calibrate: true,
                allow_portal_manage: false,
            },
            CollaborationRole::Viewer => Self {
                allow_shell_input: false,
                allow_app_launch: false,
                allow_sector_reset: false,
                allow_participant_invite: false,
                allow_viewport_control: false,
                allow_mode_switch: false,
                allow_app_calibrate: false,
                allow_portal_manage: false,
            },
        }
    }

    /// Check if a specific action is allowed
    pub fn check(&self, action: PermissionAction) -> bool {
        match action {
            PermissionAction::ShellInput => self.allow_shell_input,
            PermissionAction::AppLaunch => self.allow_app_launch,
            PermissionAction::SectorReset => self.allow_sector_reset,
            PermissionAction::ParticipantInvite => self.allow_participant_invite,
            PermissionAction::ViewportControl => self.allow_viewport_control,
            PermissionAction::ModeSwitch => self.allow_mode_switch,
            PermissionAction::AppCalibrate => self.allow_app_calibrate,
            PermissionAction::PortalManage => self.allow_portal_manage,
        }
    }
}

/// Specific permission actions that can be checked
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionAction {
    ShellInput,
    AppLaunch,
    SectorReset,
    ParticipantInvite,
    ViewportControl,
    ModeSwitch,
    AppCalibrate,
    PortalManage,
}

impl std::fmt::Display for PermissionAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionAction::ShellInput => write!(f, "shell input"),
            PermissionAction::AppLaunch => write!(f, "app launch"),
            PermissionAction::SectorReset => write!(f, "sector reset"),
            PermissionAction::ParticipantInvite => write!(f, "participant invite"),
            PermissionAction::ViewportControl => write!(f, "viewport control"),
            PermissionAction::ModeSwitch => write!(f, "mode switch"),
            PermissionAction::AppCalibrate => write!(f, "app calibrate"),
            PermissionAction::PortalManage => write!(f, "portal manage"),
        }
    }
}

/// View state for synchronization in following mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewState {
    pub hierarchy_level: HierarchyLevel,
    pub sector_index: usize,
    pub hub_index: usize,
    pub viewport_index: usize,
    pub active_app_index: Option<usize>,
}

impl ViewState {
    pub fn from_state(state: &TosState) -> Self {
        let viewport = &state.viewports[state.active_viewport_index];
        let _sector = &state.sectors[viewport.sector_index];
        
        Self {
            hierarchy_level: state.current_level,
            sector_index: viewport.sector_index,
            hub_index: viewport.hub_index,
            viewport_index: state.active_viewport_index,
            active_app_index: viewport.active_app_index,
        }
    }

    pub fn apply_to_state(&self, state: &mut TosState) {
        if self.viewport_index < state.viewports.len() {
            state.active_viewport_index = self.viewport_index;
        }
        if self.sector_index < state.sectors.len() {
            state.viewports[state.active_viewport_index].sector_index = self.sector_index;
        }
        // Note: hub_index and active_app_index would need additional validation
        state.current_level = self.hierarchy_level;
    }
}

/// Following mode state
#[derive(Debug, Clone)]
pub struct FollowingMode {
    /// ID of the participant being followed
    pub host_id: Uuid,
    /// Last synchronized view state
    pub last_synced_view: Option<ViewState>,
    /// Whether following is active
    pub active: bool,
    /// Sync interval in milliseconds
    pub sync_interval_ms: u64,
    /// Last sync timestamp
    pub last_sync: std::time::Instant,
}

impl FollowingMode {
    pub fn new(host_id: Uuid) -> Self {
        Self {
            host_id,
            last_synced_view: None,
            active: true,
            sync_interval_ms: 100, // Sync every 100ms for smooth following
            last_sync: std::time::Instant::now() - std::time::Duration::from_millis(100),
        }
    }

    pub fn stop(&mut self) {
        self.active = false;
    }

    pub fn should_sync(&self) -> bool {
        self.active && self.last_sync.elapsed().as_millis() as u64 >= self.sync_interval_ms
    }

    pub fn mark_synced(&mut self, view: ViewState) {
        self.last_synced_view = Some(view);
        self.last_sync = std::time::Instant::now();
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

/// Participant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub avatar_url: Option<String>,
    pub role: CollaborationRole,
    pub cursor_position: Option<(f32, f32)>,
    pub following_host_id: Option<Uuid>,
}

#[derive(Default)]
pub struct CollaborationManager {
    /// Active invitations
    pub invitations: HashMap<String, Invitation>,
    /// Active sessions for participants
    pub sessions: HashMap<Uuid, PermissionSet>,
    /// Participant information
    pub participants: HashMap<Uuid, Participant>,
    /// Following mode states (guest_id -> FollowingMode)
    pub following_modes: HashMap<Uuid, FollowingMode>,
    /// View state history for synchronization (participant_id -> ViewState)
    pub view_states: HashMap<Uuid, ViewState>,
    /// Active network transports for participants
    pub transports: HashMap<Uuid, Box<dyn CollaborationTransport>>,
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

    /// Add a participant to the session
    pub fn add_participant(&mut self, participant: Participant, transport: Option<Box<dyn CollaborationTransport>>) {
        let permissions = PermissionSet::for_role(participant.role);
        self.sessions.insert(participant.id, permissions);
        self.participants.insert(participant.id, participant.clone());
        if let Some(t) = transport {
            self.transports.insert(participant.id, t);
        }
    }

    /// Remove a participant
    pub fn remove_participant(&mut self, participant_id: Uuid) {
        self.sessions.remove(&participant_id);
        self.participants.remove(&participant_id);
        self.following_modes.remove(&participant_id);
        self.view_states.remove(&participant_id);
        self.transports.remove(&participant_id);
    }

    /// Perform a security check for an action
    pub fn check_permission(&self, participant_id: Uuid, action: PermissionAction) -> bool {
        self.sessions.get(&participant_id)
            .map_or(false, |perms| perms.check(action))
    }

    /// Check permission with detailed error information
    pub fn check_permission_with_details(&self, participant_id: Uuid, action: PermissionAction) -> Result<(), PermissionDeniedError> {
        let participant = self.participants.get(&participant_id)
            .ok_or(PermissionDeniedError::UnknownParticipant(participant_id))?;
        
        let perms = self.sessions.get(&participant_id)
            .ok_or(PermissionDeniedError::NoSession(participant_id))?;
        
        if perms.check(action) {
            Ok(())
        } else {
            Err(PermissionDeniedError::ActionNotAllowed {
                participant: participant.name.clone(),
                role: participant.role,
                action,
            })
        }
    }

    /// Start following mode for a guest
    pub fn start_following(&mut self, guest_id: Uuid, host_id: Uuid) -> Result<(), String> {
        // Verify both participants exist
        if !self.participants.contains_key(&guest_id) {
            return Err(format!("Guest participant {} not found", guest_id));
        }
        if !self.participants.contains_key(&host_id) {
            return Err(format!("Host participant {} not found", host_id));
        }
        
        // Update guest's following status
        if let Some(guest) = self.participants.get_mut(&guest_id) {
            guest.following_host_id = Some(host_id);
        }
        
        // Create following mode
        self.following_modes.insert(guest_id, FollowingMode::new(host_id));
        
        tracing::info!("Participant {} is now following {}", guest_id, host_id);
        Ok(())
    }

    /// Stop following mode
    pub fn stop_following(&mut self, guest_id: Uuid) {
        if let Some(following) = self.following_modes.get_mut(&guest_id) {
            following.stop();
        }
        if let Some(guest) = self.participants.get_mut(&guest_id) {
            guest.following_host_id = None;
        }
        self.following_modes.remove(&guest_id);
    }

    /// Update view state for a participant (called when their view changes)
    pub fn update_view_state(&mut self, participant_id: Uuid, view: ViewState) {
        self.view_states.insert(participant_id, view);
    }

    /// Get view state for a participant
    pub fn get_view_state(&self, participant_id: Uuid) -> Option<&ViewState> {
        self.view_states.get(&participant_id)
    }

    /// Synchronize followers with their hosts
    /// Returns list of (follower_id, new_view_state) that need to be updated
    pub fn synchronize_followers(&mut self) -> Vec<(Uuid, ViewState)> {
        let mut updates = Vec::new();
        
        for (guest_id, following) in &mut self.following_modes {
            if !following.active || !following.should_sync() {
                continue;
            }
            
            // Get host's current view state
            if let Some(host_view) = self.view_states.get(&following.host_id).cloned() {
                // Check if view has changed
                let should_update = following.last_synced_view.as_ref()
                    .map(|last| last.hierarchy_level != host_view.hierarchy_level
                        || last.sector_index != host_view.sector_index
                        || last.active_app_index != host_view.active_app_index)
                    .unwrap_or(true);
                
                if should_update {
                    updates.push((*guest_id, host_view.clone()));
                    following.mark_synced(host_view);
                }
            }
        }
        
        updates
    }

    /// Get all active participants
    pub fn get_active_participants(&self) -> Vec<&Participant> {
        self.participants.values().collect()
    }

    /// Get participants by role
    pub fn get_participants_by_role(&self, role: CollaborationRole) -> Vec<&Participant> {
        self.participants.values()
            .filter(|p| p.role == role)
            .collect()
    }

    /// Check if a participant is following someone
    pub fn is_following(&self, participant_id: Uuid) -> bool {
        self.following_modes.get(&participant_id)
            .map(|f| f.active)
            .unwrap_or(false)
    }

    /// Get who a participant is following
    pub fn get_following_host(&self, participant_id: Uuid) -> Option<Uuid> {
        self.participants.get(&participant_id)
            .and_then(|p| p.following_host_id)
    }

    /// Enforce role-based access control for an action
    /// Returns true if allowed, logs and returns false if denied
    pub fn enforce_action(&self, participant_id: Uuid, action: PermissionAction) -> bool {
        match self.check_permission_with_details(participant_id, action) {
            Ok(()) => true,
            Err(e) => {
                tracing::warn!("Permission denied: {}", e);
                false
            }
        }
    }

    /// Poll all transports for new packets
    pub async fn poll_transports(&self) -> Vec<(Uuid, crate::system::remote::SyncPacket)> {
        let mut updates = Vec::new();
        for (id, transport) in &self.transports {
            if let Ok(Some(packet)) = transport.receive_packet().await {
                updates.push((*id, packet));
            }
        }
        updates
    }

    /// Broadcast a packet to all other participants
    pub async fn broadcast_packet(&self, sender_id: Uuid, packet: &crate::system::remote::SyncPacket) {
        for (id, transport) in &self.transports {
            if *id != sender_id {
                let _ = transport.send_packet(packet).await;
            }
        }
    }
}

/// Detailed permission denial errors
#[derive(Debug, Clone)]
pub enum PermissionDeniedError {
    UnknownParticipant(Uuid),
    NoSession(Uuid),
    ActionNotAllowed {
        participant: String,
        role: CollaborationRole,
        action: PermissionAction,
    },
}

impl std::fmt::Display for PermissionDeniedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionDeniedError::UnknownParticipant(id) => {
                write!(f, "Unknown participant: {}", id)
            }
            PermissionDeniedError::NoSession(id) => {
                write!(f, "No active session for participant: {}", id)
            }
            PermissionDeniedError::ActionNotAllowed { participant, role, action } => {
                write!(f, "Participant '{}' with role '{}' is not allowed to perform: {}",
                    participant, role.as_str(), action)
            }
        }
    }
}

impl std::error::Error for PermissionDeniedError {}

/// Trait for collaboration transport (WebSocket, WebRTC, or TCP)
#[async_trait::async_trait]
pub trait CollaborationTransport: std::fmt::Debug + Send + Sync {
    async fn send_packet(&self, packet: &crate::system::remote::SyncPacket) -> Result<(), String>;
    async fn receive_packet(&self) -> Result<Option<crate::system::remote::SyncPacket>, String>;
}

#[derive(Debug)]
pub struct MockTransport {
    pub packets: std::sync::Arc<std::sync::Mutex<Vec<crate::system::remote::SyncPacket>>>,
}

#[async_trait::async_trait]
impl CollaborationTransport for MockTransport {
    async fn send_packet(&self, packet: &crate::system::remote::SyncPacket) -> Result<(), String> {
        self.packets.lock().unwrap().push(packet.clone());
        Ok(())
    }
    
    async fn receive_packet(&self) -> Result<Option<crate::system::remote::SyncPacket>, String> {
        let mut guard = self.packets.lock().unwrap();
        if guard.is_empty() {
            Ok(None)
        } else {
            Ok(Some(guard.remove(0)))
        }
    }
}

impl std::fmt::Debug for CollaborationManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollaborationManager")
            .field("invitations_count", &self.invitations.len())
            .field("active_sessions", &self.sessions.len())
            .field("participants", &self.participants.len())
            .field("following_modes", &self.following_modes.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_collaboration_role_from_str() {
        assert_eq!(CollaborationRole::from_str("co-owner").unwrap(), CollaborationRole::CoOwner);
        assert_eq!(CollaborationRole::from_str("operator").unwrap(), CollaborationRole::Operator);
        assert_eq!(CollaborationRole::from_str("viewer").unwrap(), CollaborationRole::Viewer);
        assert!(CollaborationRole::from_str("unknown").is_err());
    }

    #[test]
    fn test_permission_set_check() {
        let co_owner = PermissionSet::for_role(CollaborationRole::CoOwner);
        let operator = PermissionSet::for_role(CollaborationRole::Operator);
        let viewer = PermissionSet::for_role(CollaborationRole::Viewer);

        // CoOwner has all permissions
        assert!(co_owner.check(PermissionAction::ShellInput));
        assert!(co_owner.check(PermissionAction::SectorReset));
        assert!(co_owner.check(PermissionAction::ParticipantInvite));

        // Operator has limited permissions
        assert!(operator.check(PermissionAction::ShellInput));
        assert!(!operator.check(PermissionAction::SectorReset));
        assert!(!operator.check(PermissionAction::ParticipantInvite));

        // Viewer has no permissions
        assert!(!viewer.check(PermissionAction::ShellInput));
        assert!(!viewer.check(PermissionAction::AppLaunch));
        assert!(!viewer.check(PermissionAction::ViewportControl));
    }

    #[test]
    fn test_following_mode() {
        let host_id = Uuid::new_v4();
        let mut following = FollowingMode::new(host_id);
        
        assert!(following.active);
        assert!(following.should_sync()); // First sync should be immediate
        
        following.mark_synced(ViewState {
            hierarchy_level: HierarchyLevel::GlobalOverview,
            sector_index: 0,
            hub_index: 0,
            viewport_index: 0,
            active_app_index: None,
        });
        
        assert!(!following.should_sync()); // Just synced
        
        following.stop();
        assert!(!following.active);
    }

    #[test]
    fn test_permission_denied_error() {
        let err = PermissionDeniedError::ActionNotAllowed {
            participant: "TestUser".to_string(),
            role: CollaborationRole::Viewer,
            action: PermissionAction::ShellInput,
        };
        
        let msg = err.to_string();
        assert!(msg.contains("TestUser"));
        assert!(msg.contains("Viewer"));
        assert!(msg.contains("shell input"));
    }

    #[test]
    fn test_collaboration_manager_add_remove_participant() {
        let mut manager = CollaborationManager::new();
        let participant = Participant {
            id: Uuid::new_v4(),
            name: "TestUser".to_string(),
            color: "#ff0000".to_string(),
            avatar_url: None,
            role: CollaborationRole::Operator,
            cursor_position: None,
            following_host_id: None,
        };
        
        manager.add_participant(participant.clone(), None);
        assert!(manager.participants.contains_key(&participant.id));
        assert!(manager.sessions.contains_key(&participant.id));
        
        manager.remove_participant(participant.id);
        assert!(!manager.participants.contains_key(&participant.id));
        assert!(!manager.sessions.contains_key(&participant.id));
    }

    #[test]
    fn test_start_stop_following() {
        let mut manager = CollaborationManager::new();
        let host_id = Uuid::new_v4();
        let guest_id = Uuid::new_v4();
        
        // Add participants
        manager.add_participant(Participant {
            id: host_id,
            name: "Host".to_string(),
            color: "#ff0000".to_string(),
            avatar_url: None,
            role: CollaborationRole::CoOwner,
            cursor_position: None,
            following_host_id: None,
        }, None);
        manager.add_participant(Participant {
            id: guest_id,
            name: "Guest".to_string(),
            color: "#00ff00".to_string(),
            avatar_url: None,
            role: CollaborationRole::Viewer,
            cursor_position: None,
            following_host_id: None,
        }, None);
        
        // Start following
        assert!(manager.start_following(guest_id, host_id).is_ok());
        assert!(manager.is_following(guest_id));
        assert_eq!(manager.get_following_host(guest_id), Some(host_id));
        
        // Stop following
        manager.stop_following(guest_id);
        assert!(!manager.is_following(guest_id));
        assert!(manager.get_following_host(guest_id).is_none());
    }

    #[test]
    fn test_synchronize_followers() {
        let mut manager = CollaborationManager::new();
        let host_id = Uuid::new_v4();
        let guest_id = Uuid::new_v4();
        
        // Add participants
        manager.add_participant(Participant {
            id: host_id,
            name: "Host".to_string(),
            color: "#ff0000".to_string(),
            avatar_url: None,
            role: CollaborationRole::CoOwner,
            cursor_position: None,
            following_host_id: None,
        }, None);
        manager.add_participant(Participant {
            id: guest_id,
            name: "Guest".to_string(),
            color: "#00ff00".to_string(),
            avatar_url: None,
            role: CollaborationRole::Viewer,
            cursor_position: None,
            following_host_id: None,
        }, None);
        
        // Set up following
        manager.start_following(guest_id, host_id).unwrap();
        
        // Set host's view state
        let host_view = ViewState {
            hierarchy_level: HierarchyLevel::CommandHub,
            sector_index: 1,
            hub_index: 0,
            viewport_index: 0,
            active_app_index: Some(0),
        };
        manager.update_view_state(host_id, host_view.clone());
        
        // Synchronize
        let updates = manager.synchronize_followers();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].0, guest_id);
        assert_eq!(updates[0].1.sector_index, 1);
    }

    #[test]
    fn test_enforce_action_logging() {
        let mut manager = CollaborationManager::new();
        let participant_id = Uuid::new_v4();
        
        // Add viewer participant
        manager.add_participant(Participant {
            id: participant_id,
            name: "ViewerUser".to_string(),
            color: "#0000ff".to_string(),
            avatar_url: None,
            role: CollaborationRole::Viewer,
            cursor_position: None,
            following_host_id: None,
        }, None);
        
        // Should deny shell input
        assert!(!manager.enforce_action(participant_id, PermissionAction::ShellInput));
        
        // Should allow for co-owner
        let co_owner_id = Uuid::new_v4();
        manager.add_participant(Participant {
            id: co_owner_id,
            name: "Owner".to_string(),
            color: "#ff0000".to_string(),
            avatar_url: None,
            role: CollaborationRole::CoOwner,
            cursor_position: None,
            following_host_id: None,
        }, None);
        assert!(manager.enforce_action(co_owner_id, PermissionAction::ShellInput));
    }
}
