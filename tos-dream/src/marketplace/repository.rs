//! Repository Management
//! 
//! Handles repository index fetching, parsing, and package lookup.
//! Supports multiple repositories with priority-based resolution.

use super::{MarketplaceError, PackageMetadata, RepositoryConfig, PackageType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Repository index format (JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryIndex {
    /// Repository metadata
    pub repository: RepositoryMeta,
    /// Available packages
    pub packages: Vec<PackageEntry>,
    /// Index format version
    pub version: String,
}

/// Repository metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMeta {
    /// Repository name
    pub name: String,
    /// Repository description
    pub description: String,
    /// Base URL for downloads
    pub base_url: String,
    /// Last updated timestamp
    pub last_updated: String,
    /// TOS version compatibility
    pub tos_version: String,
}

/// Package entry in repository index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageEntry {
    /// Package name
    pub name: String,
    /// Available versions
    pub versions: Vec<VersionEntry>,
    /// Package type
    pub package_type: PackageType,
    /// Short description
    pub description: String,
    /// Author
    pub author: String,
    /// License
    pub license: String,
    /// Tags
    pub tags: Vec<String>,
}

/// Version entry for a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    /// Semantic version
    pub version: String,
    /// Download path (relative to base_url)
    pub download_path: String,
    /// SHA256 checksum
    pub sha256: String,
    /// Signature (optional)
    pub signature: Option<String>,
    /// Dependencies
    pub dependencies: Vec<String>,
    /// Minimum TOS version
    pub min_tos_version: Option<String>,
    /// Size in bytes
    pub size: u64,
    /// Release date
    pub released_at: String,
    /// Changelog
    pub changelog: Option<String>,
}

/// Manages multiple repositories
pub struct RepositoryManager {
    /// Configured repositories
    repositories: HashMap<String, Repository>,
}

impl RepositoryManager {
    /// Create a new repository manager
    pub fn new(configs: Vec<RepositoryConfig>) -> Self {
        let mut repositories = HashMap::new();
        
        for config in configs {
            let repo = Repository::new(config);
            repositories.insert(repo.name().to_string(), repo);
        }
        
        Self { repositories }
    }
    
    /// Add a repository
    pub fn add_repository(&mut self, config: RepositoryConfig) {
        let repo = Repository::new(config);
        self.repositories.insert(repo.name().to_string(), repo);
    }
    
    /// Remove a repository
    pub fn remove_repository(&mut self, name: &str) {
        self.repositories.remove(name);
    }
    
    /// Get a repository by name
    pub fn get_repository(&self, name: &str) -> Option<&Repository> {
        self.repositories.get(name)
    }
    
    /// Get all enabled repositories sorted by priority
    pub fn enabled_repositories(&self) -> Vec<&Repository> {
        let mut repos: Vec<&Repository> = self.repositories
            .values()
            .filter(|r| r.is_enabled())
            .collect();
        
        repos.sort_by_key(|r| r.priority());
        repos
    }
    
    /// List all repository names
    pub fn repository_names(&self) -> Vec<String> {
        self.repositories.keys().cloned().collect()
    }
    
    /// Refresh all repository indexes
    pub async fn refresh_all(&mut self) -> Result<Vec<String>, MarketplaceError> {
        let mut refreshed = Vec::new();
        
        for repo in self.repositories.values_mut() {
            match repo.refresh().await {
                Ok(_) => refreshed.push(repo.name().to_string()),
                Err(e) => tracing::warn!("Failed to refresh repository {}: {}", repo.name(), e),
            }
        }
        
        Ok(refreshed)
    }
}

/// Individual repository
pub struct Repository {
    /// Repository configuration
    config: RepositoryConfig,
    /// Cached index (if fetched)
    cached_index: Option<RepositoryIndex>,
    /// Last fetch timestamp
    last_fetch: Option<std::time::Instant>,
}

impl Repository {
    /// Create a new repository from configuration
    pub fn new(config: RepositoryConfig) -> Self {
        Self {
            config,
            cached_index: None,
            last_fetch: None,
        }
    }
    
    /// Get repository name
    pub fn name(&self) -> &str {
        &self.config.name
    }
    
