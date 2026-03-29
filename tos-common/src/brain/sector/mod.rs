use crate::{CommandHub, CommandHubMode, Sector, TosState};
use std::path::PathBuf;
use uuid::Uuid;

pub struct SectorManager;

impl SectorManager {
    /// Create a new sector with a default hub.
    pub fn create_sector(state: &mut TosState, name: String) -> Uuid {
        let sector_id = Uuid::new_v4();
        let hub_id = Uuid::new_v4();

        // Use current directory of active hub as starting point if possible
        let cwd = if let Some(active_sector) = state.sectors.get(state.active_sector_index) {
            active_sector.hubs[active_sector.active_hub_index]
                .current_directory
                .clone()
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
                shell_module: Some("tos-shell-fish".to_string()),
                split_layout: None,
                focused_pane_id: None,
                ai_history: vec![],
                version: 0,
                is_running: false,
                last_exit_status: None,
            }],
            active_hub_index: 0,
            frozen: false,
            is_remote: false,
            disconnected: false,
            trust_tier: crate::TrustTier::System,
            priority: 1,
            active_apps: vec![],
            active_app_index: 0,
            participants: vec![],
            version: 0,
        };

        state.sectors.push(sector);
        sector_id
    }

    /// Create a sector from a predefined blueprint.
    pub fn create_from_template(
        state: &mut TosState,
        template: crate::SectorTemplate,
    ) -> Uuid {
        let sector_id = Uuid::new_v4();
        let mut hubs = Vec::new();

        for hub_tmpl in template.hubs {
            let hub_id = Uuid::new_v4();
            let cwd = PathBuf::from(
                hub_tmpl
                    .cwd
                    .replace("~", &dirs::home_dir().unwrap_or_default().to_string_lossy()),
            );

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
                shell_module: Some(hub_tmpl.shell.clone()),
                split_layout: None,
                focused_pane_id: None,
                ai_history: vec![],
                version: 0,
                is_running: false,
                last_exit_status: None,
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
            trust_tier: crate::TrustTier::Standard,
            priority: 1,
            active_apps: vec![],
            active_app_index: 0,
            participants: vec![],
            version: 0,
        };

        state.sectors.push(sector);
        sector_id
    }

    /// Launch a new application instance within a sector (§8.2).
    pub fn launch_app(
        state: &mut TosState,
        sector_id: Uuid,
        model: crate::ApplicationModel,
    ) -> Uuid {
        let app_id = Uuid::new_v4();
        if let Some(sector) = state.sectors.iter_mut().find(|s| s.id == sector_id) {
            sector.active_apps.push(crate::AppInstance {
                id: app_id,
                model_id: model.id,
                title: model.name.clone(),
                state_summary: "INITIALIZING".to_string(),
            });
            sector.active_app_index = sector.active_apps.len() - 1;

            // Automatic Zoom In to Level 3 (§1.1)
            state.current_level = crate::HierarchyLevel::ApplicationFocus;
            tracing::info!(
                "Application {} launched in sector {}",
                model.name,
                sector_id
            );
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
                if state.current_level == crate::HierarchyLevel::ApplicationFocus {
                    state.current_level = crate::HierarchyLevel::CommandHub;
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

                    entries.push(crate::DirectoryEntry { name, is_dir, size });
                }
            }

            hub.shell_listing = Some(crate::DirectoryListing {
                path: path.to_string_lossy().to_string(),
                entries,
            });
        }
    }

    /// Refresh activity listing for process monitoring hub modes.
    pub fn refresh_activity_listing(
        state: &mut TosState,
        capture_svc: Option<&crate::services::CaptureService>,
    ) {
        use sysinfo::System;
        let idx = state.active_sector_index;
        if let Some(sector) = state.sectors.get_mut(idx) {
            let hub = &mut sector.hubs[sector.active_hub_index];
            if hub.mode != crate::CommandHubMode::Activity {
                return;
            }

            let mut sys = System::new_all();
            sys.refresh_all();

            let mut processes = Vec::new();
            for (pid_type, process) in sys.processes() {
                let pid = pid_type.as_u32();
                // Fetch dynamic snapshot from the capture service if available
                let snapshot = capture_svc.and_then(|svc| svc.get_snapshot(pid));

                processes.push(crate::ProcessEntry {
                    pid,
                    name: process.name().to_string(),
                    cpu_usage: process.cpu_usage(),
                    mem_usage: process.memory(),
                    snapshot,
                });
            }

            // Sort by CPU usage descending
            processes.sort_by(|a, b| {
                b.cpu_usage
                    .partial_cmp(&a.cpu_usage)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            hub.activity_listing = Some(crate::ActivityListing { processes });
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
                results.push(crate::SearchResult {
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

    // --- Split Pane Lifecycle ---

    /// Create a new split pane in the active hub.
    /// Returns the new pane's UUID, or None if minimum size constraints block the split.
    pub fn split_create(
        state: &mut TosState,
        display_width: u32,
        display_height: u32,
    ) -> Result<Uuid, String> {
        let idx = state.active_sector_index;
        let hub_idx = state.sectors[idx].active_hub_index;
        let hub = &mut state.sectors[idx].hubs[hub_idx];

        let existing_count = hub
            .split_layout
            .as_ref()
            .map(|l| l.pane_count())
            .unwrap_or(1);

        if !crate::SplitNode::can_split(existing_count, display_width, display_height) {
            return Err("SPLIT_BLOCKED: minimum pane size limit reached".to_string());
        }

        let new_pane_id = Uuid::new_v4();
        let new_pane = crate::SplitPane {
            id: new_pane_id,
            weight: 0.5,
            cwd: hub.current_directory.clone(),
            content: crate::PaneContent::Terminal,
        };

        let orientation =
            crate::SplitNode::ideal_orientation(display_width, display_height);

        hub.split_layout = Some(match hub.split_layout.take() {
            None => {
                // First split: wrap the implicit single pane into a 2-pane container
                let original_pane = crate::SplitPane {
                    id: Uuid::new_v4(),
                    weight: 0.5,
                    cwd: hub.current_directory.clone(),
                    content: crate::PaneContent::Terminal,
                };
                crate::SplitNode::Container {
                    orientation,
                    children: vec![
                        crate::SplitNode::Leaf(original_pane),
                        crate::SplitNode::Leaf(new_pane),
                    ],
                }
            }
            Some(existing) => crate::SplitNode::Container {
                orientation,
                children: vec![existing, crate::SplitNode::Leaf(new_pane)],
            },
        });

        hub.focused_pane_id = Some(new_pane_id);
        Ok(new_pane_id)
    }

    /// Close a specific split pane. Collapses to single pane if only one remains.
    pub fn split_close(state: &mut TosState, pane_id: Uuid) -> bool {
        let idx = state.active_sector_index;
        let hub_idx = state.sectors[idx].active_hub_index;
        let hub = &mut state.sectors[idx].hubs[hub_idx];

        fn remove_pane(
            node: crate::SplitNode,
            id: Uuid,
        ) -> Option<crate::SplitNode> {
            match node {
                crate::SplitNode::Leaf(p) => {
                    if p.id == id {
                        None
                    } else {
                        Some(crate::SplitNode::Leaf(p))
                    }
                }
                crate::SplitNode::Container {
                    orientation,
                    children,
                } => {
                    let filtered: Vec<_> = children
                        .into_iter()
                        .filter_map(|c| remove_pane(c, id))
                        .collect();
                    if filtered.is_empty() {
                        None
                    } else if filtered.len() == 1 {
                        Some(filtered.into_iter().next().unwrap())
                    } else {
                        Some(crate::SplitNode::Container {
                            orientation,
                            children: filtered,
                        })
                    }
                }
            }
        }

        if let Some(layout) = hub.split_layout.take() {
            hub.split_layout = remove_pane(layout, pane_id);
            if hub.focused_pane_id == Some(pane_id) {
                hub.focused_pane_id = hub
                    .split_layout
                    .as_ref()
                    .and_then(|l| l.all_pane_ids().into_iter().next());
            }
            return true;
        }
        false
    }

    /// Set the focused pane by ID.
    pub fn split_focus(state: &mut TosState, pane_id: Uuid) -> bool {
        let idx = state.active_sector_index;
        let hub_idx = state.sectors[idx].active_hub_index;
        let hub = &mut state.sectors[idx].hubs[hub_idx];
        let known_ids = hub
            .split_layout
            .as_ref()
            .map(|l| l.all_pane_ids())
            .unwrap_or_default();
        if known_ids.contains(&pane_id) {
            hub.focused_pane_id = Some(pane_id);
            return true;
        }
        false
    }

    /// Focus the next pane in a given direction (left/right/up/down).
    /// For simplicity, cycles through panes in order.
    pub fn split_focus_direction(state: &mut TosState, direction: &str) -> Option<Uuid> {
        let idx = state.active_sector_index;
        let hub_idx = state.sectors[idx].active_hub_index;
        let hub = &mut state.sectors[idx].hubs[hub_idx];
        let ids = hub
            .split_layout
            .as_ref()
            .map(|l| l.all_pane_ids())
            .unwrap_or_default();
        if ids.is_empty() {
            return None;
        }
        let current_pos = hub
            .focused_pane_id
            .and_then(|id| ids.iter().position(|&x| x == id))
            .unwrap_or(0);
        let next_pos = match direction {
            "right" | "down" => (current_pos + 1) % ids.len(),
            "left" | "up" => {
                if current_pos == 0 {
                    ids.len() - 1
                } else {
                    current_pos - 1
                }
            }
            _ => current_pos,
        };
        hub.focused_pane_id = Some(ids[next_pos]);
        hub.focused_pane_id
    }

    /// Equalize all pane weights in the layout.
    pub fn split_equalize(state: &mut TosState) -> bool {
        let idx = state.active_sector_index;
        let hub_idx = state.sectors[idx].active_hub_index;
        let hub = &mut state.sectors[idx].hubs[hub_idx];
        fn equalize(node: &mut crate::SplitNode) {
            if let crate::SplitNode::Container { children, .. } = node {
                let weight = 1.0 / children.len() as f32;
                for child in children.iter_mut() {
                    if let crate::SplitNode::Leaf(p) = child {
                        p.weight = weight;
                    }
                    equalize(child);
                }
            }
        }
        if let Some(layout) = &mut hub.split_layout {
            equalize(layout);
            return true;
        }
        false
    }

    /// Enter fullscreen for a specific pane (collapse layout to just that pane).
    pub fn split_fullscreen(state: &mut TosState, pane_id: Uuid) -> bool {
        let idx = state.active_sector_index;
        let hub_idx = state.sectors[idx].active_hub_index;
        let hub = &mut state.sectors[idx].hubs[hub_idx];
        fn find_pane(
            node: &crate::SplitNode,
            id: Uuid,
        ) -> Option<crate::SplitPane> {
            match node {
                crate::SplitNode::Leaf(p) => {
                    if p.id == id {
                        Some(p.clone())
                    } else {
                        None
                    }
                }
                crate::SplitNode::Container { children, .. } => {
                    children.iter().find_map(|c| find_pane(c, id))
                }
            }
        }
        if let Some(layout) = &hub.split_layout {
            if let Some(pane) = find_pane(layout, pane_id) {
                hub.split_layout = Some(crate::SplitNode::Leaf(pane));
                hub.focused_pane_id = Some(pane_id);
                return true;
            }
        }
        false
    }

    /// Detach focused pane to a new sector (Fresh Start mode: open clean shell in same cwd).
    pub fn split_detach_fresh(state: &mut TosState) -> Option<Uuid> {
        let idx = state.active_sector_index;
        let hub_idx = state.sectors[idx].active_hub_index;
        let pane_id = state.sectors[idx].hubs[hub_idx].focused_pane_id?;
        let cwd = {
            let hub_idx = state.sectors[idx].active_hub_index;
            let hub = &state.sectors[idx].hubs[hub_idx];
            let layout = hub.split_layout.as_ref()?;
            fn find_cwd(node: &crate::SplitNode, id: Uuid) -> Option<std::path::PathBuf> {
                match node {
                    crate::SplitNode::Leaf(p) => {
                        if p.id == id {
                            Some(p.cwd.clone())
                        } else {
                            None
                        }
                    }
                    crate::SplitNode::Container { children, .. } => {
                        children.iter().find_map(|c| find_cwd(c, id))
                    }
                }
            }
            find_cwd(layout, pane_id)?
        };

        // Close the pane from the source hub
        Self::split_close(state, pane_id);

        // Create a new sector with the same cwd
        let new_sector_id = Self::create_sector_at_cwd(state, "Detached".to_string(), cwd);
        state.active_sector_index = state
            .sectors
            .iter()
            .position(|s| s.id == new_sector_id)
            .unwrap_or(0);
        Some(new_sector_id)
    }

    /// Create a sector with a specific cwd (used by split detach).
    pub fn create_sector_at_cwd(
        state: &mut TosState,
        name: String,
        cwd: std::path::PathBuf,
    ) -> Uuid {
        let sector_id = Uuid::new_v4();
        let hub_id = Uuid::new_v4();
        let sector = crate::Sector {
            id: sector_id,
            name,
            hubs: vec![crate::CommandHub {
                id: hub_id,
                mode: crate::CommandHubMode::Command,
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
                shell_module: Some("tos-shell-fish".to_string()),
                split_layout: None,
                focused_pane_id: None,
                ai_history: vec![],
                version: 0,
                is_running: false,
                last_exit_status: None,
            }],
            active_hub_index: 0,
            frozen: false,
            is_remote: false,
            disconnected: false,
            trust_tier: crate::TrustTier::System,
            priority: 1,
            active_apps: vec![],
            active_app_index: 0,
            participants: vec![],
            version: 0,
        };
        state.sectors.push(sector);
        sector_id
    }
}
