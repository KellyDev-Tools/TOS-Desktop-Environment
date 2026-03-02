use crate::common::{Sector, CommandHub, CommandHubMode, TosState};
use uuid::Uuid;
use std::path::PathBuf;

pub struct SectorManager;

impl SectorManager {
    /// Create a new sector with a default hub.
    pub fn create_sector(state: &mut TosState, name: String) -> Uuid {
        let sector_id = Uuid::new_v4();
        let hub_id = Uuid::new_v4();
        
        // Use current directory of active hub as starting point if possible
        let cwd = if let Some(active_sector) = state.sectors.get(state.active_sector_index) {
            active_sector.hubs[active_sector.active_hub_index].current_directory.clone()
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
        };

        let sector = Sector {
            id: sector_id,
            name,
            hubs: vec![CommandHub {
                id: hub_id,
                mode: CommandHubMode::Command,
                prompt: String::new(),
                current_directory: cwd,
                terminal_output: vec![],
                buffer_limit: 500,
                shell_listing: None,
                activity_listing: None,
                search_results: None,
                staged_command: None,
                ai_explanation: None,
                json_context: None,
                version: 0,
            }],
            active_hub_index: 0,
            frozen: false,
            is_remote: false,
            disconnected: false,
            trust_tier: crate::common::TrustTier::System,
            version: 0,
        };

        state.sectors.push(sector);
        sector_id
    }

    /// Clone an existing sector, duplicating its state.
    pub fn clone_sector(state: &mut TosState, source_id: Uuid) -> Option<Uuid> {
        let source_index = state.sectors.iter().position(|s| s.id == source_id)?;
        let source_sector = &state.sectors[source_index];
        
        let new_id = Uuid::new_v4();
        let mut new_sector = source_sector.clone();
        new_sector.id = new_id;
        new_sector.name = format!("{} (Clone)", source_sector.name);
        
        // Give hubs new IDs too
        for hub in &mut new_sector.hubs {
            hub.id = Uuid::new_v4();
        }

        state.sectors.push(new_sector);
        Some(new_id)
    }

    /// Close a sector and reclaim its resources.
    pub fn close_sector(state: &mut TosState, id: Uuid) -> bool {
        let initial_len = state.sectors.len();
        state.sectors.retain(|s| s.id != id);
        
        // Ensure active_sector_index stays valid
        if state.active_sector_index >= state.sectors.len() && !state.sectors.is_empty() {
            state.active_sector_index = state.sectors.len() - 1;
        }
        
        state.sectors.len() < initial_len
    }

    /// Toggle sector heart-beat/PTY update processing.
    pub fn toggle_freeze(state: &mut TosState, id: Uuid) {
        if let Some(sector) = state.sectors.iter_mut().find(|s| s.id == id) {
            sector.frozen = !sector.frozen;
            tracing::info!("Sector {} frozen set to {}", id, sector.frozen);
        }
    }

    /// Refresh directory listing via local filesystem interrogation.
    pub fn refresh_directory_listing(state: &mut TosState) {
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let hub = &mut sector.hubs[sector.active_hub_index];
            if hub.mode != CommandHubMode::Directory {
                return;
            }

            let path = hub.current_directory.clone();
            let mut entries = Vec::new();

            if let Ok(read_dir) = std::fs::read_dir(&path) {
                for entry in read_dir.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let metadata = entry.metadata();
                    let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                    let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                    
                    entries.push(crate::common::DirectoryEntry {
                        name,
                        is_dir,
                        size,
                    });
                }
            }

            hub.shell_listing = Some(crate::common::DirectoryListing {
                path: path.to_string_lossy().to_string(),
                entries,
            });
        }
    }

    /// Refresh activity listing for process monitoring hub modes.
    pub fn refresh_activity_listing(state: &mut TosState) {
        use sysinfo::System;
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let hub = &mut sector.hubs[sector.active_hub_index];
            if hub.mode != crate::common::CommandHubMode::Activity {
                return;
            }

            let mut sys = System::new_all();
            sys.refresh_all();
            
            let mut processes = Vec::new();
            for (pid, process) in sys.processes() {
                // In production, snapshots are pulled from Wayland DMABUF shared memory via PID mapping
                // For Alpha-2.1 UI testing, we generate a mock placeholder for active graphical PIDs
                let snapshot = if process.cpu_usage() > 1.0 {
                    Some("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==".to_string())
                } else {
                    None
                };

                processes.push(crate::common::ProcessEntry {
                    pid: pid.as_u32(),
                    name: process.name().to_string(),
                    cpu_usage: process.cpu_usage(),
                    mem_usage: process.memory(),
                    snapshot,
                });
            }

            // Sort by CPU usage descending
            processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));

            hub.activity_listing = Some(crate::common::ActivityListing {
                processes,
            });
        }
    }

    /// Perform global search across all active sectors and their respective hubs.
    pub fn perform_search(state: &mut TosState, query: &str) {
        if query.is_empty() {
            return;
        }

        let mut results = Vec::new();

        for sector in &state.sectors {
            let mut matches = Vec::new();
            for hub in &sector.hubs {
                for line in &hub.terminal_output {
                    if line.text.contains(query) {
                        matches.push(line.text.clone());
                    }
                }
            }
            if !matches.is_empty() {
                results.push(crate::common::SearchResult {
                    source_sector: sector.name.clone(),
                    matches,
                });
            }
        }

        // Apply results to active hub
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let hub = &mut sector.hubs[sector.active_hub_index];
            hub.search_results = Some(results);
        }
    }
}
