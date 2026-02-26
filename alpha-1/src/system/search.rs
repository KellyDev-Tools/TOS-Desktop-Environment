use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Search domains as defined in §3.4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchDomain {
    Surfaces,
    Files,
    Logs,
    Commands,
    Settings,
    Sectors,
    Help,
    Contacts,
    Modules,
    Notifications,
    Web,
}

/// A single search result (§3.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub description: String,
    pub domain: SearchDomain,
    pub relevance: f32, // 0.0 - 1.0 relevance score
    pub priority_score: u8, // §5.1 priority weighting
    pub target_location: SearchTarget,
}

/// Navigation target for search results (§3.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchTarget {
    Viewport(Uuid),
    Sector(Uuid),
    FilePath(String),
    Command(String),
    Url(String),
    Module(String),
}

/// Unified Search Manager (§3.4)
#[derive(Debug, Default)]
pub struct SearchManager {
    pub current_query: String,
    pub results: Vec<SearchResult>,
    pub is_searching: bool,
}

impl SearchManager {
    pub fn new() -> Self {
        Self {
            current_query: String::new(),
            results: Vec::new(),
            is_searching: false,
        }
    }

    pub fn start_search(&mut self, query: &str) {
        self.current_query = query.to_string();
        self.is_searching = true;
        self.results.clear();
    }

    pub fn add_results(&mut self, mut results: Vec<SearchResult>) {
        self.results.append(&mut results);
        // Sort by priority/relevance if needed, for now just append
        self.results.sort_by(|a, b| b.priority_score.cmp(&a.priority_score));
    }

    pub fn clear(&mut self) {
        self.current_query.clear();
        self.results.clear();
        self.is_searching = false;
    }
}
