// Multi-Viewport State Management
// Based on "Multi-Monitor Support.md" and "Spatial Model Reconciliation.md"
//
// Each Viewport maintains its own independent zoom path stack.
// A physical monitor hosts one or more Viewports.
// Viewports can be created via Split or by plugging in new monitors.

use crate::navigation::zoom::ZoomLevel;
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a viewport
pub type ViewportId = u32;

/// Unique identifier for a physical monitor/output
pub type OutputId = u32;

/// The path stack representing position within the zoom hierarchy.
/// e.g. [SectorID, AppID, WindowID] — each element represents a
/// node at increasing depth.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoomPath {
    /// Stack of node identifiers at each depth level
    pub nodes: Vec<u32>,
}

impl ZoomPath {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Current depth in the hierarchy (0 = root)
    pub fn depth(&self) -> usize {
        self.nodes.len()
    }

    /// Push a node onto the path (zoom in)
    pub fn push(&mut self, node_id: u32) {
        self.nodes.push(node_id);
    }

    /// Pop the last node (zoom out), returns the removed node
    pub fn pop(&mut self) -> Option<u32> {
        self.nodes.pop()
    }

    /// Get the leaf (deepest focused node)
    pub fn leaf(&self) -> Option<u32> {
        self.nodes.last().copied()
    }

    /// Get the sector id (first element in path)
    pub fn sector_id(&self) -> Option<u32> {
        self.nodes.first().copied()
    }

    /// Get the app id (second element in path)
    pub fn app_id(&self) -> Option<u32> {
        self.nodes.get(1).copied()
    }

    /// Find the nearest common ancestor depth between two paths
    pub fn common_ancestor_depth(&self, other: &ZoomPath) -> usize {
        let mut depth = 0;
        for (a, b) in self.nodes.iter().zip(other.nodes.iter()) {
            if a == b {
                depth += 1;
            } else {
                break;
            }
        }
        depth
    }

    /// Generate the sequence of zoom operations to navigate from self to target.
    /// Returns (zoom_outs_needed, zoom_ins_with_targets)
    pub fn transition_to(&self, target: &ZoomPath) -> (usize, Vec<u32>) {
        let ancestor_depth = self.common_ancestor_depth(target);
        let zoom_outs = self.depth() - ancestor_depth;
        let zoom_ins: Vec<u32> = target.nodes[ancestor_depth..].to_vec();
        (zoom_outs, zoom_ins)
    }
}

impl fmt::Display for ZoomPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.nodes.is_empty() {
            write!(f, "[ROOT]")
        } else {
            let path_str: Vec<String> = self.nodes.iter().map(|n| n.to_string()).collect();
            write!(f, "[{}]", path_str.join(" → "))
        }
    }
}

/// A Viewport is a logical display pane that maintains its own
/// independent navigation state within the zoom hierarchy.
#[derive(Debug, Clone)]
pub struct Viewport {
    pub id: ViewportId,
    /// Which physical output this viewport is displayed on
    pub output_id: OutputId,
    /// Current zoom level enum (for compatibility with existing code)
    pub current_level: ZoomLevel,
    /// The full path stack (the source of truth for position)
    pub path: ZoomPath,
    /// Secondary surface id for split view within this viewport
    pub secondary_surface_id: Option<u32>,
    /// Viewport geometry on the output (fractional, 0.0-1.0)
    pub geometry: ViewportGeometry,
    /// Whether this viewport currently has input focus
    pub has_focus: bool,
    /// Name/label for this viewport (e.g. "Primary", "Split-Left")
    pub label: String,
}

/// Fractional geometry within a physical output
#[derive(Debug, Clone, Copy)]
pub struct ViewportGeometry {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ViewportGeometry {
    pub fn full() -> Self {
        Self { x: 0.0, y: 0.0, width: 1.0, height: 1.0 }
    }

    pub fn left_half() -> Self {
        Self { x: 0.0, y: 0.0, width: 0.5, height: 1.0 }
    }

    pub fn right_half() -> Self {
        Self { x: 0.5, y: 0.0, width: 0.5, height: 1.0 }
    }

    pub fn top_half() -> Self {
        Self { x: 0.0, y: 0.0, width: 1.0, height: 0.5 }
    }

    pub fn bottom_half() -> Self {
        Self { x: 0.0, y: 0.5, width: 1.0, height: 0.5 }
    }

