pub mod gpu;
pub mod wayland;

use std::collections::HashMap;
use crate::navigation::zoom::ZoomLevel;

#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceRole {
    Toplevel, // A standard application window
    Popup,    // A menu or tooltip
    OSD,      // On-screen display (like volume indicator)
    Background,
}

pub fn id_to_noise(id: u32, offset: u32) -> u32 {
    let mut x = (id + offset) as i64;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x.abs() as u32
}

#[derive(Debug, Clone)]
pub struct TosSurface {
    pub id: u32,
    pub title: String,
    pub app_class: String, // For grouping windows (e.g., "Terminal")
    pub role: SurfaceRole,
    pub sector_id: Option<usize>,
    pub history: Vec<String>, // Event history for Level 4 Node History view
    pub cpu_usage: u8,
    pub mem_usage: u8,
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
        // Use everything before first space or the whole title as app_class
        let app_class = title.split_whitespace().next().unwrap_or("App").to_string();
        
        let surface = TosSurface {
            id,
            title: title.to_string(),
            app_class,
            role,
            sector_id,
            history: vec![format!("Surface created: {}", title)],
            cpu_usage: 5,
            mem_usage: 10,
        };
        self.surfaces.insert(id, surface);
        self.next_id += 1;
        id
    }

    pub fn update_telemetry(&mut self) {
        // Mock oscillation for demo
        for surface in self.surfaces.values_mut() {
            surface.cpu_usage = (surface.cpu_usage as i16 + (id_to_noise(surface.id, 0) % 10) as i16 - 5)
                .clamp(0, 100) as u8;
            surface.mem_usage = (surface.mem_usage as i16 + (id_to_noise(surface.id, 1) % 4) as i16 - 2)
                .clamp(0, 100) as u8;
            
            surface.history.push(format!("Tick - CPU: {}% MEM: {}%", surface.cpu_usage, surface.mem_usage));
            if surface.history.len() > 10 {
                surface.history.remove(0);
            }
        }
    }

    pub fn get_surface(&self, id: u32) -> Option<&TosSurface> {
        self.surfaces.get(&id)
    }

    pub fn get_surfaces_in_sector(&self, sector_id: usize) -> Vec<TosSurface> {
        let mut surfaces: Vec<TosSurface> = self.surfaces
            .values()
            .filter(|s| s.sector_id == Some(sector_id))
            .cloned()
            .collect();
        surfaces.sort_by_key(|s| s.id);
        surfaces
    }

    pub fn get_surfaces_in_group(&self, app_class: &str) -> Vec<TosSurface> {
        let mut surfaces: Vec<TosSurface> = self.surfaces
            .values()
            .filter(|s| s.app_class == app_class)
            .cloned()
            .collect();
        surfaces.sort_by_key(|s| s.id);
        surfaces
    }

    pub fn find_surfaces(&self, query: &str) -> Vec<TosSurface> {
        let query = query.to_lowercase();
        let mut surfaces: Vec<TosSurface> = self.surfaces
            .values()
            .filter(|s| s.title.to_lowercase().contains(&query) || s.app_class.to_lowercase().contains(&query))
            .cloned()
            .collect();
        surfaces.sort_by_key(|s| s.id);
        surfaces
    }

    pub fn get_all_surface_titles(&self) -> Vec<String> {
        self.surfaces.values().map(|s| s.title.clone()).collect()
    }

    pub fn add_event(&mut self, id: u32, event: &str) {
        if let Some(s) = self.surfaces.get_mut(&id) {
            s.history.push(event.to_string());
            if s.history.len() > 10 {
                s.history.remove(0); // Keep last 10
            }
        }
    }

    pub fn remove_surface(&mut self, id: u32) {
        self.surfaces.remove(&id);
    }

    pub fn move_to_sector(&mut self, id: u32, sector_id: usize) -> bool {
        if let Some(s) = self.surfaces.get_mut(&id) {
            s.sector_id = Some(sector_id);
            s.history.push(format!("Orchestration: Moved to Sector {}", sector_id));
            true
        } else {
            false
        }
    }
}

pub struct SpatialMapper;

