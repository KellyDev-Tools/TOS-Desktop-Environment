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
    ToggleHiddenFiles,
    
    // Bezel & Layout
    ToggleBezel,
    SplitViewport,
    CloseViewport,
    
    // System
    TacticalReset,
    OpenGlobalOverview,
    
    // Text
    SubmitPrompt,
    HistoryPrev,
    HistoryNext,
}
