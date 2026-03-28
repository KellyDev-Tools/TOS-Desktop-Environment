//! Multi-user collaboration payloads for WebRTC data channel communication.
//!
//! These types are shared between the Brain (which brokers collaboration
//! state) and any Face (which renders participant cursors, presence, etc.).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// WebRTC data channel message variants for real-time collaboration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebRtcPayload {
    /// Presence heartbeat — broadcasts a participant's current state.
    Presence {
        user: Uuid,
        status: PresenceStatus,
        level: u8,
        /// Active viewport title for other participants to see.
        active_viewport_title: Option<String>,
        /// Left chip column synchronization state.
        left_chip_state: Option<String>,
        /// Right chip column synchronization state.
        right_chip_state: Option<String>,
    },
    /// Cursor position synchronization (normalized 0.0–1.0 coordinates).
    CursorSync {
        user: Uuid,
        x: f32,
        y: f32,
        target: Option<String>,
    },
    /// Follow-mode binding between two participants.
    Following {
        follower: Uuid,
        leader: Uuid,
        sync: bool,
    },
    /// Administrative role change for a participant.
    RoleChange {
        target: Uuid,
        new_role: ParticipantRole,
        admin: Uuid,
    },
}

/// Connection status for a collaboration participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresenceStatus {
    Active,
    Idle,
    Offline,
}

/// Permission role for a collaboration participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParticipantRole {
    Observer,
    Operator,
    Admin,
}

/// An active guest connected to a sector via WebRTC.
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
