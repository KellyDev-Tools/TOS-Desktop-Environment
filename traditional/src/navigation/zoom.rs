use std::fmt;

// Definitions for the Recursive Zoom Hierarchy
// Based on "Thoughts on Spatial Navigation.md"

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZoomLevel {
    Level1Root,      // Overview of all sectors
    Level2Sector,    // Group of related apps/tasks (e.g. Work, Media)
    Level3Focus,     // Active application window
    Level3aPicker,   // Window picker for an app with multiple windows
    Level3Split,     // Two windows side-by-side
    Level4Detail,    // Deep-dive into a specific file or process detail
}

impl fmt::Display for ZoomLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZoomLevel::Level1Root => write!(f, "Level 1: Root (Overview)"),
            ZoomLevel::Level2Sector => write!(f, "Level 2: Sector (Group)"),
            ZoomLevel::Level3Focus => write!(f, "Level 3: Focus (App)"),
            ZoomLevel::Level3aPicker => write!(f, "Level 3a: Picker (Windows)"),
            ZoomLevel::Level3Split => write!(f, "Level 3: Split View"),
            ZoomLevel::Level4Detail => write!(f, "Level 4: Detail (Inspect)"),
        }
    }
}

pub struct SectorInfo {
    pub name: String,
    pub color: String, // CSS color or LCARS class
}

pub struct SpatialNavigator {
    pub current_level: ZoomLevel,
    pub sectors: Vec<SectorInfo>,
    pub active_sector_index: Option<usize>,
    pub active_app_index: Option<usize>,
    pub active_window_index: Option<usize>,
    pub secondary_app_id: Option<u32>, // For Split View
}

impl SpatialNavigator {
    pub fn new() -> Self {
        Self {
            current_level: ZoomLevel::Level1Root,
            sectors: vec![
                SectorInfo { name: "WORK".to_string(), color: "var(--lcars-blue)".to_string() },
                SectorInfo { name: "MEDIA".to_string(), color: "var(--lcars-orange)".to_string() },
                SectorInfo { name: "CORE".to_string(), color: "var(--lcars-red)".to_string() },
                SectorInfo { name: "DATA".to_string(), color: "var(--lcars-blue-dark)".to_string() },
            ],
            active_sector_index: None,
            active_app_index: None,
            active_window_index: None,
            secondary_app_id: None,
        }
    }

    pub fn zoom_in(&mut self, target_index: usize) {
        match self.current_level {
            ZoomLevel::Level1Root => {
                if target_index < self.sectors.len() {
                    self.active_sector_index = Some(target_index);
                    self.current_level = ZoomLevel::Level2Sector;
                    println!("[Zoom In] Entering Sector {}", target_index);
                } else {
                    println!("[Navigate] Invalid sector index: {}", target_index);
                }
            }
            ZoomLevel::Level2Sector => {
                self.active_app_index = Some(target_index);
                self.current_level = ZoomLevel::Level3Focus;
                println!("[Zoom In] Focusing on App {} (Morphing SSD Frame...)", target_index);
            }
            ZoomLevel::Level3aPicker => {
                self.active_window_index = Some(target_index);
                self.current_level = ZoomLevel::Level3Focus;
                println!("[Zoom In] Selected Window {} from Picker.", target_index);
            }
            ZoomLevel::Level3Focus => {
                self.current_level = ZoomLevel::Level4Detail;
                println!("[Zoom In] Deep-diving into Details (Level 4).");
            }
            _ => {
                println!("[Navigate] Already at deepest level (Level 4 Detail).");
            }
        }
    }

    pub fn zoom_out(&mut self, has_multiple_windows: bool) {
        self.secondary_app_id = None; // Always reset split on zoom out
        match self.current_level {
            ZoomLevel::Level4Detail => {
                self.current_level = ZoomLevel::Level3Focus;
                println!("[Zoom Out] Returning to App Focus (Level 3).");
            }
            ZoomLevel::Level3Split => {
                self.current_level = ZoomLevel::Level3Focus;
                println!("[Split Out] Returning to Single Focus View.");
            }
            ZoomLevel::Level3Focus => {
                if has_multiple_windows {
                    self.current_level = ZoomLevel::Level3aPicker;
                    println!("[Zoom Out] Multiple windows detected -> Entering Window Picker (Level 3a).");
                } else {
                    self.current_level = ZoomLevel::Level2Sector;
                    self.active_app_index = None;
                    println!("[Zoom Out] Returning to Sector View (Level 2).");
                }
            }
            ZoomLevel::Level3aPicker => {
                self.current_level = ZoomLevel::Level2Sector;
                self.active_app_index = None;
                println!("[Zoom Out] Returning to Sector View (Level 2) from Picker.");
            }
            ZoomLevel::Level2Sector => {
                self.current_level = ZoomLevel::Level1Root;
                self.active_sector_index = None;
                println!("[Zoom Out] Returning to Root Overview (Level 1).");
            }
            _ => {
                println!("[Navigate] Already at top level (Level 1 Root).");
            }
        }
    }

    pub fn split_view(&mut self, secondary_id: u32) {
        if self.current_level == ZoomLevel::Level3Focus {
            println!("[Split] Entering Split View with Surface {}", secondary_id);
            self.secondary_app_id = Some(secondary_id);
            self.current_level = ZoomLevel::Level3Split;
        } else {
            println!("[Split] Can only split from a focused app (Level 3).");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let nav = SpatialNavigator::new();
        assert_eq!(nav.current_level, ZoomLevel::Level1Root);
        assert!(nav.active_sector_index.is_none());
    }

    #[test]
    fn test_zoom_flow() {
        let mut nav = SpatialNavigator::new();

        // 1. Zoom into Sector (Level 1 -> 2)
        nav.zoom_in(0);
        assert_eq!(nav.current_level, ZoomLevel::Level2Sector);
        assert_eq!(nav.active_sector_index, Some(0));

        // 2. Zoom into App (Level 2 -> 3)
        nav.zoom_in(1); 
        assert_eq!(nav.current_level, ZoomLevel::Level3Focus);

        // 3. Zoom Out (Level 3 -> 2)
        nav.zoom_out(false);
        assert_eq!(nav.current_level, ZoomLevel::Level2Sector);
        assert!(nav.active_app_index.is_none());
        
        // 4. Zoom Out to Root (Level 2 -> 1)
        nav.zoom_out(false);
        assert_eq!(nav.current_level, ZoomLevel::Level1Root);
    }
    
    #[test]
    fn test_picker_flow() {
        let mut nav = SpatialNavigator::new();
        nav.zoom_in(0); // Sector
        nav.zoom_in(2); // Focus

        // Should go to Picker on Zoom Out if multiple windows exist
        nav.zoom_out(true);
        assert_eq!(nav.current_level, ZoomLevel::Level3aPicker);
        
        // Picker back to Sector
        nav.zoom_out(false);
        assert_eq!(nav.current_level, ZoomLevel::Level2Sector);
    }
}
