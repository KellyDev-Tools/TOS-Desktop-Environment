//! Dependency Resolution
//! 
//! Handles dependency graph construction, cycle detection, and topological sorting
//! for determining installation order of packages.

use super::{MarketplaceError, PackageMetadata, RepositoryManager};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Dependency resolver for package dependencies
#[derive(Clone)]
pub struct DependencyResolver {
    /// Maximum depth for dependency resolution
    max_depth: u32,
    /// Cache for resolved dependencies
    resolution_cache: HashMap<String, PackageMetadata>,
}

/// Dependency graph for tracking package relationships
#[derive(Debug)]
pub struct DependencyGraph {
    /// Nodes (packages)
    pub nodes: HashMap<String, PackageNode>,
    /// Edges (dependencies)
    pub edges: HashMap<String, Vec<String>>,
    /// Root package
    pub root: String,
}

/// Package node in dependency graph
#[derive(Debug, Clone)]
pub struct PackageNode {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package metadata (if resolved)
    pub metadata: Option<PackageMetadata>,
    /// Resolution state
    pub state: ResolutionState,
}

/// Resolution state for a package
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionState {
    /// Not yet resolved
    Unresolved,
    /// Currently resolving
    Resolving,
    /// Successfully resolved
    Resolved,
    /// Failed to resolve
    Failed,
}

/// Dependency constraint
#[derive(Debug, Clone)]
pub struct DependencyConstraint {
    /// Package name
    pub name: String,
    /// Version constraint
    pub version_constraint: String,
    /// Whether optional
    pub optional: bool,
    /// Required features
    pub features: Vec<String>,
}

