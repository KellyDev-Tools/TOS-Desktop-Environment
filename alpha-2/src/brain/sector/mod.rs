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
            priority: 1,
            active_apps: vec![],
            active_app_index: 0,
            version: 0,
        };

        state.sectors.push(sector);
        sector_id
    }

    /// Create a sector from a predefined blueprint.
    pub fn create_from_template(state: &mut TosState, template: crate::common::SectorTemplate) -> Uuid {
        let sector_id = Uuid::new_v4();
        let mut hubs = Vec::new();

        for hub_tmpl in template.hubs {
            let hub_id = Uuid::new_v4();
            let cwd = PathBuf::from(hub_tmpl.cwd.replace("~", &dirs::home_dir().unwrap_or_default().to_string_lossy()));
            
            hubs.push(CommandHub {
                id: hub_id,
                mode: hub_tmpl.mode,
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
            });
        }

        // Ensure at least one hub exists
        if hubs.is_empty() {
             Self::create_sector(state, template.name);
             return sector_id;
        }

        let sector = Sector {
            id: sector_id,
            name: template.name,
            hubs,
            active_hub_index: 0,
            frozen: false,
            is_remote: false,
            disconnected: false,
            trust_tier: crate::common::TrustTier::Standard,
            priority: 1,
            active_apps: vec![],
            active_app_index: 0,
            version: 0,
        };

        state.sectors.push(sector);
        sector_id
    }

    /// Launch a new application instance within a sector (§8.2).
    pub fn launch_app(state: &mut TosState, sector_id: Uuid, model: crate::common::ApplicationModel) -> Uuid {
        let app_id = Uuid::new_v4();
        if let Some(sector) = state.sectors.iter_mut().find(|s| s.id == sector_id) {
            sector.active_apps.push(crate::common::AppInstance {
                id: app_id,
                model_id: model.id,
                title: model.name.clone(),
                state_summary: "INITIALIZING".to_string(),
            });
            sector.active_app_index = sector.active_apps.len() - 1;
            
            // Automatic Zoom In to Level 3 (§1.1)
            state.current_level = crate::common::HierarchyLevel::ApplicationFocus;
            tracing::info!("Application {} launched in sector {}", model.name, sector_id);
        }
        app_id
    }

    /// Close an application instance and revert focus if needed (§8.2).
    pub fn close_app(state: &mut TosState, sector_id: Uuid, app_id: Uuid) {
        if let Some(sector) = state.sectors.iter_mut().find(|s| s.id == sector_id) {
            sector.active_apps.retain(|a| a.id != app_id);
            if sector.active_apps.is_empty() {
                sector.active_app_index = 0;
                // Zoom Out to Level 2 if no apps remain (§1.1)
                if state.current_level == crate::common::HierarchyLevel::ApplicationFocus {
                    state.current_level = crate::common::HierarchyLevel::CommandHub;
                }
            } else if sector.active_app_index >= sector.active_apps.len() {
                sector.active_app_index = sector.active_apps.len() - 1;
            }
        }
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