    /// Convert fractional coordinates to pixel coordinates given output dimensions
    pub fn to_pixels(&self, output_width: u32, output_height: u32) -> (u32, u32, u32, u32) {
        let px = (self.x * output_width as f64) as u32;
        let py = (self.y * output_height as f64) as u32;
        let pw = (self.width * output_width as f64) as u32;
        let ph = (self.height * output_height as f64) as u32;
        (px, py, pw, ph)
    }
}

/// Information about a physical monitor/output
#[derive(Debug, Clone)]
pub struct OutputInfo {
    pub id: OutputId,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32, // in mHz (e.g. 60000 = 60Hz)
    pub scale_factor: f64,
    pub connected: bool,
}

/// The ViewportManager owns all viewports and outputs,
/// providing the multi-monitor / multi-pane infrastructure.
pub struct ViewportManager {
    viewports: HashMap<ViewportId, Viewport>,
    outputs: HashMap<OutputId, OutputInfo>,
    next_viewport_id: ViewportId,
    next_output_id: OutputId,
    /// Which viewport currently has keyboard/input focus
    focused_viewport: Option<ViewportId>,
}

impl ViewportManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            viewports: HashMap::new(),
            outputs: HashMap::new(),
            next_viewport_id: 1,
            next_output_id: 1,
            focused_viewport: None,
        };

        // Create a default output and viewport (the primary monitor)
        let output_id = mgr.add_output("Primary Monitor", 1920, 1080, 60000);
        let vp_id = mgr.create_viewport(output_id, ViewportGeometry::full(), "Primary");
        mgr.focused_viewport = Some(vp_id);

        mgr
    }

    // ─── Output Management ─────────────────────────────

    /// Register a new physical output (monitor)
    pub fn add_output(&mut self, name: &str, width: u32, height: u32, refresh_rate: u32) -> OutputId {
        let id = self.next_output_id;
        self.next_output_id += 1;
        self.outputs.insert(id, OutputInfo {
            id,
            name: name.to_string(),
            width,
            height,
            refresh_rate,
            scale_factor: 1.0,
            connected: true,
        });
        println!("[ViewportMgr] Output added: {} ({}x{} @{}Hz)", name, width, height, refresh_rate / 1000);
        id
    }

    /// Handle monitor hotplug disconnect
    pub fn remove_output(&mut self, output_id: OutputId) {
        // Reassign viewports from this output to the first remaining output
        let fallback = self.outputs.keys()
            .find(|&&k| k != output_id)
            .copied();

        if let Some(fallback_id) = fallback {
            let affected: Vec<ViewportId> = self.viewports.values()
                .filter(|v| v.output_id == output_id)
                .map(|v| v.id)
                .collect();

            for vp_id in affected {
                if let Some(vp) = self.viewports.get_mut(&vp_id) {
                    println!("[ViewportMgr] Migrating viewport {} from output {} to {}", vp_id, output_id, fallback_id);
                    vp.output_id = fallback_id;
                }
            }
        } else {
            // Last output being removed — remove all viewports
            let affected: Vec<ViewportId> = self.viewports.values()
                .filter(|v| v.output_id == output_id)
                .map(|v| v.id)
                .collect();
            for vp_id in affected {
                self.viewports.remove(&vp_id);
            }
        }

        self.outputs.remove(&output_id);
        println!("[ViewportMgr] Output {} removed", output_id);
    }

    pub fn get_output(&self, id: OutputId) -> Option<&OutputInfo> {
        self.outputs.get(&id)
    }

    pub fn get_all_outputs(&self) -> Vec<&OutputInfo> {
        self.outputs.values().collect()
    }

    // ─── Viewport Management ───────────────────────────

    /// Create a new viewport on a specific output
    pub fn create_viewport(&mut self, output_id: OutputId, geometry: ViewportGeometry, label: &str) -> ViewportId {
        let id = self.next_viewport_id;
        self.next_viewport_id += 1;

        let viewport = Viewport {
            id,
            output_id,
            current_level: ZoomLevel::Level1Root,
            path: ZoomPath::new(),
            secondary_surface_id: None,
            geometry,
            has_focus: self.viewports.is_empty(), // First viewport gets focus
            label: label.to_string(),
        };

        println!("[ViewportMgr] Viewport {} created on output {} ({})", id, output_id, label);
        self.viewports.insert(id, viewport);
        id
    }

    /// Remove a viewport (e.g. when closing a split pane)
    pub fn remove_viewport(&mut self, id: ViewportId) {
        if self.focused_viewport == Some(id) {
            // Transfer focus to another viewport
            self.focused_viewport = self.viewports.keys()
                .find(|&&k| k != id)
                .copied();
        }
        self.viewports.remove(&id);
        println!("[ViewportMgr] Viewport {} removed", id);
    }

    /// Get a reference to a viewport
    pub fn get_viewport(&self, id: ViewportId) -> Option<&Viewport> {
        self.viewports.get(&id)
    }

    /// Get a mutable reference to a viewport
    pub fn get_viewport_mut(&mut self, id: ViewportId) -> Option<&mut Viewport> {
        self.viewports.get_mut(&id)
    }

    /// Get the currently focused viewport
    pub fn get_focused(&self) -> Option<&Viewport> {
        self.focused_viewport.and_then(|id| self.viewports.get(&id))
    }

    /// Get mutable reference to the focused viewport
    pub fn get_focused_mut(&mut self) -> Option<&mut Viewport> {
        self.focused_viewport.and_then(|id| self.viewports.get_mut(&id))
    }

    pub fn focused_id(&self) -> Option<ViewportId> {
        self.focused_viewport
    }

    /// Set focus to a specific viewport
    pub fn set_focus(&mut self, id: ViewportId) {
        // Remove focus from all
        for vp in self.viewports.values_mut() {
            vp.has_focus = false;
        }
        // Set focus on target
        if let Some(vp) = self.viewports.get_mut(&id) {
            vp.has_focus = true;
            self.focused_viewport = Some(id);
            println!("[ViewportMgr] Focus -> Viewport {} ({})", id, vp.label);
        }
    }

    /// Get all viewports on a specific output
    pub fn get_viewports_on_output(&self, output_id: OutputId) -> Vec<&Viewport> {
        self.viewports.values()
            .filter(|v| v.output_id == output_id)
            .collect()
    }

    /// Get all viewport ids
    pub fn get_all_viewport_ids(&self) -> Vec<ViewportId> {
        self.viewports.keys().copied().collect()
    }

    // ─── Split Operations ──────────────────────────────

    /// Split the given viewport into two side-by-side viewports.
    /// The original viewport takes the left half, and a new viewport
    /// is created on the right half at Level 1 (for content selection).
    /// Returns the new viewport's id.
    pub fn split_horizontal(&mut self, viewport_id: ViewportId) -> Option<ViewportId> {
        let (output_id, original_geom) = {
            let vp = self.viewports.get(&viewport_id)?;
            (vp.output_id, vp.geometry)
        };

        // Resize original to left half of its current area
        if let Some(vp) = self.viewports.get_mut(&viewport_id) {
            vp.geometry = ViewportGeometry {
                x: original_geom.x,
                y: original_geom.y,
                width: original_geom.width / 2.0,
                height: original_geom.height,
            };
        }

        // Create new viewport on right half
        let new_geom = ViewportGeometry {
            x: original_geom.x + original_geom.width / 2.0,
            y: original_geom.y,
            width: original_geom.width / 2.0,
            height: original_geom.height,
        };

        let new_id = self.create_viewport(output_id, new_geom, "Split-Right");
        println!("[ViewportMgr] Split viewport {} → {} (left) + {} (right)", viewport_id, viewport_id, new_id);
        Some(new_id)
    }

    /// Split the given viewport vertically (top/bottom)
    pub fn split_vertical(&mut self, viewport_id: ViewportId) -> Option<ViewportId> {
        let (output_id, original_geom) = {
            let vp = self.viewports.get(&viewport_id)?;
            (vp.output_id, vp.geometry)
        };

        // Resize original to top half
        if let Some(vp) = self.viewports.get_mut(&viewport_id) {
            vp.geometry = ViewportGeometry {
                x: original_geom.x,
                y: original_geom.y,
                width: original_geom.width,
                height: original_geom.height / 2.0,
            };
        }

        let new_geom = ViewportGeometry {
            x: original_geom.x,
            y: original_geom.y + original_geom.height / 2.0,
            width: original_geom.width,
            height: original_geom.height / 2.0,
        };

        let new_id = self.create_viewport(output_id, new_geom, "Split-Bottom");
        println!("[ViewportMgr] Split viewport {} → {} (top) + {} (bottom)", viewport_id, viewport_id, new_id);
        Some(new_id)
    }

    /// Close a split by removing a viewport and restore the sibling to full size
    pub fn unsplit(&mut self, viewport_id: ViewportId) -> bool {
        let output_id = match self.viewports.get(&viewport_id) {
            Some(vp) => vp.output_id,
            None => return false,
        };

        // Find sibling viewports on the same output
        let siblings: Vec<ViewportId> = self.viewports.values()
            .filter(|v| v.output_id == output_id && v.id != viewport_id)
            .map(|v| v.id)
            .collect();

        self.remove_viewport(viewport_id);

        // If there's exactly one sibling, restore it to full size
        if siblings.len() == 1 {
            if let Some(vp) = self.viewports.get_mut(&siblings[0]) {
                vp.geometry = ViewportGeometry::full();
                return true;
            }
        }
        false
    }

    // ─── Navigation (per-viewport) ─────────────────────

    /// Zoom into a target on the focused viewport
    pub fn zoom_in_focused(&mut self, target_id: u32) -> Option<ZoomLevel> {
        let vp = self.get_focused_mut()?;
        vp.path.push(target_id);

        // Map path depth to zoom level
        vp.current_level = match vp.path.depth() {
            1 => ZoomLevel::Level2Sector,
            2 => ZoomLevel::Level3Focus,
            3 => ZoomLevel::Level4Detail,
            4 => ZoomLevel::Level5Buffer,
            _ => ZoomLevel::Level5Buffer,
        };

        println!("[ViewportMgr] Viewport {} zoom in → {} (path: {})",
            vp.id, vp.current_level, vp.path);
        Some(vp.current_level)
    }

    /// Zoom out on the focused viewport
    pub fn zoom_out_focused(&mut self) -> Option<ZoomLevel> {
        let vp = self.get_focused_mut()?;
        vp.path.pop();

        vp.current_level = match vp.path.depth() {
            0 => ZoomLevel::Level1Root,
            1 => ZoomLevel::Level2Sector,
            2 => ZoomLevel::Level3Focus,
            3 => ZoomLevel::Level4Detail,
            _ => ZoomLevel::Level5Buffer,
        };

        println!("[ViewportMgr] Viewport {} zoom out → {} (path: {})",
            vp.id, vp.current_level, vp.path);
        Some(vp.current_level)
    }

    /// Perform an Automated Vertical Transition on the focused viewport.
    /// This zooms out to the common ancestor and then zooms in to the target.
    /// Returns the sequence of operations for animation.
    pub fn navigate_to(&mut self, target_path: &ZoomPath) -> Option<Vec<NavigationStep>> {
        let vp = self.get_focused_mut()?;
        let (zoom_outs, zoom_ins) = vp.path.transition_to(target_path);

        let mut steps = Vec::new();

        // Zoom out phase
        for _ in 0..zoom_outs {
            let from = vp.path.clone();
            vp.path.pop();
            steps.push(NavigationStep::ZoomOut { from_path: from });
        }

        // Zoom in phase
        for node_id in &zoom_ins {
            vp.path.push(*node_id);
            steps.push(NavigationStep::ZoomIn { target_id: *node_id, to_path: vp.path.clone() });
        }

        // Update level
        vp.current_level = match vp.path.depth() {
            0 => ZoomLevel::Level1Root,
            1 => ZoomLevel::Level2Sector,
            2 => ZoomLevel::Level3Focus,
            3 => ZoomLevel::Level4Detail,
            _ => ZoomLevel::Level5Buffer,
        };

        println!("[ViewportMgr] Auto-transition: {} steps ({} out, {} in) → {}",
            steps.len(), zoom_outs, zoom_ins.len(), vp.path);

        Some(steps)
    }

    /// Get a summary of all viewports for debug/display
    pub fn summary(&self) -> String {
        let mut s = String::new();
        for vp in self.viewports.values() {
            let focus_marker = if vp.has_focus { " ★" } else { "" };
            s.push_str(&format!(
                "  VP-{}{}: {} | {} | path={}\n",
                vp.id, focus_marker, vp.label, vp.current_level, vp.path
            ));
        }
        s
    }
}

