use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceHome {
    pub featured: Vec<MarketplaceModuleSummary>,
    pub categories: Vec<MarketplaceCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceCategory {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub module_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceModuleSummary {
    pub id: String,
    pub name: String,
    pub module_type: String,
    pub author: String,
    pub icon: Option<String>,
    pub rating: f32,
    pub price: String, // "Free", "$5", etc.
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceModuleDetail {
    pub summary: MarketplaceModuleSummary,
    pub description: String,
    pub screenshots: Vec<String>,
    pub permissions: Vec<String>,
    pub reviews: Vec<MarketplaceReview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceReview {
    pub author: String,
    pub rating: u8,
    pub comment: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub module_id: String,
    pub progress: f32,  // 0.0 to 1.0
    pub status: String, // "Downloading", "Verifying", "Installing", "Complete", "Error"
    pub error: Option<String>,
}
