use std::sync::{Arc, RwLock};
use std::thread;
use std::path::PathBuf;
use walkdir::WalkDir;
use regex::Regex;

#[derive(Clone, Debug)]
pub struct SearchHit {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

pub struct SearchService {
    index: Arc<RwLock<Vec<SearchHit>>>,
}

impl SearchService {
    pub fn new() -> Self {
        let index = Arc::new(RwLock::new(Vec::new()));
        let index_clone = index.clone();
        
        // Spawn background indexer
        thread::spawn(move || {
            let mut local_index = Vec::new();
            
            // Base directories to scan - limit for Alpha-2 implementation
            let dirs_to_scan = vec![
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
                dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join(".config/tos"),
            ];

            for root in dirs_to_scan {
                if !root.exists() { continue; }
                
                // Fast shallow walk for alpha to prevent blocking
                for entry in WalkDir::new(&root).max_depth(3).into_iter().filter_map(|e| e.ok()) {
                    let path_str = entry.path().to_string_lossy().to_string();
                    
                    // Exclude massive generic dirs
                    if path_str.contains(".git") || path_str.contains("node_modules") || path_str.contains("target") {
                        continue;
                    }
                    
                    local_index.push(SearchHit {
                        path: path_str,
                        is_dir: entry.file_type().is_dir(),
                        size: entry.metadata().map(|m: std::fs::Metadata| m.len()).unwrap_or(0),
                    });
                }
            }
            
            // Atomically swap the new index in
            if let Ok(mut lock) = index_clone.write() {
                *lock = local_index;
            }
        });

        Self {
            index,
        }
    }

    pub fn query(&self, pattern: &str) -> Vec<SearchHit> {
        let lock = match self.index.read() {
            Ok(l) => l,
            Err(_) => return vec![],
        };
        
        let safe_pattern = regex::escape(pattern);
        let re = match Regex::new(&format!("(?i){}", safe_pattern)) {
            Ok(r) => r,
            Err(_) => return vec![],
        };

        lock.iter()
            .filter(|hit| re.is_match(&hit.path))
            .take(50) // Cap results sent over IPC
            .cloned()
            .collect()
    }

    /// Simulates semantic embedding-based retrieval by tokenising natural language queries
    /// and performing an overlap scoring, mimicking TF-IDF or dot-product embeddings for Alpha-2
    pub fn semantic_query(&self, prompt: &str) -> Vec<SearchHit> {
        let lock = match self.index.read() {
            Ok(l) => l,
            Err(_) => return vec![],
        };

        let stop_words = ["find", "search", "where", "is", "for", "the", "my", "a", "an", "all", "show", "me"];
        let words: Vec<String> = prompt.split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
            .filter(|w| !w.is_empty() && !stop_words.contains(&w.as_str()))
            .collect();

        if words.is_empty() {
            return vec![];
        }

        let mut scored_hits: Vec<(&SearchHit, usize)> = lock.iter().map(|hit| {
            let path_lower = hit.path.to_lowercase();
            // Score by how many semantic tokens exist in the file path
            let score = words.iter().filter(|w| path_lower.contains(*w)).count();
            (hit, score)
        }).filter(|(_, score)| *score > 0).collect();

        // Sort by highest overlap (mock highest cosine similarity)
        scored_hits.sort_by(|a, b| b.1.cmp(&a.1));

        scored_hits.into_iter().take(20).map(|(h, _)| h.clone()).collect()
    }
}
