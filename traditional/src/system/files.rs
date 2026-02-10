use std::collections::HashMap;

// Based on "File Management.md"
// Level N Spatial File Browser

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: usize,
}

pub struct VirtualFileSystem {
    pub current_path: String,
    // Mock filesystem for demo
    pub nodes: HashMap<String, Vec<FileNode>>,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        nodes.insert("/".to_string(), vec![
            FileNode { name: "home".to_string(), is_dir: true, size_bytes: 4096 },
            FileNode { name: "etc".to_string(), is_dir: true, size_bytes: 4096 },
        ]);
        nodes.insert("/home".to_string(), vec![
            FileNode { name: "user".to_string(), is_dir: true, size_bytes: 4096 },
        ]);
        nodes.insert("/home/user".to_string(), vec![
            FileNode { name: "documents".to_string(), is_dir: true, size_bytes: 4096 },
            FileNode { name: "notes.txt".to_string(), is_dir: false, size_bytes: 1024 },
        ]);

        Self {
            current_path: "/home/user".to_string(),
            nodes,
        }
    }

    pub fn get_current_entries(&self) -> Option<&Vec<FileNode>> {
        self.nodes.get(&self.current_path)
    }

    pub fn list_current_dir(&self) {
        println!("[Files] Contents of {}:", self.current_path);
        if let Some(contents) = self.get_current_entries() {
            for node in contents {
                let type_str = if node.is_dir { "<DIR>" } else { "     " };
                println!("  {} {} ({} bytes)", type_str, node.name, node.size_bytes);
            }
        } else {
            println!("  (Empty or access denied)");
        }
    }

    pub fn navigate_up(&mut self) {
        if self.current_path == "/" {
            println!("[Files] Already at root.");
            return;
        }
        
        // Simple string manipulation for mock path
        let last_slash = self.current_path.rfind('/').unwrap_or(0);
        if last_slash == 0 {
            self.current_path = "/".to_string();
        } else {
            self.current_path = self.current_path[..last_slash].to_string();
        }
        println!("[Files] Navigated up to: {}", self.current_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_path() {
        let vfs = VirtualFileSystem::new();
        assert_eq!(vfs.current_path, "/home/user");
    }

    #[test]
    fn test_list_entries() {
        let vfs = VirtualFileSystem::new();
        let entries = vfs.get_current_entries().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "documents");
        assert!(entries[0].is_dir);
        assert_eq!(entries[1].name, "notes.txt");
        assert!(!entries[1].is_dir);
    }

    #[test]
    fn test_navigate_up() {
        let mut vfs = VirtualFileSystem::new();
        assert_eq!(vfs.current_path, "/home/user");

        vfs.navigate_up();
        assert_eq!(vfs.current_path, "/home");

        vfs.navigate_up();
        assert_eq!(vfs.current_path, "/");

        // Should stay at root
        vfs.navigate_up();
        assert_eq!(vfs.current_path, "/");
    }

    #[test]
    fn test_root_entries() {
        let mut vfs = VirtualFileSystem::new();
        vfs.current_path = "/".to_string();
        let entries = vfs.get_current_entries().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "home");
        assert_eq!(entries[1].name, "etc");
    }
}
