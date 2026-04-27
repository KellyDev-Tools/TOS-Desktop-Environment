use tos_common::services::audio::AudioService;

#[test]
fn test_spatial_audio_trigger() {
    let (audio_svc, _) = AudioService::new();
    
    // This will send the PlaySpatialEarcon command to the background thread
    audio_svc.play_spatial_earcon("system_ready", 1.0, 0.0, 0.0);
    
    // We can't easily verify the actual audio output in this environment,
    // but the test confirms the pipeline doesn't panic.
}