/// A single step in an Automated Vertical Transition animation sequence.
#[derive(Debug, Clone)]
pub enum NavigationStep {
    ZoomOut { from_path: ZoomPath },
    ZoomIn { target_id: u32, to_path: ZoomPath },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_path_basics() {
        let mut path = ZoomPath::new();
        assert_eq!(path.depth(), 0);
        assert_eq!(path.leaf(), None);

        path.push(0); // Sector 0
        assert_eq!(path.depth(), 1);
        assert_eq!(path.sector_id(), Some(0));

        path.push(3); // App 3
        assert_eq!(path.depth(), 2);
        assert_eq!(path.app_id(), Some(3));
        assert_eq!(path.leaf(), Some(3));

        let popped = path.pop();
        assert_eq!(popped, Some(3));
        assert_eq!(path.depth(), 1);
    }

    #[test]
    fn test_common_ancestor() {
        let mut a = ZoomPath::new();
        a.push(0); a.push(1); a.push(2);

        let mut b = ZoomPath::new();
        b.push(0); b.push(1); b.push(5);

        assert_eq!(a.common_ancestor_depth(&b), 2); // Sector 0, App 1

        let mut c = ZoomPath::new();
        c.push(1); c.push(4);
        assert_eq!(a.common_ancestor_depth(&c), 0); // Different sectors
    }

