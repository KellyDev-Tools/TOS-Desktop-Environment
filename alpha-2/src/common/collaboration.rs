use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// §13.7: WebRTC Data Channel Collaboration Payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebRtcPayload {
    Presence {
        user: Uuid,
        status: PresenceStatus,
        level: u8,
        // Active viewport state sharing
        active_viewport_title: Option<String>,
        // Dual-sided chip state synchronization
        left_chip_state: Option<String>,
        right_chip_state: Option<String>,
    },
    CursorSync {
        user: Uuid,
        x: f32, // Normalized 0.0 to 1.0
        y: f32, // Normalized 0.0 to 1.0
        target: Option<String>,
    },
    Following {
        follower: Uuid,
        leader: Uuid,
        sync: bool,
    },
    RoleChange {
        target: Uuid,
        new_role: ParticipantRole,
        admin: Uuid,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresenceStatus {
    Active,
    Idle,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParticipantRole {
    Observer,
    Operator,
    Admin,
}

/// Representation of an active guest connected to a sector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub alias: String,
    pub status: PresenceStatus,
    pub role: ParticipantRole,
    pub current_level: u8,
    pub viewport_title: Option<String>,
    pub left_chip_state: Option<String>,
    pub right_chip_state: Option<String>,
    pub cursor_x: Option<f32>,
    pub cursor_y: Option<f32>,
    pub cursor_target: Option<String>,
    pub following: Option<Uuid>, 
}