    /// Get repository URL
    pub fn url(&self) -> &str {
        &self.config.url
    }
    
    /// Check if repository is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    /// Get repository priority
    pub fn priority(&self) -> u32 {
        self.config.priority
    }
    
    /// Refresh the repository index
    pub async fn refresh(&mut self) -> Result<(), MarketplaceError> {
        let index_url = format!("{}/index.json", self.config.url);
        
        let client = reqwest::Client::new();
        let mut request = client.get(&index_url);
        
        // Add authentication if available
        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(MarketplaceError::Network(
                format!("HTTP {}: {}", response.status(), index_url)
            ));
        }
        
        let index: RepositoryIndex = response.json().await?;
        self.cached_index = Some(index);
        self.last_fetch = Some(std::time::Instant::now());
        
        tracing::info!("Repository {} refreshed successfully", self.config.name);
        Ok(())
    }
    
    /// Search for packages matching query
    pub async fn search(&self, query: &str) -> Result<Vec<PackageMetadata>, MarketplaceError> {
        let index = self.get_index().await?;
        
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        for entry in &index.packages {
            // Match against name, description, or tags
            if entry.name.to_lowercase().contains(&query_lower)
                || entry.description.to_lowercase().contains(&query_lower)
                || entry.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            {
                // Get latest version
                if let Some(latest) = entry.versions.first() {
                    results.push(self.entry_to_metadata(entry, latest, &index.repository.base_url));
                }
            }
        }
        
        Ok(results)
    }
    
    /// Find a specific package with version constraint
    pub async fn find_package(
        &self,
        name: &str,
        version_constraint: &str,
    ) -> Result<PackageMetadata, MarketplaceError> {
        let index = self.get_index().await?;
        
        let entry = index.packages
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| MarketplaceError::NotFound(
                format!("Package {} not found in repository {}", name, self.config.name)
            ))?;
        
        // Find matching version
        let version = if version_constraint == "latest" {
            entry.versions.first()
        } else {
            // Simple version matching (exact match or semver parsing)
            entry.versions.iter()
                .find(|v| v.version == version_constraint || 
                    self.version_matches(&v.version, version_constraint))
        };
        
        let version = version.ok_or_else(|| MarketplaceError::NotFound(
            format!("Version {} of package {} not found", version_constraint, name)
        ))?;
        
        Ok(self.entry_to_metadata(entry, version, &index.repository.base_url))
    }
    
    /// Get all packages from this repository
    pub async fn list_packages(&self) -> Result<Vec<PackageMetadata>, MarketplaceError> {
        let index = self.get_index().await?;
        
        let mut results = Vec::new();
        
        for entry in &index.packages {
            if let Some(latest) = entry.versions.first() {
                results.push(self.entry_to_metadata(entry, latest, &index.repository.base_url));
            }
        }
        
        Ok(results)
    }
    
    /// Get the repository index (fetch if needed)
    async fn get_index(&self) -> Result<&RepositoryIndex, MarketplaceError> {
        // Check if we need to fetch
        let needs_fetch = self.cached_index.is_none() || 
            self.last_fetch.map_or(true, |t| t.elapsed().as_secs() > 3600); // 1 hour cache
        
        if needs_fetch {
            // Need to refresh - this requires mutable self, so we return an error
            // In practice, the manager should pre-fetch repositories
            return Err(MarketplaceError::Network(
                "Repository index not cached. Call refresh() first.".to_string()
            ));
        }
        
        Ok(self.cached_index.as_ref().unwrap())
    }
    
    /// Convert a package entry to metadata
    fn entry_to_metadata(
        &self,
        entry: &PackageEntry,
        version: &VersionEntry,
        base_url: &str,
    ) -> PackageMetadata {
        PackageMetadata {
            name: entry.name.clone(),
            version: version.version.clone(),
            description: entry.description.clone(),
            author: entry.author.clone(),
            license: entry.license.clone(),
            package_type: entry.package_type,
            download_url: format!("{}{}", base_url, version.download_path),
            sha256: version.sha256.clone(),
            signature: version.signature.clone(),
            dependencies: version.dependencies.clone(),
            min_tos_version: version.min_tos_version.clone(),
            tags: entry.tags.clone(),
            size: version.size,
            created_at: version.released_at.clone(),
        }
    }
    
    /// Check if a version matches a constraint (basic semver support)
    fn version_matches(&self, version: &str, constraint: &str) -> bool {
        // Simple implementation - just check if version starts with constraint
        // For full semver support, use the semver crate
        if constraint.starts_with(">=") {
            version >= constraint[2..].trim()
        } else if constraint.starts_with(">") {
            version > constraint[1..].trim()
        } else if constraint.starts_with("<=") {
            version <= constraint[2..].trim()
        } else if constraint.starts_with("<") {
            version < constraint[1..].trim()
        } else if constraint.starts_with("~") {
            // Compatible version (same major.minor)
            let prefix = &constraint[1..].trim();
            version.starts_with(prefix)
        } else if constraint.starts_with("^") {
            // Compatible with major version
            let parts: Vec<&str> = constraint[1..].trim().split('.').collect();
            if parts.len() >= 1 {
                version.starts_with(parts[0])
            } else {
                false
            }
        } else {
            // Exact match
            version == constraint
        }
    }
}

