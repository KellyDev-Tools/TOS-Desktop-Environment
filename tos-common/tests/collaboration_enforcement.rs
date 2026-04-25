use tos_common::collaboration::{WebRtcPayload, PresenceStatus, ParticipantRole};
use uuid::Uuid;

#[tokio::test]
async fn test_remote_command_role_enforcement() -> anyhow::Result<()> {
    // 1. Initialize Brain (Desktop mock/real)
    let brain = tos_common::brain::Brain::new()?;
    let handler = brain.ipc.clone();
    let state = brain.state.clone();

    let user_id = Uuid::new_v4();
    let sector_id = {
        let s = state.lock().unwrap();
        s.sectors[0].id
    };
    
    // 2. Setup a Viewer participant
    {
        let mut s = state.lock().unwrap();
        let sector = &mut s.sectors[0];
        sector.participants.push(tos_common::collaboration::Participant {
            id: user_id,
            alias: "Guest".to_string(),
            status: PresenceStatus::Active,
            role: ParticipantRole::Viewer,
            current_level: 1,
            viewport_title: None,
            left_chip_state: None,
            right_chip_state: None,
            cursor_x: None,
            cursor_y: None,
            cursor_target: None,
            following: None,
        });
    }

    // 3. Viewer attempts to run a command (rejected)
    let payload = WebRtcPayload::Command {
        user: user_id,
        request: "prompt_submit:ls".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    handler.handle_request(&format!("webrtc_presence:{}", json));

    // Verify command was NOT executed
    {
        let s = state.lock().unwrap();
        let hub = &s.sectors[0].hubs[0];
        assert!(!hub.is_running, "Viewer should not be able to start command");
    }

    // 4. Promote to Operator (requires Admin/CoOwner, but local IPC is trusted)
    handler.handle_request(&format!("collaboration_role_set:{};{};operator", sector_id, user_id));

    // 5. Operator attempts to run command (accepted)
    handler.handle_request(&format!("webrtc_presence:{}", json));

    // 6. Verify command WAS executed
    {
        let s = state.lock().unwrap();
        let hub = &s.sectors[0].hubs[0];
        assert!(hub.is_running, "Operator should be able to start command");
    }

    // 7. Test Admin/CoOwner restriction (Operator cannot promote others)
    let other_user = Uuid::new_v4();
    let promotion_req = WebRtcPayload::Command {
        user: user_id,
        request: format!("collaboration_role_set:{};{};coowner", sector_id, other_user),
    };
    let promotion_json = serde_json::to_string(&promotion_req).unwrap();
    handler.handle_request(&format!("webrtc_presence:{}", promotion_json));

    // Verify other_user is NOT a CoOwner (should not even exist in participants yet, 
    // or if we added them, role shouldn't be CoOwner)
    {
        let s = state.lock().unwrap();
        let sector = &s.sectors[0];
        if let Some(p) = sector.participants.iter().find(|p| p.id == other_user) {
            assert_ne!(p.role, ParticipantRole::CoOwner);
        }
    }

    Ok(())
}
