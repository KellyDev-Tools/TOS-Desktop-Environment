use crate::state::TosState;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Service for recording and navigating the chronological history of system states (§19.1).
///
/// Provides the "scrubbing" capability for Level 4 (Detail/Timeline) views.
pub struct TimelineService {
    history: Arc<Mutex<VecDeque<TosState>>>,
    max_entries: usize,
}

impl TimelineService {
    /// Create a new timeline service with a fixed-size ring buffer for history.
    pub fn new(max_entries: usize) -> Self {
        Self {
            history: Arc::new(Mutex::new(VecDeque::with_capacity(max_entries))),
            max_entries,
        }
    }

    /// Record a point-in-time snapshot of the system state.
    pub fn record_snapshot(&self, state: &TosState) {
        let mut history = self.history.lock().unwrap();
        
        // Don't record if we're currently scrubbing (timeline_cursor is set)
        if state.timeline_cursor.is_some() {
            return;
        }

        if history.len() >= self.max_entries {
            history.pop_front();
        }
        
        // Strip timeline-specific fields from the archived state to avoid recursion/confusion
        let mut archived = state.clone();
        archived.timeline_cursor = None;
        archived.timeline_history_len = 0;
        
        history.push_back(archived);
    }

    /// Retrieve a snapshot by index.
    pub fn get_snapshot(&self, index: usize) -> Option<TosState> {
        let history = self.history.lock().unwrap();
        history.get(index).cloned()
    }

    /// Number of recorded entries in the timeline.
    pub fn len(&self) -> usize {
        self.history.lock().unwrap().len()
    }

    /// Clear all history.
    pub fn clear(&self) {
        self.history.lock().unwrap().clear();
    }
}
