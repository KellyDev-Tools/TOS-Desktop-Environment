use std::fmt;

// Definitions for the Recursive Zoom Hierarchy
// Based on "Thoughts on Spatial Navigation.md"

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZoomLevel {
    Level1Root,      // Overview of all sectors
    Level2Sector,    // Group of related apps/tasks (e.g. Work, Media)
    Level3Focus,     // Active application window
    Level3aPicker,   // Window picker for an app with multiple windows
}

impl fmt::Display for ZoomLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZoomLevel::Level1Root => write!(f, "Level 1: Root (Overview)"),
            ZoomLevel::Level2Sector => write!(f, "Level 2: Sector (Group)"),
            ZoomLevel::Level3Focus => write!(f, "Level 3: Focus (App)"),
            ZoomLevel::Level3aPicker => write!(f, "Level 3a: Picker (Windows)"),
        }
    }
}

pub struct SpatialNavigator {
    pub current_level: ZoomLevel,
    pub active_sector_index: Option<usize>,
    pub active_app_index: Option<usize>,
    pub active_window_index: Option<usize>,
}

impl SpatialNavigator {
    pub fn new() -> Self {
        Self {
            current_level: ZoomLevel::Level1Root,
            active_sector_index: None,
            active_app_index: None,
            active_window_index: None,
        }
    }

    pub fn zoom_in(&mut self, target_index: usize) {
        match self.current_level {
            ZoomLevel::Level1Root => {
                self.active_sector_index = Some(target_index);
                self.current_level = ZoomLevel::Level2Sector;
                println!("[Zoom In] Entering Sector {}", target_index);
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
            _ => {
                println!("[Navigate] Already at deepest level (Level 3 Focus).");
            }
        }
    }

    pub fn zoom_out(&mut self) {
        match self.current_level {
            ZoomLevel::Level3Focus => {
                // Mock condition: even index apps have multiple windows to simulate logic
                let has_multiple_windows = self.active_app_index.map_or(false, |idx| idx % 2 == 0);
                
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

    pub fn split_view(&self) {
        if self.current_level == ZoomLevel::Level3Focus {
            println!("[Split] Splitting Viewport...");
            println!("  -> Left Pane: Retains App Focus (Level 3)");
            println!("  -> Right Pane: Reverts to Level 2 (Sector Selection)");
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
        // With index 1 (Odd), simulates single window app
        nav.zoom_in(1); 
        assert_eq!(nav.current_level, ZoomLevel::Level3Focus);
        assert_eq!(nav.active_app_index, Some(1));

        // 3. Zoom Out (Level 3 -> 2)
        nav.zoom_out();
        assert_eq!(nav.current_level, ZoomLevel::Level2Sector);
        assert!(nav.active_app_index.is_none());
        
        // 4. Zoom Out to Root (Level 2 -> 1)
        nav.zoom_out();
        assert_eq!(nav.current_level, ZoomLevel::Level1Root);
    }
    
    #[test]
    fn test_picker_flow() {
        let mut nav = SpatialNavigator::new();
        nav.zoom_in(0); // Sector
        nav.zoom_in(2); // Even index app -> multiple windows

        // Should go to Picker on Zoom Out
        nav.zoom_out();
        assert_eq!(nav.current_level, ZoomLevel::Level3aPicker);
        
        // Picker back to Sector
        nav.zoom_out();
        assert_eq!(nav.current_level, ZoomLevel::Level2Sector);
    }
}
