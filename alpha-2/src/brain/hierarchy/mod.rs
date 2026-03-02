use crate::common::{HierarchyLevel, TosState};

pub struct HierarchyManager;

impl HierarchyManager {
    /// TOC §5: Transition logic between levels
    pub fn zoom_in(state: &mut TosState) -> bool {
        let next_level = match state.current_level {
            HierarchyLevel::GlobalOverview => Some(HierarchyLevel::CommandHub),
            HierarchyLevel::CommandHub => Some(HierarchyLevel::ApplicationFocus),
            HierarchyLevel::ApplicationFocus => Some(HierarchyLevel::DetailView),
            HierarchyLevel::DetailView => {
                // §17.4: Deep Inspection requires privilege check
                let sid = state.sectors.get(state.active_sector_index).map(|s| s.id.to_string());
                if state.settings.resolve("deep_inspection", sid.as_deref(), None) == Some("true".to_string()) {
                    Some(HierarchyLevel::BufferView)
                } else {
                    tracing::warn!("Deep Inspection (Level 5) denied: Privilege escalation required.");
                    None
                }
            }
            HierarchyLevel::BufferView => None,
        };

        if let Some(level) = next_level {
            state.current_level = level;
            true
        } else {
            false
        }
    }

    pub fn zoom_out(state: &mut TosState) -> bool {
        let prev_level = match state.current_level {
            HierarchyLevel::BufferView => Some(HierarchyLevel::DetailView),
            HierarchyLevel::DetailView => Some(HierarchyLevel::ApplicationFocus),
            HierarchyLevel::ApplicationFocus => Some(HierarchyLevel::CommandHub),
            HierarchyLevel::CommandHub => Some(HierarchyLevel::GlobalOverview),
            HierarchyLevel::GlobalOverview => None,
        };

        if let Some(level) = prev_level {
            state.current_level = level;
            true
        } else {
            false
        }
    }

    pub fn set_level(state: &mut TosState, level: HierarchyLevel) -> bool {
        if level == HierarchyLevel::BufferView {
             let sid = state.sectors.get(state.active_sector_index).map(|s| s.id.to_string());
             if state.settings.resolve("deep_inspection", sid.as_deref(), None) != Some("true".to_string()) {
                tracing::warn!("Direct transition to Level 5 denied.");
                return false;
             }
        }
        state.current_level = level;
        true
    }
}