    #[test]
    fn test_transition_calculation() {
        let mut from = ZoomPath::new();
        from.push(0); from.push(1); from.push(2);

        let mut to = ZoomPath::new();
        to.push(0); to.push(3);

        let (outs, ins) = from.transition_to(&to);
        assert_eq!(outs, 2); // pop 2, pop 1
        assert_eq!(ins, vec![3]); // push 3
    }

    #[test]
    fn test_viewport_manager_creation() {
        let mgr = ViewportManager::new();
        assert_eq!(mgr.get_all_outputs().len(), 1);
        assert_eq!(mgr.get_all_viewport_ids().len(), 1);
        assert!(mgr.focused_id().is_some());
    }

    #[test]
    fn test_multi_output() {
        let mut mgr = ViewportManager::new();
        let out2 = mgr.add_output("External Monitor", 2560, 1440, 144000);
        let vp2 = mgr.create_viewport(out2, ViewportGeometry::full(), "External");

        assert_eq!(mgr.get_all_outputs().len(), 2);
        assert_eq!(mgr.get_all_viewport_ids().len(), 2);

        // Each viewport is on a different output
        let vps_out2 = mgr.get_viewports_on_output(out2);
        assert_eq!(vps_out2.len(), 1);
        assert_eq!(vps_out2[0].id, vp2);
    }

