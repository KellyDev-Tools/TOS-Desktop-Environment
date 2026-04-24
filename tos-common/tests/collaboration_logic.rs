use tos_common::collaboration::*;
use uuid::Uuid;

#[test]
fn test_presence_payload_serde() {
    let payload = WebRtcPayload::Presence {
        user: Uuid::new_v4(),
        status: PresenceStatus::Active,
        level: 2,
        active_viewport_title: Some("Terminal".to_string()),
        left_chip_state: None,
        right_chip_state: None,
    };
    let serialized = serde_json::to_string(&payload).unwrap();
    let deserialized: WebRtcPayload = serde_json::from_str(&serialized).unwrap();
    if let WebRtcPayload::Presence { status, level, .. } = deserialized {
        assert_eq!(status, PresenceStatus::Active);
        assert_eq!(level, 2);
    } else {
        panic!("Wrong variant");
    }
}

#[test]
fn test_role_change_payload_serde() {
    let payload = WebRtcPayload::RoleChange {
        target: Uuid::new_v4(),
        new_role: ParticipantRole::CoOwner,
        admin: Uuid::new_v4(),
    };
    let serialized = serde_json::to_string(&payload).unwrap();
    let deserialized: WebRtcPayload = serde_json::from_str(&serialized).unwrap();
    if let WebRtcPayload::RoleChange { new_role, .. } = deserialized {
        assert_eq!(new_role, ParticipantRole::CoOwner);
    } else {
        panic!("Wrong variant");
    }
}

#[test]
fn test_participant_struct_serde() {
    let participant = Participant {
        id: Uuid::new_v4(),
        alias: "Participant 1".to_string(),
        status: PresenceStatus::Idle,
        role: ParticipantRole::Operator,
        current_level: 3,
        viewport_title: None,
        left_chip_state: None,
        right_chip_state: None,
        cursor_x: Some(0.1),
        cursor_y: Some(0.2),
        cursor_target: None,
        following: None,
    };
    let serialized = serde_json::to_string(&participant).unwrap();
    let deserialized: Participant = serde_json::from_str(&serialized).unwrap();
    assert_eq!(participant.alias, deserialized.alias);
    assert_eq!(participant.cursor_x, deserialized.cursor_x);
}
