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

    pub fn navigate_to(&mut self, name: &str) -> bool {
        let target_path = if self.current_path == "/" {
            format!("/{}", name)
        } else {
            format!("{}/{}", self.current_path, name)
        };

        if self.nodes.contains_key(&target_path) {
            self.current_path = target_path;
            true
        } else {
            false
        }
    }

    pub fn navigate_up(&mut self) {
        if self.current_path == "/" {
            return;
        }
        
        let last_slash = self.current_path.rfind('/').unwrap_or(0);
        if last_slash == 0 {
            self.current_path = "/".to_string();
        } else {
            self.current_path = self.current_path[..last_slash].to_string();
        }
    }

    pub fn create_file(&mut self, name: &str) {
        let entry = FileNode { name: name.to_string(), is_dir: false, size_bytes: 0 };
        if let Some(nodes) = self.nodes.get_mut(&self.current_path) {
            nodes.push(entry);
        }
    }

    pub fn delete_node(&mut self, name: &str) {
        if let Some(nodes) = self.nodes.get_mut(&self.current_path) {
            nodes.retain(|n| n.name != name);
        }
        // Also remove if it's a directory entry in HashMap
        let target_path = if self.current_path == "/" {
            format!("/{}", name)
        } else {
            format!("{}/{}", self.current_path, name)
        };
        self.nodes.remove(&target_path);
    }

    pub fn create_dir(&mut self, name: &str) {
        let entry = FileNode { name: name.to_string(), is_dir: true, size_bytes: 4096 };
        if let Some(nodes) = self.nodes.get_mut(&self.current_path) {
            nodes.push(entry);
        }
        let target_path = if self.current_path == "/" {
            format!("/{}", name)
        } else {
            format!("{}/{}", self.current_path, name)
        };
        self.nodes.insert(target_path, Vec::new());
    }

    pub fn search(&self, query: &str) -> Vec<(String, FileNode)> {
        let query = query.to_lowercase();
        let mut results = Vec::new();
        for (path, nodes) in &self.nodes {
            for node in nodes {
                if node.name.to_lowercase().contains(&query) {
                    results.push((path.clone(), node.clone()));
                }
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_manipulation() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file("new_script.sh");
        assert!(vfs.get_current_entries().unwrap().iter().any(|n| n.name == "new_script.sh"));
        
        vfs.create_dir("work");
        assert!(vfs.navigate_to("work"));
        assert_eq!(vfs.current_path, "/home/user/work");
        
        vfs.navigate_up();
        vfs.delete_node("work");
        assert!(!vfs.get_current_entries().unwrap().iter().any(|n| n.name == "work"));
    }
}
