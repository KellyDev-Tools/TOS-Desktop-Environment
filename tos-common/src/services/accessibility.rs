use std::sync::{Arc, Mutex};
use crate::state::{TosState, ScanningMode};
use crate::ipc::IpcDispatcher;

pub struct AccessibilityService {
    state: Arc<Mutex<Option<Arc<Mutex<TosState>>>>>,
    ipc: Arc<Mutex<Option<Arc<dyn IpcDispatcher>>>>,
}

impl AccessibilityService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(None)),
            ipc: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_state(&self, state: Arc<Mutex<TosState>>) {
        let mut lock = self.state.lock().unwrap();
        *lock = Some(state);
    }

    pub fn set_ipc(&self, ipc: Arc<dyn IpcDispatcher>) {
        let mut lock = self.ipc.lock().unwrap();
        *lock = Some(ipc);
    }

    /// Toggle switch scanning on/off.
    pub fn toggle_scanning(&self) {
        let state_arc = self.state.lock().unwrap().clone();
        if let Some(state_lock) = state_arc {
            let mut state = state_lock.lock().unwrap();
            state.accessibility.scanning_enabled = !state.accessibility.scanning_enabled;
            state.version += 1;
            
            if state.accessibility.scanning_enabled {
                self.refresh_scan_path(&mut state);
            }
        }
    }

    /// Set scanning mode (Auto vs Manual).
    pub fn set_scanning_mode(&self, mode: ScanningMode) {
        let state_arc = self.state.lock().unwrap().clone();
        if let Some(state_lock) = state_arc {
            let mut state = state_lock.lock().unwrap();
            state.accessibility.scanning_mode = mode;
            state.version += 1;
        }
    }

    /// Advance the scanning focus to the next element.
    pub fn advance_scan(&self) {
        let state_arc = self.state.lock().unwrap().clone();
        if let Some(state_lock) = state_arc {
            let mut state = state_lock.lock().unwrap();
            if !state.accessibility.scanning_enabled || state.accessibility.active_scan_path.is_empty() {
                return;
            }

            state.accessibility.current_scan_index = (state.accessibility.current_scan_index + 1) % state.accessibility.active_scan_path.len();
            state.version += 1;
        }
    }

    /// Select the currently highlighted element.
    pub fn select_current(&self) -> String {
        let state_arc = self.state.lock().unwrap().clone();
        if let Some(state_lock) = state_arc {
            let (path, index) = {
                let state = state_lock.lock().unwrap();
                if !state.accessibility.scanning_enabled || state.accessibility.active_scan_path.is_empty() {
                    return "ERROR: Scanning not active".to_string();
                }
                (state.accessibility.active_scan_path.clone(), state.accessibility.current_scan_index)
            };

            if let Some(target) = path.get(index) {
                // Dispatch the selection as an IPC command if it maps to a command
                // For now, we'll support "sector:N", "hub:N", "level:NAME"
                if let Some(ipc) = &*self.ipc.lock().unwrap() {
                    if target.starts_with("sector:") {
                        let idx = target.strip_prefix("sector:").unwrap();
                        return ipc.dispatch(&format!("set_active_sector:{}", idx));
                    } else if target.starts_with("level:") {
                        let level = target.strip_prefix("level:").unwrap();
                        return ipc.dispatch(&format!("set_mode:{}", level));
                    }
                }
                return format!("SELECTED: {}", target);
            }
        }

        "ERROR: Selection failed".to_string()
    }

    /// Re-calculate the scan path based on the current UI hierarchy.
    pub fn refresh_scan_path(&self, state: &mut TosState) {
        let mut path = vec![];

        // 1. Level Navigation (Global, Hub, App, Detail)
        path.push("level:global".to_string());
        path.push("level:hubs".to_string());
        path.push("level:sectors".to_string());
        path.push("level:detail".to_string());

        // 2. Sectors (if at Global Overview)
        if state.current_level == crate::state::HierarchyLevel::GlobalOverview {
            for i in 0..state.sectors.len() {
                path.push(format!("sector:{}", i));
            }
        }

        // 3. Hubs (if in a sector)
        // TODO: Add Hubs/Apps to scan path

        state.accessibility.active_scan_path = path;
        state.accessibility.current_scan_index = 0;
        state.version += 1;
    }
}