/// Resolution result
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// Resolved packages in installation order
    pub packages: Vec<PackageMetadata>,
    /// Warnings
    pub warnings: Vec<String>,
    /// Circular dependencies detected
    pub circular_deps: Vec<Vec<String>>,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new() -> Self {
        Self {
            max_depth: 50,
            resolution_cache: HashMap::new(),
        }
    }
    
    /// Set maximum resolution depth
    pub fn with_max_depth(mut self, depth: u32) -> Self {
        self.max_depth = depth;
        self
    }
    
    /// Resolve dependencies for a package
    pub async fn resolve(
        &self,
        package: &PackageMetadata,
        _repository_manager: &RepositoryManager,
    ) -> Result<Vec<PackageMetadata>, MarketplaceError> {
        // Build dependency graph
        let mut graph = DependencyGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            root: package.name.clone(),
        };
        
        // Add root node
        graph.nodes.insert(
            package.name.clone(),
            PackageNode {
                name: package.name.clone(),
                version: package.version.clone(),
                metadata: Some(package.clone()),
                state: ResolutionState::Resolved,
            },
        );
        
        // BFS to resolve all dependencies
        let mut queue: VecDeque<(String, u32)> = VecDeque::new();
        let mut visited: HashSet<String> = HashSet::new();
        
        queue.push_back((package.name.clone(), 0));
        visited.insert(package.name.clone());
        
        while let Some((pkg_name, depth)) = queue.pop_front() {
            if depth > self.max_depth {
                return Err(MarketplaceError::Dependency(
                    format!("Maximum dependency depth exceeded (>{})", self.max_depth)
                ));
            }
            
            // Get package metadata
            let metadata = if let Some(node) = graph.nodes.get(&pkg_name) {
                if let Some(ref meta) = node.metadata {
                    Some(meta.clone())
                } else {
                    // Try to resolve from repositories
                    match self.find_package_in_repos(&pkg_name, &node.version, _repository_manager).await {
                        Ok(meta) => {
                            // Update node in graph
                            if let Some(node_mut) = graph.nodes.get_mut(&pkg_name) {
                                node_mut.metadata = Some(meta.clone());
                                node_mut.state = ResolutionState::Resolved;
                            }
                            Some(meta)
                        }
                        Err(e) => {
                            tracing::error!("Failed to resolve dependency {}: {}", pkg_name, e);
                            if let Some(node_mut) = graph.nodes.get_mut(&pkg_name) {
                                node_mut.state = ResolutionState::Failed;
                            }
                            None
                        }
                    }
                }
            } else {
                None
            };

            if let Some(metadata) = metadata {
                // Resolve each dependency
                let mut deps = Vec::new();
                for dep_str in &metadata.dependencies {
                    // Parse dependency string (format: "name@version" or just "name")
                    let (dep_name, dep_version) = self.parse_dependency(dep_str);
                    
                    deps.push(dep_name.clone());
                    
                    // Add to queue if not visited
                    if !visited.contains(&dep_name) {
                        visited.insert(dep_name.clone());
                        queue.push_back((dep_name.clone(), depth + 1));
                        
                        // Add node
                        graph.nodes.insert(
                            dep_name.clone(),
                            PackageNode {
                                name: dep_name,
                                version: dep_version,
                                metadata: None,
                                state: ResolutionState::Unresolved,
                            },
                        );
                    }
                }
                
                graph.edges.insert(pkg_name, deps);
            }
        }
        
        // Check for circular dependencies
        let cycles = self.detect_cycles(&graph);
        if !cycles.is_empty() {
            return Err(MarketplaceError::Dependency(
                format!("Circular dependencies detected: {:?}", cycles)
            ));
        }
        
        // Topological sort to get installation order
        let sorted = self.topological_sort(&graph)?;
        
        // Convert to package metadata
        let mut result = Vec::new();
        for name in sorted {
            if name == package.name {
                continue; // Skip root package
            }
            
            if let Some(node) = graph.nodes.get(&name) {
                if let Some(ref metadata) = node.metadata {
                    result.push(metadata.clone());
                } else {
                    return Err(MarketplaceError::Dependency(
                        format!("Failed to resolve dependency: {}", name)
                    ));
                }
            }
        }
        
        Ok(result)
    }
    
    /// Resolve with full result details
    pub async fn resolve_detailed(
        &self,
        package: &PackageMetadata,
        _repository_manager: &RepositoryManager,
    ) -> Result<ResolutionResult, MarketplaceError> {
        let packages = self.resolve(package, _repository_manager).await?;
        
        Ok(ResolutionResult {
            packages,
            warnings: Vec::new(),
            circular_deps: Vec::new(),
        })
    }
    
    /// Check if all dependencies are satisfied
    pub async fn check_dependencies(
        &self,
        package: &PackageMetadata,
        repository_manager: &RepositoryManager,
    ) -> Result<bool, MarketplaceError> {
        for dep_str in &package.dependencies {
            let (dep_name, dep_version) = self.parse_dependency(dep_str);
            
            if self.find_package_in_repos(&dep_name, &dep_version, repository_manager).await.is_err() {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Get missing dependencies
    pub async fn get_missing_dependencies(
        &self,
        package: &PackageMetadata,
        repository_manager: &RepositoryManager,
    ) -> Result<Vec<DependencyConstraint>, MarketplaceError> {
        let mut missing = Vec::new();
        
        for dep_str in &package.dependencies {
            let (dep_name, dep_version) = self.parse_dependency(dep_str);
            
            if self.find_package_in_repos(&dep_name, &dep_version, repository_manager).await.is_err() {
                missing.push(DependencyConstraint {
                    name: dep_name,
                    version_constraint: dep_version,
                    optional: false,
                    features: Vec::new(),
                });
            }
        }
        
        Ok(missing)
    }
    
    /// Parse dependency string
    fn parse_dependency(&self, dep_str: &str) -> (String, String) {
        if let Some(at_pos) = dep_str.find('@') {
            let name = dep_str[..at_pos].to_string();
            let version = dep_str[at_pos + 1..].to_string();
            (name, version)
        } else {
            (dep_str.to_string(), "latest".to_string())
        }
    }
    
    /// Find package in repositories
    async fn find_package_in_repos(
        &self,
        name: &str,
        version_constraint: &str,
        repository_manager: &RepositoryManager,
    ) -> Result<PackageMetadata, MarketplaceError> {
        // Check cache first
        let cache_key = format!("{}@{}", name, version_constraint);
        if let Some(cached) = self.resolution_cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        // Search repositories
        for repo in repository_manager.enabled_repositories() {
            if let Ok(package) = repo.find_package(name, version_constraint).await {
                return Ok(package);
            }
        }
        
        Err(MarketplaceError::NotFound(
            format!("Dependency {}@{} not found in any repository", name, version_constraint)
        ))
    }
    
    /// Detect circular dependencies using DFS
    fn detect_cycles(&self, graph: &DependencyGraph) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();
        
        for node in graph.nodes.keys() {
            if !visited.contains(node) {
                self.dfs_cycle_check(
                    node,
                    graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }
        
        cycles
    }
    
    /// DFS helper for cycle detection
    fn dfs_cycle_check(
        &self,
        node: &str,
        graph: &DependencyGraph,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());
        
        if let Some(deps) = graph.edges.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    self.dfs_cycle_check(dep, graph, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(dep) {
                    // Found a cycle
                    if let Some(pos) = path.iter().position(|n| n == dep) {
                        let cycle = path[pos..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }
        
        path.pop();
        rec_stack.remove(node);
    }
    
    /// Topological sort for installation order
    /// Returns nodes in order: dependencies first, then dependents
    fn topological_sort(&self, graph: &DependencyGraph) -> Result<Vec<String>, MarketplaceError> {
        let mut in_degree = HashMap::new();
        
        // Initialize in-degrees to 0
        for node in graph.nodes.keys() {
            in_degree.insert(node.clone(), 0);
        }
        
        // Calculate in-degrees: count how many packages depend on each node
        // If A depends on B (edge A -> B), then B has an incoming edge from A
        // For installation order, we want to install B before A
        // So we count: for each edge A -> B, B's in-degree increases
        for (node, deps) in &graph.edges {
            for _dep in deps {
                // node depends on dep, so dep must be installed before node
                // This means dep has an outgoing edge to node in the dependency graph
                // But for topological sort, we need to reverse the edges
                // Actually, we want: if A depends on B, B comes before A
                // So we treat edges as B -> A (B must come before A)
                *in_degree.entry(node.clone()).or_insert(0) += 1;
            }
        }
        
        // Find all nodes with in-degree 0 (no dependencies)
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| name.clone())
            .collect();
        
        let mut result = Vec::new();
        
        while let Some(node) = queue.pop_front() {
            result.push(node.clone());
            
            // For all nodes that depend on this node (reverse edges)
            for (dependent, deps) in &graph.edges {
                if deps.contains(&node) {
                    if let Some(deg) = in_degree.get_mut(dependent) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }
        
        // Check if all nodes were processed
        if result.len() != graph.nodes.len() {
            return Err(MarketplaceError::Dependency(
                "Dependency graph has cycles, cannot determine installation order".to_string()
            ));
        }
        
        Ok(result)
    }
    
    /// Get dependency tree as string representation
    pub fn format_dependency_tree(
        &self,
        package: &PackageMetadata,
        _repository_manager: &RepositoryManager,
    ) -> String {
        let mut output = format!("{}@{}\n", package.name, package.version);
        
        for (i, dep) in package.dependencies.iter().enumerate() {
            let is_last = i == package.dependencies.len() - 1;
            let prefix = if is_last { "└── " } else { "├── " };
            output.push_str(&format!("{}{}\n", prefix, dep));
        }
        
        output
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Dependency lock file for reproducible installations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    /// Lock file version
    pub version: String,
    /// Resolved packages
    pub packages: Vec<LockedPackage>,
    /// Creation timestamp
    pub created_at: String,
}

/// Locked package entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    /// Package name
    pub name: String,
    /// Resolved version
    pub version: String,
    /// Source repository
    pub source: String,
    /// SHA256 checksum
    pub sha256: String,
    /// Dependencies (names only)
    pub dependencies: Vec<String>,
}

impl LockFile {
    /// Create a new lock file
    pub fn new(packages: Vec<LockedPackage>) -> Self {
        Self {
            version: "1.0".to_string(),
            packages,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    /// Load lock file from path
    pub fn load(path: &std::path::Path) -> Result<Self, MarketplaceError> {
        let content = std::fs::read_to_string(path)?;
        let lockfile: LockFile = serde_json::from_str(&content)
            .map_err(|e| MarketplaceError::Parse(format!("Invalid lock file: {}", e)))?;
        Ok(lockfile)
    }
    
    /// Save lock file to path
    pub fn save(&self, path: &std::path::Path) -> Result<(), MarketplaceError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Find a package in the lock file
    pub fn find_package(&self, name: &str) -> Option<&LockedPackage> {
        self.packages.iter().find(|p| p.name == name)
    }
    
    /// Get all package names
    pub fn package_names(&self) -> Vec<String> {
        self.packages.iter().map(|p| p.name.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_package(name: &str, version: &str, deps: Vec<&str>) -> PackageMetadata {
        PackageMetadata {
            name: name.to_string(),
            version: version.to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            license: "MIT".to_string(),
            package_type: super::super::PackageType::ApplicationModel,
            download_url: "https://example.com".to_string(),
            sha256: "abc".to_string(),
            signature: None,
            dependencies: deps.iter().map(|d| d.to_string()).collect(),
            min_tos_version: None,
            tags: vec![],
            size: 1024,
            created_at: "2024-01-01".to_string(),
        }
    }
    
    #[test]
    fn test_parse_dependency() {
        let resolver = DependencyResolver::new();
        
        let (name, version) = resolver.parse_dependency("package@1.0.0");
        assert_eq!(name, "package");
        assert_eq!(version, "1.0.0");
        
        let (name, version) = resolver.parse_dependency("package");
        assert_eq!(name, "package");
        assert_eq!(version, "latest");
    }
    
    #[test]
    fn test_dependency_graph() {
        let mut graph = DependencyGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            root: "root".to_string(),
        };
        
        graph.nodes.insert(
            "root".to_string(),
            PackageNode {
                name: "root".to_string(),
                version: "1.0.0".to_string(),
                metadata: None,
                state: ResolutionState::Unresolved,
            },
        );
        
        graph.edges.insert("root".to_string(), vec!["dep1".to_string(), "dep2".to_string()]);
        
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.get("root").unwrap().len(), 2);
    }
    
    #[test]
    fn test_detect_cycles() {
        let resolver = DependencyResolver::new();
        
        let mut graph = DependencyGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            root: "a".to_string(),
        };
        
        // Create cycle: a -> b -> c -> a
        for name in &["a", "b", "c"] {
            graph.nodes.insert(
                name.to_string(),
                PackageNode {
                    name: name.to_string(),
                    version: "1.0.0".to_string(),
                    metadata: None,
                    state: ResolutionState::Unresolved,
                },
            );
        }
        
        graph.edges.insert("a".to_string(), vec!["b".to_string()]);
        graph.edges.insert("b".to_string(), vec!["c".to_string()]);
        graph.edges.insert("c".to_string(), vec!["a".to_string()]);
        
        let cycles = resolver.detect_cycles(&graph);
        assert!(!cycles.is_empty());
    }
    
    #[test]
    fn test_topological_sort() {
        let resolver = DependencyResolver::new();
        
        let mut graph = DependencyGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            root: "app".to_string(),
        };
        
        // app -> lib1 -> lib2
        //     -> lib3
        for name in &["app", "lib1", "lib2", "lib3"] {
            graph.nodes.insert(
                name.to_string(),
                PackageNode {
                    name: name.to_string(),
                    version: "1.0.0".to_string(),
                    metadata: None,
                    state: ResolutionState::Unresolved,
                },
            );
        }
        
        graph.edges.insert("app".to_string(), vec!["lib1".to_string(), "lib3".to_string()]);
        graph.edges.insert("lib1".to_string(), vec!["lib2".to_string()]);
        graph.edges.insert("lib2".to_string(), vec![]);
        graph.edges.insert("lib3".to_string(), vec![]);
        
        let sorted = resolver.topological_sort(&graph).unwrap();
        
        // lib2 and lib3 should come before lib1 and app
        let lib2_pos = sorted.iter().position(|n| n == "lib2").unwrap();
        let lib1_pos = sorted.iter().position(|n| n == "lib1").unwrap();
        let app_pos = sorted.iter().position(|n| n == "app").unwrap();
        
        assert!(lib2_pos < lib1_pos);
        assert!(lib1_pos < app_pos);
    }
    
    #[test]
    fn test_lock_file() {
        let packages = vec![
            LockedPackage {
                name: "pkg1".to_string(),
                version: "1.0.0".to_string(),
                source: "repo1".to_string(),
                sha256: "abc".to_string(),
                dependencies: vec!["pkg2".to_string()],
            },
            LockedPackage {
                name: "pkg2".to_string(),
                version: "2.0.0".to_string(),
                source: "repo1".to_string(),
                sha256: "def".to_string(),
                dependencies: vec![],
            },
        ];
        
        let lockfile = LockFile::new(packages);
        
        assert_eq!(lockfile.packages.len(), 2);
        assert_eq!(lockfile.version, "1.0");
        
        let pkg1 = lockfile.find_package("pkg1").unwrap();
        assert_eq!(pkg1.version, "1.0.0");
        
        let names = lockfile.package_names();
        assert!(names.contains(&"pkg1".to_string()));
        assert!(names.contains(&"pkg2".to_string()));
    }
    
    #[test]
    fn test_format_dependency_tree() {
        let resolver = DependencyResolver::new();
        let package = create_test_package("my-app", "1.0.0", vec!["dep1", "dep2@2.0.0"]);
        
        let tree = resolver.format_dependency_tree(&package, &RepositoryManager::new(vec![]));
        
        assert!(tree.contains("my-app@1.0.0"));
        assert!(tree.contains("dep1"));
        assert!(tree.contains("dep2@2.0.0"));
    }
}
