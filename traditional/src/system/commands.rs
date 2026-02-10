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
                if let Some(level_str) = args.get(0) {
                    if let Ok(level) = level_str.parse::<u8>() {
                        match level {
                            1 => {
                                env.start_zoom_morph(false);
                                env.navigator.zoom_out();
                                if env.navigator.current_level == crate::navigation::zoom::ZoomLevel::Level2Sector {
                                    env.navigator.zoom_out();
                                }
                                return format!("Zooming to Level 1 (ROOT)");
                            }
                            2 => {
                                env.start_zoom_morph(true);
                                env.navigator.zoom_in(0);
                                return format!("Zooming to Level 2 (SECTOR)");
                            }
                            3 => {
                                env.start_zoom_morph(true);
                                env.navigator.zoom_in(0);
                                env.navigator.zoom_in(0);
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
                        env.navigator.split_view(id);
                        return format!("Splitting view with Surface ID: {}", id);
                    }
                }
                format!("Usage: split [secondary_id]")
            }
            "inspect" | "detail" => {
                env.navigator.current_level = crate::navigation::zoom::ZoomLevel::Level4Detail;
                format!("Entering Level 4: Detail Inspection.")
            }
            "help" => {
                format!("Commands: zoom [n], spawn [name], alert [msg], kill [id], split [id], inspect, help")
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