    #[test]
    fn test_split_horizontal() {
        let mut mgr = ViewportManager::new();
        let vp1 = mgr.focused_id().unwrap();

        let vp2 = mgr.split_horizontal(vp1).unwrap();

        // Now we have 2 viewports on the same output
        assert_eq!(mgr.get_all_viewport_ids().len(), 2);

        let v1 = mgr.get_viewport(vp1).unwrap();
        let v2 = mgr.get_viewport(vp2).unwrap();

        // Left half
        assert!((v1.geometry.width - 0.5).abs() < 0.001);
        assert!((v1.geometry.x - 0.0).abs() < 0.001);

        // Right half
        assert!((v2.geometry.width - 0.5).abs() < 0.001);
        assert!((v2.geometry.x - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_independent_navigation() {
        let mut mgr = ViewportManager::new();
        let vp1 = mgr.focused_id().unwrap();
        let out1 = mgr.get_viewport(vp1).unwrap().output_id;

        // Create second viewport
        let vp2 = mgr.create_viewport(out1, ViewportGeometry::right_half(), "Right");

        // Navigate viewport 1 into Sector 0 → App 1
        mgr.set_focus(vp1);
        mgr.zoom_in_focused(0);
        mgr.zoom_in_focused(1);

        // Navigate viewport 2 into Sector 2
        mgr.set_focus(vp2);
        mgr.zoom_in_focused(2);

        // They should be at different depths and different sectors
        let v1 = mgr.get_viewport(vp1).unwrap();
        let v2 = mgr.get_viewport(vp2).unwrap();

        assert_eq!(v1.current_level, ZoomLevel::Level3Focus);
        assert_eq!(v1.path.sector_id(), Some(0));
        assert_eq!(v1.path.app_id(), Some(1));

        assert_eq!(v2.current_level, ZoomLevel::Level2Sector);
        assert_eq!(v2.path.sector_id(), Some(2));
        assert_eq!(v2.path.app_id(), None);
    }

    #[test]
    fn test_automated_vertical_transition() {
        let mut mgr = ViewportManager::new();

        // Navigate to Sector 0 → App 1 → Window 2
        mgr.zoom_in_focused(0);
        mgr.zoom_in_focused(1);
        mgr.zoom_in_focused(2);

        // Target: Sector 0 → App 3
        let mut target = ZoomPath::new();
        target.push(0);
        target.push(3);

        let steps = mgr.navigate_to(&target).unwrap();

        // Should zoom out 2 (remove Window 2, remove App 1), then zoom in 1 (App 3)
        assert_eq!(steps.len(), 3);

        let vp = mgr.get_focused().unwrap();
        assert_eq!(vp.path.depth(), 2);
        assert_eq!(vp.path.app_id(), Some(3));
        assert_eq!(vp.current_level, ZoomLevel::Level3Focus);
    }

    #[test]
    fn test_cross_sector_transition() {
        let mut mgr = ViewportManager::new();

        // Navigate to Sector 0 → App 1
        mgr.zoom_in_focused(0);
        mgr.zoom_in_focused(1);

        // Target: Sector 2 → App 4
        let mut target = ZoomPath::new();
        target.push(2);
        target.push(4);

        let steps = mgr.navigate_to(&target).unwrap();

        // Should zoom out 2 (to root), then zoom in 2 (sector 2, app 4)
        assert_eq!(steps.len(), 4);

        let vp = mgr.get_focused().unwrap();
        assert_eq!(vp.path.sector_id(), Some(2));
        assert_eq!(vp.path.app_id(), Some(4));
    }

    #[test]
    fn test_unsplit() {
        let mut mgr = ViewportManager::new();
        let vp1 = mgr.focused_id().unwrap();
        let vp2 = mgr.split_horizontal(vp1).unwrap();

        assert_eq!(mgr.get_all_viewport_ids().len(), 2);

        // Close the right split
        mgr.unsplit(vp2);
        assert_eq!(mgr.get_all_viewport_ids().len(), 1);

        // Remaining viewport should be full size
        let v1 = mgr.get_viewport(vp1).unwrap();
        assert!((v1.geometry.width - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_output_hotplug() {
        let mut mgr = ViewportManager::new();
        let out2 = mgr.add_output("HDMI-1", 1920, 1080, 60000);
        let _vp2 = mgr.create_viewport(out2, ViewportGeometry::full(), "HDMI");

        assert_eq!(mgr.get_all_viewport_ids().len(), 2);

        // Disconnect HDMI
        mgr.remove_output(out2);

        // Viewport should have been migrated to remaining output
        assert_eq!(mgr.get_all_viewport_ids().len(), 2);
        for vp in mgr.viewports.values() {
            assert_ne!(vp.output_id, out2);
        }
    }

    #[test]
    fn test_focus_switching() {
        let mut mgr = ViewportManager::new();
        let vp1 = mgr.focused_id().unwrap();
        let out1 = mgr.get_viewport(vp1).unwrap().output_id;
        let vp2 = mgr.create_viewport(out1, ViewportGeometry::right_half(), "Right");

        mgr.set_focus(vp2);
        assert_eq!(mgr.focused_id(), Some(vp2));
        assert!(!mgr.get_viewport(vp1).unwrap().has_focus);
        assert!(mgr.get_viewport(vp2).unwrap().has_focus);

        mgr.set_focus(vp1);
        assert_eq!(mgr.focused_id(), Some(vp1));
        assert!(mgr.get_viewport(vp1).unwrap().has_focus);
        assert!(!mgr.get_viewport(vp2).unwrap().has_focus);
    }

    #[test]
    fn test_path_display() {
        let path = ZoomPath::new();
        assert_eq!(format!("{}", path), "[ROOT]");

        let mut path2 = ZoomPath::new();
        path2.push(0);
        path2.push(3);
        path2.push(7);
        assert_eq!(format!("{}", path2), "[0 → 3 → 7]");
    }
}
