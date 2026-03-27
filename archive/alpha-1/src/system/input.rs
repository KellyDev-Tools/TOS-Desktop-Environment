// Advanced Input submodules
pub mod advanced;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticEvent {
    // Navigation
    ZoomIn,
    ZoomOut,
    NextElement,
    PrevElement,
    NextViewport,
    
    // Selection
    Select,
    SecondarySelect,
    MultiSelectToggle,
    
    // Mode Control
    CycleMode,
    ModeCommand,
    ModeDirectory,
    ModeActivity,
    ModeSearch,
    ModeAi,
    ToggleHiddenFiles,
    
    // Bezel & Layout
    ToggleBezel,
    SplitViewport,
    CloseViewport,
    
    // System (Section 14: Tactical Reset)
    /// Level 1: Sector reset (Super+Backspace)
    TacticalReset,
    /// Level 2: System reset dialog (Super+Alt+Backspace)
    SystemReset,
    OpenGlobalOverview,
    
    // Text
    SubmitPrompt,
    HistoryPrev,
    HistoryNext,

    // Voice
    VoiceCommandStart,

    // AI (§9.1)
    AiSubmit,
    AiStop,
    AiModeToggle,

    // Collaboration (§9.1)
    RaiseHand,

    // Operations (§9.1)
    StopOperation,

    // UI State
    ToggleMiniMap,
    ToggleComms,
}

#[cfg(feature = "gamepad")]
pub fn poll_gamepad(state: std::sync::Arc<std::sync::Mutex<crate::TosState>>) {
    use std::thread;
    use std::time::Duration;
    use gilrs::{Gilrs, Event as GilrsEvent, Button};

    thread::spawn(move || {
        let mut gilrs = Gilrs::new().expect("Failed to initialize Gilrs");
        loop {
            while let Some(GilrsEvent { event, .. }) = gilrs.next_event() {
                let mut state_lock = state.lock().unwrap();
                match event {
                    gilrs::EventType::ButtonPressed(button, _) => {
                        match button {
                            Button::South => state_lock.handle_semantic_event(SemanticEvent::ZoomIn),
                            Button::East => state_lock.handle_semantic_event(SemanticEvent::ZoomOut),
                            Button::North => state_lock.handle_semantic_event(SemanticEvent::TacticalReset),
                            Button::West => state_lock.handle_semantic_event(SemanticEvent::CycleMode),
                            Button::LeftTrigger => state_lock.handle_semantic_event(SemanticEvent::ModeCommand),
                            Button::RightTrigger => state_lock.handle_semantic_event(SemanticEvent::ModeDirectory),
                            Button::Select => state_lock.handle_semantic_event(SemanticEvent::ModeActivity),
                            Button::Start => state_lock.handle_semantic_event(SemanticEvent::ToggleBezel),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    });
}
