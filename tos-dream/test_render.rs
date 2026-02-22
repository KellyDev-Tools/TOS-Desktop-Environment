use std::sync::{Arc, Mutex};
use tos_core::TosState;

fn main() {
    let mut s = TosState::new_fresh();
    s.sectors.truncate(1);
    s.current_level = tos_core::HierarchyLevel::CommandHub;
    s.viewports[0].current_level = tos_core::HierarchyLevel::CommandHub;
    
    let html = s.render_current_view();
    println!("Rendered bytes: {}", html.len());
}
