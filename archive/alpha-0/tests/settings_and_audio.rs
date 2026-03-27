use tos_comp::DesktopEnvironment;
use tos_comp::ui::dashboard::SettingsWidget;
use tos_comp::system::commands::CommandParser;

#[test]
fn test_settings_interaction() {
    let mut env = DesktopEnvironment::new(None);
    env.dashboard.add_widget(Box::new(SettingsWidget { audio_on: true, chirps_on: true }));
    
    // Initial state
    assert!(env.settings.audio_enabled);
    assert!(env.settings.chirps_enabled);
    
    // Toggle audio off via command parser
    CommandParser::process(&mut env, "config audio off");
    assert!(!env.settings.audio_enabled);
    assert!(!env.audio.enabled);
    
    // Tick to sync widget
    env.tick();
    
    let html = env.dashboard.render_all_html();
    assert!(html.contains("AUDIO MASTER:"));
    assert!(html.contains("OFF"));
    
    // Toggle chirps off
    CommandParser::process(&mut env, "config chirps off");
    assert!(!env.settings.chirps_enabled);
    
    env.tick();
    let html2 = env.dashboard.render_all_html();
    assert!(html2.contains("TACTILE CHIRPS:"));
    assert!(html2.contains("OFF"));
}

#[test]
fn test_audio_queue_consumption() {
    let mut env = DesktopEnvironment::new(None);
    
    // Trigger audio
    env.audio.play_sound("zoom_in");
    assert_eq!(env.audio.queue.len(), 1);
    
    // Rendering should consume and inject
    let html = env.generate_viewport_html();
    assert!(html.contains("id='audio-buffer'"));
    assert!(html.contains("data-sounds='zoom_in'"));
    
    // Queue should be empty now
    assert_eq!(env.audio.queue.len(), 0);
}
