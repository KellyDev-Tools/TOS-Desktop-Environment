use crate::DesktopEnvironment;
use crate::system::notifications::Priority;
use crate::compositor::SurfaceRole;

pub struct CommandParser;

impl CommandParser {
    pub fn process(env: &mut DesktopEnvironment, input: &str) -> String {
        let input = input.trim();
        if input.is_empty() {
            return String::new();
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();
        let args = &parts[1..];

        match cmd.as_str() {
            "zoom" => {
                env.search_query = None;
                if let Some(level_str) = args.get(0) {
                    if let Ok(level) = level_str.parse::<u8>() {
                        match level {
                            1 => {
                                env.start_zoom_morph(false);
                                env.intelligent_zoom_out();
                                if env.navigator.current_level == crate::navigation::zoom::ZoomLevel::Level2Sector {
                                    env.intelligent_zoom_out();
                                }
                                env.audio.play_sound("zoom_out");
                                return format!("Zooming to Level 1 (ROOT)");
                            }
                            2 => {
                                env.start_zoom_morph(true);
                                if env.navigator.current_level == crate::navigation::zoom::ZoomLevel::Level1Root {
                                    env.navigator.zoom_in(0);
                                }
                                env.audio.play_sound("zoom_in");
                                return format!("Zooming to Level 2 (SECTOR)");
                            }
                            3 => {
                                env.start_zoom_morph(true);
                                if env.navigator.current_level == crate::navigation::zoom::ZoomLevel::Level1Root {
                                    env.navigator.zoom_in(0);
                                    env.navigator.zoom_in(0);
                                } else if env.navigator.current_level == crate::navigation::zoom::ZoomLevel::Level2Sector {
                                    env.navigator.zoom_in(0);
                                }
                                env.audio.play_sound("zoom_in");
                                return format!("Zooming to Level 3 (FOCUS)");
                            }
                            _ => return format!("Error: Invalid Zoom Level {}", level),
                        }
                    }
                }
                format!("Usage: zoom [1|2|3]")
            }
            "spawn" | "launch" => {
                let name = if args.is_empty() { "Terminal" } else { args[0] };
                let sector = env.navigator.active_sector_index.unwrap_or(0);
                let id = env.surfaces.create_surface(name, SurfaceRole::Toplevel, Some(sector));
                format!("Launched '{}' (ID: {}) in Sector {}", name, id, sector)
            }
            "alert" | "notify" => {
                let msg = args.join(" ");
                let priority = if msg.to_lowercase().contains("critical") {
                    Priority::Critical
                } else {
                    Priority::Normal
                };
                env.notifications.push("COMM-LINK", &msg, priority);
                format!("Notification sent.")
            }
            "kill" | "terminate" => {
                if let Some(id_str) = args.get(0) {
                    if let Ok(id) = id_str.parse::<u32>() {
                        env.surfaces.remove_surface(id);
                        return format!("Terminated process ID: {}", id);
                    }
                }
                format!("Usage: kill [id]")
            }
            "split" => {
                if let Some(id_str) = args.get(0) {
                    if let Ok(id) = id_str.parse::<u32>() {
                        if let (Some(sector), Some(app_idx)) = (env.navigator.active_sector_index, env.navigator.active_app_index) {
                            if let Some(primary) = env.surfaces.get_surfaces_in_sector(sector).get(app_idx) {
                                let p_id = primary.id;
                                env.surfaces.add_event(p_id, &format!("Entered split-view with node ID {}", id));
                                env.surfaces.add_event(id, &format!("Entered split-view with node ID {}", p_id));
                            }
                        }
                        env.navigator.split_view(id);
                        return format!("Splitting view with Surface ID: {}", id);
                    }
                }
                format!("Usage: split [secondary_id]")
            }
            "inspect" | "detail" => {
                if let (Some(sector), Some(app_idx)) = (env.navigator.active_sector_index, env.navigator.active_app_index) {
                     if let Some(surface) = env.surfaces.get_surfaces_in_sector(sector).get(app_idx) {
                        let id = surface.id;
                        env.surfaces.add_event(id, "Deep-scan inspection initiated.");
                     }
                }
                env.navigator.current_level = crate::navigation::zoom::ZoomLevel::Level4Detail;
                format!("Entering Level 4: Detail Inspection.")
            }
            "ls" => {
                let path = &env.files.current_path;
                format!("Contents of {}: {:?}", path, env.files.get_current_entries())
            }
            "cd" => {
                if let Some(dir) = args.get(0) {
                    if *dir == ".." {
                        env.files.navigate_up();
                        return format!("Moved to {}", env.files.current_path);
                    } else if env.files.navigate_to(dir) {
                        return format!("Moved to {}", env.files.current_path);
                    }
                }
                format!("Usage: cd [dir|..]")
            }
            "touch" => {
                if let Some(name) = args.get(0) {
                    env.files.create_file(name);
                    return format!("Created file: {}", name);
                }
                format!("Usage: touch [name]")
            }
            "mkdir" => {
                if let Some(name) = args.get(0) {
                    env.files.create_dir(name);
                    return format!("Created directory: {}", name);
                }
                format!("Usage: mkdir [name]")
            }
            "rm" => {
                if let Some(name) = args.get(0) {
                    env.files.delete_node(name);
                    return format!("Removed: {}", name);
                }
                format!("Usage: rm [name]")
            }
            "clone" | "duplicate" => {
                if let (Some(sector), Some(app_idx)) = (env.navigator.active_sector_index, env.navigator.active_app_index) {
                     if let Some(surface) = env.surfaces.get_surfaces_in_sector(sector).get(app_idx) {
                        let new_id = env.surfaces.create_surface(&surface.title, surface.role.clone(), Some(sector));
                        return format!("Cloned '{}' (ID: {})", surface.title, new_id);
                     }
                }
                format!("Error: Must be focusing an app to clone.")
            }
            "swap" => {
                if env.swap_split() {
                    format!("Swapped Split View slots.")
                } else {
                    format!("Error: Not in Split View or cannot determine swap targets.")
                }
            }
            "find" | "search" => {
                let query = args.join(" ");
                if query.is_empty() {
                    env.search_query = None;
                    return "Search cleared.".to_string();
                }
                env.navigator.current_level = crate::navigation::zoom::ZoomLevel::Level1Root;
                env.search_query = Some(query.clone());
                format!("Searching for '{}' across all sectors...", query)
            }
            "clear" => {
                env.search_query = None;
                format!("System filters cleared.")
            }
            "config" | "settings" => {
                if let Some(key) = args.get(0) {
                    let val = args.get(1).map(|v| *v == "on" || *v == "true").unwrap_or(false);
                    match *key {
                        "audio" => {
                            env.settings.audio_enabled = val;
                            env.audio.enabled = val;
                            return format!("Audio Master: {}", if val { "ON" } else { "OFF" });
                        }
                        "chirps" => {
                            env.settings.chirps_enabled = val;
                            env.audio.effects_enabled = val;
                            return format!("Tactile Chirps: {}", if val { "ON" } else { "OFF" });
                        }
                        "debug" => {
                            env.settings.debug_mode = val;
                            return format!("Debug Mode: {}", if val { "ON" } else { "OFF" });
                        }
                        _ => return format!("Unknown setting: {}", key),
                    }
                }
                format!("Current Settings: Audio={:?}, Chirps={:?}, Debug={:?}", 
                    env.settings.audio_enabled, env.settings.chirps_enabled, env.settings.debug_mode)
            }
            "help" => {
                format!("Commands: zoom [n], spawn [name], alert [msg], kill [id], split [id], swap, find [q], config [key] [on/off], help")
            }
            _ => format!("Unknown command: '{}'. Type 'help' for list.", cmd),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DesktopEnvironment;

    #[test]
    fn test_zoom_command() {
        let mut env = DesktopEnvironment::new(None);
        let result = CommandParser::process(&mut env, "zoom 2");
        assert!(result.contains("Level 2"));
        assert_eq!(env.navigator.current_level, crate::navigation::zoom::ZoomLevel::Level2Sector);
    }

    #[test]
    fn test_spawn_command() {
        let mut env = DesktopEnvironment::new(None);
        let result = CommandParser::process(&mut env, "spawn Browser");
        assert!(result.contains("Launched 'Browser'"));
        // Check if surface was actually created
        assert_eq!(env.surfaces.get_surfaces_in_sector(0).len(), 1);
    }

    #[test]
    fn test_alert_command() {
        let mut env = DesktopEnvironment::new(None);
        CommandParser::process(&mut env, "alert Test message");
        assert_eq!(env.notifications.queue.len(), 1);
    }

    #[test]
    fn test_kill_command() {
        let mut env = DesktopEnvironment::new(None);
        let id = env.surfaces.create_surface("Test", SurfaceRole::Toplevel, Some(0));
        assert_eq!(env.surfaces.get_all_surface_titles().len(), 1);
        
        CommandParser::process(&mut env, &format!("kill {}", id));
        assert_eq!(env.surfaces.get_all_surface_titles().len(), 0);
    }
}
