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

    pub fn execute_search(&mut self, query: &str) {
        self.current_query = query.to_string();
        self.is_searching = true;
        
        // Mock results for now
        self.results = vec![
            SearchResult {
                id: "test-1".to_string(),
                title: format!("Match for '{}' in Files", query),
                description: "/home/user/tos-config.json".to_string(),
                domain: SearchDomain::Files,
                relevance: 0.95,
                priority_score: 2,
                target_location: SearchTarget::FilePath("/home/user/tos-config.json".to_string()),
            },
            SearchResult {
                id: "web-1".to_string(),
                title: format!("Search Google for '{}'", query),
                description: "External Provider".to_string(),
                domain: SearchDomain::Web,
                relevance: 0.5,
                priority_score: 0,
                target_location: SearchTarget::Url(format!("https://google.com/search?q={}", query)),
            }
        ];
    }

    pub fn clear(&mut self) {
        self.current_query.clear();
        self.results.clear();
        self.is_searching = false;
    }
}