/// Create a sample repository index for testing
pub fn create_sample_index() -> RepositoryIndex {
    RepositoryIndex {
        repository: RepositoryMeta {
            name: "tos-official".to_string(),
            description: "Official TOS Module Repository".to_string(),
            base_url: "https://marketplace.tos.dev/packages/".to_string(),
            last_updated: "2024-01-15T10:00:00Z".to_string(),
            tos_version: "0.1.0".to_string(),
        },
        packages: vec![
            PackageEntry {
                name: "terminal-enhanced".to_string(),
                versions: vec![
                    VersionEntry {
                        version: "1.0.0".to_string(),
                        download_path: "terminal-enhanced-1.0.0.tos-appmodel".to_string(),
                        sha256: "abc123...".to_string(),
                        signature: None,
                        dependencies: vec![],
                        min_tos_version: Some("0.1.0".to_string()),
                        size: 10240,
                        released_at: "2024-01-15".to_string(),
                        changelog: Some("Initial release".to_string()),
                    },
                ],
                package_type: PackageType::ApplicationModel,
                description: "Enhanced terminal with custom actions".to_string(),
                author: "TOS Team".to_string(),
                license: "MIT".to_string(),
                tags: vec!["terminal".to_string(), "productivity".to_string()],
            },
        ],
        version: "1.0".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_repository_index_serialization() {
        let index = create_sample_index();
        let json = serde_json::to_string(&index).unwrap();
        assert!(json.contains("tos-official"));
        assert!(json.contains("terminal-enhanced"));
    }
    
    #[test]
    fn test_repository_manager() {
        let configs = vec![
            RepositoryConfig {
                name: "repo1".to_string(),
                url: "https://repo1.example.com".to_string(),
                enabled: true,
                priority: 1,
                auth_token: None,
            },
            RepositoryConfig {
                name: "repo2".to_string(),
                url: "https://repo2.example.com".to_string(),
                enabled: false,
                priority: 2,
                auth_token: Some("token".to_string()),
            },
        ];
        
        let manager = RepositoryManager::new(configs);
        assert_eq!(manager.repository_names().len(), 2);
        
        let enabled = manager.enabled_repositories();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name(), "repo1");
    }
    
    #[test]
    fn test_version_matching() {
        let repo = Repository::new(RepositoryConfig {
            name: "test".to_string(),
            url: "https://test.com".to_string(),
            enabled: true,
            priority: 1,
            auth_token: None,
        });
        
        assert!(repo.version_matches("1.0.0", "1.0.0")); // exact
        assert!(repo.version_matches("1.0.0", ">=1.0.0")); // >=
        assert!(!repo.version_matches("0.9.0", ">=1.0.0")); // >= fail
        assert!(repo.version_matches("1.0.0", "~1.0")); // ~ compatible
        assert!(repo.version_matches("1.0.0", "^1")); // ^ compatible
    }
}
