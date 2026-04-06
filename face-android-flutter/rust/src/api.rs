use tos_common::state::{TosState, HierarchyLevel};

mod frb_generated;

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Standard Flutter-Rust-Bridge 2.0 init
    flutter_rust_bridge::setup_default_user_utils();
}

pub fn greet(name: String) -> String {
    format!("Hello, {name}! TOS is ready for your Android Face.")
}

pub fn get_tos_status() -> String {
    "TOS System: ONLINE\nProtocol: Flutter/Rust Bridge\nUI: Premium Glassmorphism".to_string()
}

pub fn get_initial_state() -> TosState {
    TosState::default()
}

pub fn set_hierarchy_level(state: &mut TosState, level: i32) {
    state.current_level = match level {
        1 => HierarchyLevel::GlobalOverview,
        2 => HierarchyLevel::CommandHub,
        3 => HierarchyLevel::ApplicationFocus,
        4 => HierarchyLevel::DetailView,
        5 => HierarchyLevel::BufferView,
        6 => HierarchyLevel::Marketplace,
        _ => HierarchyLevel::GlobalOverview,
    };
}

pub fn get_system_logs(state: &TosState) -> Vec<String> {
    state.system_log.iter().map(|l| l.text.clone()).collect()
}

pub fn add_log_entry(state: &mut TosState, text: String) {
    state.system_log.push(tos_common::state::TerminalLine {
        text,
        priority: 1,
        timestamp: chrono::Local::now(),
    });
}
