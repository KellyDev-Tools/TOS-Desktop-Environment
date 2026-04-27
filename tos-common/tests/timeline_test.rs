use tos_common::state::TosState;
use tos_common::services::timeline::TimelineService;

#[test]
fn test_timeline_recording_and_scrubbing() {
    let timeline = TimelineService::new(10);
    let mut state = TosState::default();
    
    // 1. Record first snapshot
    state.sys_status = "T1".to_string();
    timeline.record_snapshot(&state);
    
    // 2. Record second snapshot
    state.sys_status = "T2".to_string();
    timeline.record_snapshot(&state);
    
    assert_eq!(timeline.len(), 2);
    
    // 3. Retrieve first snapshot
    let s1 = timeline.get_snapshot(0).unwrap();
    assert_eq!(s1.sys_status, "T1");
    
    // 4. Test scrubbing logic (simulation)
    let index = 0;
    if let Some(snapshot) = timeline.get_snapshot(index) {
        let mut current_state = state.clone();
        current_state = snapshot;
        current_state.timeline_cursor = Some(index);
        
        assert_eq!(current_state.sys_status, "T1");
        assert_eq!(current_state.timeline_cursor, Some(0));
        
        // 5. Verify recording is paused during scrub
        timeline.record_snapshot(&current_state);
        assert_eq!(timeline.len(), 2); // Still 2
    }
}
