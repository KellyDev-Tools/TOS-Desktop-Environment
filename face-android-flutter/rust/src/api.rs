pub use tos_common::state::{TosState, HierarchyLevel, TerminalLine};
use tos_common::platform::SemanticEvent as TosSemanticEvent;

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Standard Flutter-Rust-Bridge 2.0 init
    flutter_rust_bridge::setup_default_user_utils();
}

pub fn greet(name: String) -> String {
    format!("Hello, {name}! TOS is ready for your Android Face.")
}

pub fn get_tos_status() -> String {
    "TOS System: ONLINE\nProtocol: Flutter/Rust Bridge\nUI: Premium Amber LCARS".to_string()
}

pub fn get_initial_state() -> TosState {
    TosState::default()
}

/// A Flutter-compatible version of SemanticEvent for the bridge.
pub enum AndroidSemanticEvent {
    Home,
    CommandHub,
    ZoomIn,
    ZoomOut,
    ToggleBezel,
    SetHierarchy(i32),
}

/// §14.1: Handle a SemanticEvent in the Face.
pub fn handle_semantic_event(state: &mut TosState, event: AndroidSemanticEvent) {
    let tos_event = match event {
        AndroidSemanticEvent::Home => TosSemanticEvent::Home,
        AndroidSemanticEvent::CommandHub => TosSemanticEvent::CommandHub,
        AndroidSemanticEvent::ZoomIn => TosSemanticEvent::ZoomIn,
        AndroidSemanticEvent::ZoomOut => TosSemanticEvent::ZoomOut,
        AndroidSemanticEvent::ToggleBezel => TosSemanticEvent::ToggleBezel,
        AndroidSemanticEvent::SetHierarchy(l) => {
            state.current_level = match l {
                1 => HierarchyLevel::GlobalOverview,
                2 => HierarchyLevel::CommandHub,
                3 => HierarchyLevel::ApplicationFocus,
                4 => HierarchyLevel::DetailView,
                5 => HierarchyLevel::BufferView,
                6 => HierarchyLevel::Marketplace,
                _ => state.current_level,
            };
            TosSemanticEvent::Select(format!("LEVEL_{}", l))
        }
    };

    // Internal handling logic (simulated for now)
    match tos_event {
        TosSemanticEvent::Home => state.current_level = HierarchyLevel::GlobalOverview,
        TosSemanticEvent::CommandHub => state.current_level = HierarchyLevel::CommandHub,
        TosSemanticEvent::ZoomIn => {
            state.current_level = match state.current_level {
                HierarchyLevel::GlobalOverview => HierarchyLevel::CommandHub,
                HierarchyLevel::CommandHub => HierarchyLevel::ApplicationFocus,
                HierarchyLevel::ApplicationFocus => HierarchyLevel::DetailView,
                HierarchyLevel::DetailView => HierarchyLevel::BufferView,
                _ => state.current_level,
            }
        }
        TosSemanticEvent::ZoomOut => {
            state.current_level = match state.current_level {
                HierarchyLevel::BufferView => HierarchyLevel::DetailView,
                HierarchyLevel::DetailView => HierarchyLevel::ApplicationFocus,
                HierarchyLevel::ApplicationFocus => HierarchyLevel::CommandHub,
                HierarchyLevel::CommandHub => HierarchyLevel::GlobalOverview,
                _ => state.current_level,
            }
        }
        _ => {}
    }
    
    add_log_entry(state, format!("UI_ACTION: Semantic Event triggered: {:?}", tos_event));
}

pub fn get_system_logs(state: &TosState) -> Vec<String> {
    state.system_log.iter().map(|l| l.text.clone()).collect()
}

pub fn add_log_entry(state: &mut TosState, text: String) {
    state.system_log.push(TerminalLine {
        text,
        priority: 1,
        timestamp: chrono::Local::now(),
    });
}
