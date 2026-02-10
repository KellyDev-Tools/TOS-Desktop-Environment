use std::collections::HashMap;
use crate::navigation::zoom::ZoomLevel;

#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceRole {
    Toplevel, // A standard application window
    Popup,    // A menu or tooltip
    OSD,      // On-screen display (like volume indicator)
    Background,
}

#[derive(Debug, Clone)]
pub struct TosSurface {
    pub id: u32,
    pub title: String,
    pub role: SurfaceRole,
    pub sector_id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct SurfaceLayout {
    pub surface: TosSurface,
    pub grid_x: u16,
    pub grid_y: u16,
    pub width: u16,
    pub height: u16,
}

pub struct SurfaceManager {
    surfaces: HashMap<u32, TosSurface>,
    next_id: u32,
}

impl SurfaceManager {
    pub fn new() -> Self {
        Self {
            surfaces: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create_surface(&mut self, title: &str, role: SurfaceRole, sector_id: Option<usize>) -> u32 {
        let id = self.next_id;
        let surface = TosSurface {
            id,
            title: title.to_string(),
            role,
            sector_id,
        };
        self.surfaces.insert(id, surface);
        self.next_id += 1;
        id
    }

    pub fn get_surface(&self, id: u32) -> Option<&TosSurface> {
        self.surfaces.get(&id)
    }

    pub fn get_surfaces_in_sector(&self, sector_id: usize) -> Vec<TosSurface> {
        self.surfaces
            .values()
            .filter(|s| s.sector_id == Some(sector_id))
            .cloned()
            .collect()
    }

    pub fn get_all_surface_titles(&self) -> Vec<String> {
        self.surfaces.values().map(|s| s.title.clone()).collect()
    }

    pub fn remove_surface(&mut self, id: u32) {
        self.surfaces.remove(&id);
    }
}

pub struct SpatialMapper;

impl SpatialMapper {
    pub fn get_layout(
        manager: &SurfaceManager,
        level: ZoomLevel,
        active_sector: Option<usize>,
        active_app_id: Option<u32>,
    ) -> Vec<SurfaceLayout> {
        let surfaces = match level {
            ZoomLevel::Level1Root => Vec::new(),
            ZoomLevel::Level2Sector => {
                if let Some(sector_id) = active_sector {
                    manager.get_surfaces_in_sector(sector_id)
                } else {
                    Vec::new()
                }
            }
            ZoomLevel::Level3Focus | ZoomLevel::Level3aPicker => {
                if let Some(id) = active_app_id {
                    manager.get_surface(id).into_iter().cloned().collect()
                } else {
                    Vec::new()
                }
            }
        };

        let mut layouts = Vec::new();
        let cols = 3;

        for (i, surface) in surfaces.into_iter().enumerate() {
            let (gx, gy, w, h) = match level {
                ZoomLevel::Level2Sector => {
                    let x = (i % cols) as u16;
                    let y = (i / cols) as u16;
                    (x, y, 1, 1)
                }
                ZoomLevel::Level3Focus => (0, 0, 3, 3),
                _ => (0, 0, 1, 1),
            };

            layouts.push(SurfaceLayout {
                surface,
                grid_x: gx,
                grid_y: gy,
                width: w,
                height: h,
            });
        }
        layouts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_creation() {
        let mut mgr = SurfaceManager::new();
        let id = mgr.create_surface("Terminal", SurfaceRole::Toplevel, Some(0));
        let surface = mgr.get_surface(id).unwrap();
        assert_eq!(surface.title, "Terminal");
        assert_eq!(surface.sector_id, Some(0));
    }

    #[test]
    fn test_sector_filtering() {
        let mut mgr = SurfaceManager::new();
        mgr.create_surface("Terminal", SurfaceRole::Toplevel, Some(0));
        mgr.create_surface("Files", SurfaceRole::Toplevel, Some(0));
        mgr.create_surface("Browser", SurfaceRole::Toplevel, Some(1));

        let sector_0 = mgr.get_surfaces_in_sector(0);
        assert_eq!(sector_0.len(), 2);
    }

    #[test]
    fn test_spatial_mapper_layout() {
        let mut mgr = SurfaceManager::new();
        let term_id = mgr.create_surface("Terminal", SurfaceRole::Toplevel, Some(0));
        
        let layouts = SpatialMapper::get_layout(
            &mgr, 
            ZoomLevel::Level3Focus, 
            Some(0), 
            Some(term_id)
        );
        
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].surface.title, "Terminal");
        assert_eq!(layouts[0].width, 3);
    }
}