impl SpatialMapper {
    pub fn get_layout(
        manager: &SurfaceManager,
        level: ZoomLevel,
        active_sector: Option<usize>,
        primary_id: Option<u32>,
        secondary_id: Option<u32>,
        sectors_count: usize,
    ) -> Vec<SurfaceLayout> {
        if level == ZoomLevel::Level1Root {
            let mut layouts = Vec::new();
            for i in 0..sectors_count {
                let x = (i % 2) as u16;
                let y = (i / 2) as u16;
                layouts.push(SurfaceLayout {
                    surface: TosSurface {
                        id: 1000 + i as u32,
                        title: format!("Sector {}", i),
                        app_class: "Sector".to_string(),
                        role: SurfaceRole::Toplevel,
                        sector_id: Some(i),
                        history: Vec::new(),
                        cpu_usage: 0,
                        mem_usage: 0,
                    },
                    grid_x: x * 1, // Scaling sectors
                    grid_y: y * 1,
                    width: 1,
                    height: 1,
                });
            }
            return layouts;
        }

        let surfaces = match level {
            ZoomLevel::Level1Root => Vec::new(),
            ZoomLevel::Level2Sector => {
                if let Some(sector_id) = active_sector {
                    manager.get_surfaces_in_sector(sector_id)
                } else {
                    Vec::new()
                }
            }
            ZoomLevel::Level3Focus | ZoomLevel::Level4Detail | ZoomLevel::Level5Buffer => {
                if let Some(id) = primary_id {
                    manager.get_surface(id).into_iter().cloned().collect()
                } else {
                    Vec::new()
                }
            }
            ZoomLevel::Level3aPicker => {
                if let Some(id) = primary_id {
                    if let Some(s) = manager.get_surface(id) {
                        manager.get_surfaces_in_group(&s.app_class)
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            ZoomLevel::Level3Split => {
                let mut split = Vec::new();
                if let Some(id) = primary_id {
                    if let Some(s) = manager.get_surface(id) { split.push(s.clone()); }
                }
                if let Some(id) = secondary_id {
                    if let Some(s) = manager.get_surface(id) { split.push(s.clone()); }
                }
                split
            }
        };

        let mut layouts = Vec::new();

        for (i, surface) in surfaces.into_iter().enumerate() {
            let (gx, gy, w, h) = match level {
                ZoomLevel::Level2Sector => {
                    let count = manager.get_surfaces_in_sector(active_sector.unwrap_or(0)).len();
                    match count {
                        1 => (0, 0, 3, 3), // Single centered app
                        2 => {
                            if i == 0 { (0, 0, 2, 3) } else { (2, 0, 1, 3) }
                        }
                        3 => {
                            if i == 0 { (0, 0, 2, 2) }
                            else if i == 1 { (2, 0, 1, 2) }
                            else { (0, 2, 3, 1) }
                        }
                        _ => {
                            // Standard Asymmetric Grid
                            if i == 0 { (0, 0, 2, 2) }      // Featured
                            else if i == 1 { (2, 0, 1, 1) } // Top Right
                            else if i == 2 { (2, 1, 1, 1) } // Mid Right
                            else {
                                // Bottom row or overflow
                                let over = (i - 3) as u16;
                                let x = over % 3;
                                let y = 2 + (over / 3);
                                (x, y, 1, 1)
                            }
                        }
                    }
                }
                ZoomLevel::Level3Focus | ZoomLevel::Level4Detail | ZoomLevel::Level5Buffer => (0, 0, 3, 3), // Full span
                ZoomLevel::Level3aPicker => {
                    // Display grouped windows in a smaller grid
                    let x = (i % 2) as u16 + 1; // Slightly offset/centered
                    let y = (i / 2) as u16 + 1;
                    (x, y, 1, 1)
                }
                ZoomLevel::Level3Split => {
                    if i == 0 { (0, 0, 2, 3) } else { (2, 0, 1, 3) }
                }
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
            Some(term_id),
            None,
            4
        );
        
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].surface.title, "Terminal");
        assert_eq!(layouts[0].width, 3);
    }
}
