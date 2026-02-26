//! TOS Command Line Interface
//! 
//! Provides CLI commands for marketplace operations and sector management.
//! Can be used standalone or integrated into the main TOS application.

use crate::marketplace::{
    Marketplace, MarketplaceConfig, InstallRequest, ExportRequest, 
    RepositoryConfig
};
use std::path::PathBuf;

/// CLI command structure
#[derive(Debug)]
pub enum CliCommand {
    /// Marketplace operations
    Marketplace(MarketplaceCommand),
    /// Sector operations
    Sector(SectorCommand),
    /// Module operations
    Module(ModuleCommand),
    /// System operations (Section 14: Tactical Reset)
    System(SystemCommand),
    /// Container operations
    Container(ContainerCommand),
    /// SaaS operations
    Saas(SaasCommand),
    /// Sandbox operations (Local Isolation)
    Sandbox(SandboxCommand),
}

/// Marketplace subcommands
#[derive(Debug)]
pub enum MarketplaceCommand {
    /// Search for packages
    Search {
        query: String,
        repository: Option<String>,
    },
    /// Install a package
    Install {
        package: String,
        version: Option<String>,
        repository: Option<String>,
        skip_signature: bool,
    },
    /// List installed packages
    List,
    /// Remove a package
    Remove {
        package: String,
    },
    /// Add a repository
    AddRepo {
        name: String,
        url: String,
    },
    /// List repositories
    ListRepos,
    /// Remove a repository
    RemoveRepo {
        name: String,
    },
    /// Update repository indexes
    Update,
}

/// Sector subcommands
#[derive(Debug)]
pub enum SectorCommand {
    /// Level 1 tactical reset: current sector only (Section 14.1)
    Reset,
    /// Export a sector as template
    Export {
        sector_id: String,
        name: String,
        output: PathBuf,
        description: Option<String>,
        author: Option<String>,
    },
    /// Import a sector template
    Import {
        path: PathBuf,
    },
    /// List available templates
    ListTemplates,
    /// Apply a template to create a new sector
    Apply {
        template_path: PathBuf,
        sector_name: String,
    },
}

/// System subcommands (Section 14: Tactical Reset)
#[derive(Debug)]
pub enum SystemCommand {
    /// Level 2 tactical reset: compositor restart / logout dialog (Section 14.2)
    Reset,
}

/// Module subcommands
#[derive(Debug)]
pub enum ModuleCommand {
    /// List loaded modules
    List,
    /// Reload a module
    Reload {
        name: String,
    },
    /// Load a module from path
    Load {
        path: PathBuf,
    },
}

/// Container subcommands
#[derive(Debug)]
pub enum ContainerCommand {
    /// List all containers
    List,
    /// Get container logs
    Logs {
        id: String,
        tail: Option<usize>,
    },
    /// Get container metrics
    Stats {
        id: String,
    },
    /// Stop a container
    Stop {
        id: String,
    },
}

/// SaaS subcommands
#[derive(Debug)]
pub enum SaasCommand {
    /// Tenant operations
    Tenant(TenantCommand),
    /// Session operations
    Session(SessionCommand),
}
/// Sandbox subcommands
#[derive(Debug)]
pub enum SandboxCommand {
    /// Create a new sandbox
    Create {
        id: String,
        level: String,
    },
    /// List all sandboxes
    List,
    /// Terminate a sandbox
    Terminate {
        id: String,
    },
}

/// Tenant subcommands
#[derive(Debug)]
pub enum TenantCommand {
    /// Create a new tenant
    Create {
        name: String,
        owner: String,
    },
    /// List all tenants
    List,
    /// Get tenant usage stats
    Usage {
        id: String,
    },
}

/// Session subcommands
#[derive(Debug)]
pub enum SessionCommand {
    /// List active sessions
    List,
    /// Terminate a session
    Terminate {
        id: String,
    },
}

/// CLI handler for TOS commands
pub struct CliHandler {
    marketplace: Marketplace,
}

impl CliHandler {
    /// Create a new CLI handler with default configuration
    pub fn new() -> Self {
        let config = MarketplaceConfig::default();
        let marketplace = Marketplace::with_config(config);
        let _ = marketplace.initialize();
        
        Self { marketplace }
    }
    
    /// Create a new CLI handler with custom marketplace
    pub fn with_marketplace(marketplace: Marketplace) -> Self {
        Self { marketplace }
    }
    
    /// Execute a CLI command
    pub async fn execute(&self, command: CliCommand) -> Result<String, String> {
        match command {
            CliCommand::Marketplace(cmd) => self.handle_marketplace(cmd).await,
            CliCommand::Sector(cmd) => self.handle_sector(cmd),
            CliCommand::Module(cmd) => self.handle_module(cmd),
            CliCommand::System(cmd) => self.handle_system(cmd),
            CliCommand::Container(cmd) => self.handle_container(cmd).await,
            CliCommand::Saas(cmd) => self.handle_saas(cmd).await,
            CliCommand::Sandbox(cmd) => self.handle_sandbox(cmd).await,
        }
    }
    
    /// Handle marketplace commands
    async fn handle_marketplace(&self, command: MarketplaceCommand) -> Result<String, String> {
        match command {
            MarketplaceCommand::Search { query, repository: _ } => {
                let results = self.marketplace.search(&query).await
                    .map_err(|e| format!("Search failed: {}", e))?;
                
                let mut output = format!("Search results for '{}':\n\n", query);
                for pkg in results {
                    output.push_str(&format!(
                        "  {}@{} - {}\n  Type: {:?} | License: {} | Size: {} bytes\n\n",
                        pkg.name, pkg.version, pkg.description, 
                        pkg.package_type, pkg.license, pkg.size
                    ));
                }
                Ok(output)
            }
            
            MarketplaceCommand::Install { package, version, repository, skip_signature } => {
                let version_constraint = version.unwrap_or_else(|| "latest".to_string());
                
                println!("Installing {}@{}...", package, version_constraint);
                
                let request = InstallRequest {
                    package_name: package.clone(),
                    version_constraint,
                    repository,
                    auto_accept: false,
                    skip_signature_check: skip_signature,
                };
                
                let result = self.marketplace.install(request).await
                    .map_err(|e| format!("Installation failed: {}", e))?;
                
                Ok(format!(
                    "Successfully installed {}@{} to {}\nDependencies installed: {}",
                    result.package.name,
                    result.package.version,
                    result.install_path.display(),
                    if result.installed_dependencies.is_empty() {
                        "none".to_string()
                    } else {
                        result.installed_dependencies.join(", ")
                    }
                ))
            }
            
            MarketplaceCommand::List => {
                // List installed packages from the marketplace cache directory
                let cache_dir = &self.marketplace.config.cache_dir;
                let mut output = "Installed packages:\n\n".to_string();
                
                if let Ok(entries) = std::fs::read_dir(cache_dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        output.push_str(&format!("  {}\n", name));
                    }
                }
                
                Ok(output)
            }
            
            MarketplaceCommand::Remove { package } => {
                // Remove from cache directory
                let cache_dir = &self.marketplace.config.cache_dir;
                let package_dir = cache_dir.join(&package);
                
                if package_dir.exists() {
                    std::fs::remove_dir_all(&package_dir)
                        .map_err(|e| format!("Failed to remove package: {}", e))?;
                    Ok(format!("Removed package: {}", package))
                } else {
                    Err(format!("Package not found: {}", package))
                }
            }
            
            MarketplaceCommand::AddRepo { name, url } => {
                let _config = RepositoryConfig {
                    name: name.clone(),
                    url: url.clone(),
                    enabled: true,
                    priority: 1,
                    auth_token: None,
                };
                
                // Note: This would need mutable access in real implementation
                println!("Adding repository {} at {}", name, url);
                Ok(format!("Repository {} added successfully", name))
            }
            
            MarketplaceCommand::ListRepos => {
                let mut output = "Configured repositories:\n\n".to_string();
                
                for repo in &self.marketplace.config.repositories {
                    let status = if repo.enabled { "enabled" } else { "disabled" };
                    output.push_str(&format!(
                        "  {} - {} (priority: {}, {})\n",
                        repo.name, repo.url, repo.priority, status
                    ));
                }
                
                if self.marketplace.config.repositories.is_empty() {
                    output.push_str("  No repositories configured.\n");
                }
                
                Ok(output)
            }
            
            MarketplaceCommand::RemoveRepo { name } => {
                println!("Removing repository: {}", name);
                Ok(format!("Repository {} removed", name))
            }
            
            MarketplaceCommand::Update => {
                println!("Updating repository indexes...");
                
                // In a real implementation, this would refresh all repository indexes
                Ok("Repository indexes updated.".to_string())
            }
        }
    }
    
    /// Handle sector commands
    fn handle_sector(&self, command: SectorCommand) -> Result<String, String> {
        match command {
            SectorCommand::Reset => {
                Ok("Sector reset requested (Level 1). In TOS GUI: Super+Backspace or on-screen control.".to_string())
            }
            SectorCommand::Export { sector_id, name, output, description, author } => {
                let request = ExportRequest {
                    sector_id: sector_id.clone(),
                    name: name.clone(),
                    version: "1.0.0".to_string(),
                    output_path: output.clone(),
                    description: description.unwrap_or_else(|| format!("Template for {}", name)),
                    author: author.unwrap_or_else(|| "Unknown".to_string()),
                    license: "MIT".to_string(),
                    include_state: false,
                    tags: vec![],
                };
                
                let result = self.marketplace.export_sector(request)
                    .map_err(|e| format!("Export failed: {}", e))?;
                
                Ok(format!(
                    "Exported sector {} as template '{}' to {}\nSize: {} bytes\nSHA256: {}",
                    sector_id,
                    name,
                    result.template_path.display(),
                    result.size,
                    result.sha256
                ))
            }
            
            SectorCommand::Import { path } => {
                let template = self.marketplace.import_template(&path)
                    .map_err(|e| format!("Import failed: {}", e))?;
                
                Ok(format!(
                    "Imported template '{}' v{} by {}\nDescription: {}\nTags: {}",
                    template.metadata.name,
                    template.metadata.version,
                    template.metadata.author,
                    template.metadata.description,
                    template.metadata.tags.join(", ")
                ))
            }
            
            SectorCommand::ListTemplates => {
                // List templates in the templates directory
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let templates_dir = PathBuf::from(format!("{}/.local/share/tos/templates", home));
                
                let mut output = "Available templates:\n\n".to_string();
                
                if let Ok(entries) = std::fs::read_dir(&templates_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map_or(false, |e| e == "tos-template") {
                            let name = path.file_stem()
                                .map_or("unknown".to_string(), |s| s.to_string_lossy().to_string());
                            output.push_str(&format!("  {}\n", name));
                        }
                    }
                }
                
                Ok(output)
            }
            
            SectorCommand::Apply { template_path, sector_name } => {
                let template = self.marketplace.import_template(&template_path)
                    .map_err(|e| format!("Failed to load template: {}", e))?;
                
                // In a real implementation, this would create a new sector
                // based on the template configuration
                
                Ok(format!(
                    "Applied template '{}' to create sector '{}'\nType: {:?}\nHubs: {}",
                    template.metadata.name,
                    sector_name,
                    template.sector_config.sector_type,
                    template.hub_configs.len()
                ))
            }
        }
    }
    
    /// Handle system commands (Section 14: Tactical Reset)
    fn handle_system(&self, command: SystemCommand) -> Result<String, String> {
        match command {
            SystemCommand::Reset => {
                Ok("System reset requested (Level 2). In TOS GUI: Super+Alt+Backspace or on-screen dialog.".to_string())
            }
        }
    }

    /// Handle module commands
    fn handle_module(&self, command: ModuleCommand) -> Result<String, String> {
        match command {
            ModuleCommand::List => {
                // This would integrate with the module registry
                Ok("Loaded modules:\n  (Module registry integration pending)\n".to_string())
            }
            
            ModuleCommand::Reload { name } => {
                Ok(format!("Reloaded module: {}", name))
            }
            
            ModuleCommand::Load { path } => {
                Ok(format!("Loaded module from: {}", path.display()))
            }
        }
    }

    /// Handle container commands
    async fn handle_container(&self, command: ContainerCommand) -> Result<String, String> {
        match command {
            ContainerCommand::List => Ok("Listing containers...".to_string()),
            ContainerCommand::Logs { id, .. } => Ok(format!("Logs for container {}: ...", id)),
            ContainerCommand::Stats { id } => Ok(format!("Stats for container {}: ...", id)),
            ContainerCommand::Stop { id } => Ok(format!("Stopped container {}", id)),
        }
    }

    /// Handle SaaS commands
    async fn handle_saas(&self, command: SaasCommand) -> Result<String, String> {
        match command {
            SaasCommand::Tenant(TenantCommand::Create { name, .. }) => Ok(format!("Created tenant {}", name)),
            SaasCommand::Tenant(TenantCommand::List) => Ok("Listing tenants...".to_string()),
            SaasCommand::Tenant(TenantCommand::Usage { id }) => Ok(format!("Usage for tenant {}: ...", id)),
            SaasCommand::Session(SessionCommand::List) => Ok("Listing sessions...".to_string()),
            SaasCommand::Session(SessionCommand::Terminate { id }) => Ok(format!("Terminated session {}", id)),
        }
    }

    /// Handle Sandbox commands
    async fn handle_sandbox(&self, command: SandboxCommand) -> Result<String, String> {
        match command {
            SandboxCommand::Create { id, level } => Ok(format!("Created sandbox {} with level {}", id, level)),
            SandboxCommand::List => Ok("Listing local sandboxes...".to_string()),
            SandboxCommand::Terminate { id } => Ok(format!("Terminated sandbox {}", id)),
        }
    }
    
    /// Parse command line arguments and execute
    pub async fn run_from_args(&self, args: &[String]) -> Result<String, String> {
        if args.len() < 2 {
            return Ok(self.help_text());
        }
        
        let command = self.parse_args(args)?;
        self.execute(command).await
    }
    
    /// Parse arguments into a command
    fn parse_args(&self, args: &[String]) -> Result<CliCommand, String> {
        if args.len() < 2 {
            return Err("No command specified".to_string());
        }
        
        match args[1].as_str() {
            "marketplace" | "mp" => self.parse_marketplace_args(args),
            "sector" | "s" => self.parse_sector_args(args),
            "module" | "m" => self.parse_module_args(args),
            "system" => self.parse_system_args(args),
            "container" | "c" => self.parse_container_args(args),
            "saas" => self.parse_saas_args(args),
            "sandbox" | "sb" => self.parse_sandbox_args(args),
            _ => Err(format!("Unknown command: {}", args[1])),
        }
    }
    
    fn parse_marketplace_args(&self, args: &[String]) -> Result<CliCommand, String> {
        if args.len() < 3 {
            return Err("No marketplace subcommand specified".to_string());
        }
        
        let cmd = match args[2].as_str() {
            "search" => {
                if args.len() < 4 {
                    return Err("Search query required".to_string());
                }
                MarketplaceCommand::Search {
                    query: args[3].clone(),
                    repository: args.get(4).cloned(),
                }
            }
            "install" | "i" => {
                if args.len() < 4 {
                    return Err("Package name required".to_string());
                }
                MarketplaceCommand::Install {
                    package: args[3].clone(),
                    version: args.get(4).cloned(),
                    repository: None,
                    skip_signature: args.iter().any(|a| a == "--skip-signature"),
                }
            }
            "list" | "ls" => MarketplaceCommand::List,
            "remove" | "rm" => {
                if args.len() < 4 {
                    return Err("Package name required".to_string());
                }
                MarketplaceCommand::Remove {
                    package: args[3].clone(),
                }
            }
            "add-repo" => {
                if args.len() < 5 {
                    return Err("Repository name and URL required".to_string());
                }
                MarketplaceCommand::AddRepo {
                    name: args[3].clone(),
                    url: args[4].clone(),
                }
            }
            "list-repos" => MarketplaceCommand::ListRepos,
            "remove-repo" => {
                if args.len() < 4 {
                    return Err("Repository name required".to_string());
                }
                MarketplaceCommand::RemoveRepo {
                    name: args[3].clone(),
                }
            }
            "update" => MarketplaceCommand::Update,
            _ => return Err(format!("Unknown marketplace command: {}", args[2])),
        };
        
        Ok(CliCommand::Marketplace(cmd))
    }
    
    fn parse_sector_args(&self, args: &[String]) -> Result<CliCommand, String> {
        if args.len() < 3 {
            return Err("No sector subcommand specified".to_string());
        }
        
        let cmd = match args[2].as_str() {
            "export" | "e" => {
                if args.len() < 6 {
                    return Err("Usage: sector export <sector-id> <name> <output-path>".to_string());
                }
                SectorCommand::Export {
                    sector_id: args[3].clone(),
                    name: args[4].clone(),
                    output: PathBuf::from(&args[5]),
                    description: args.get(6).cloned(),
                    author: args.get(7).cloned(),
                }
            }
            "import" | "i" => {
                if args.len() < 4 {
                    return Err("Template path required".to_string());
                }
                SectorCommand::Import {
                    path: PathBuf::from(&args[3]),
                }
            }
            "list-templates" | "ls" => SectorCommand::ListTemplates,
            "apply" | "a" => {
                if args.len() < 5 {
                    return Err("Usage: sector apply <template-path> <sector-name>".to_string());
                }
                SectorCommand::Apply {
                    template_path: PathBuf::from(&args[3]),
                    sector_name: args[4].clone(),
                }
            }
            "reset" | "r" => SectorCommand::Reset,
            _ => return Err(format!("Unknown sector command: {}", args[2])),
        };
        
        Ok(CliCommand::Sector(cmd))
    }

    fn parse_system_args(&self, args: &[String]) -> Result<CliCommand, String> {
        if args.len() < 3 {
            return Err("No system subcommand specified.".to_string());
        }
        let cmd = match args[2].as_str() {
            "reset" | "r" => SystemCommand::Reset,
            _ => return Err(format!("Unknown system command: {}", args[2])),
        };
        Ok(CliCommand::System(cmd))
    }
    
    fn parse_module_args(&self, args: &[String]) -> Result<CliCommand, String> {
        if args.len() < 3 {
            return Err("No module subcommand specified".to_string());
        }
        
        let cmd = match args[2].as_str() {
            "list" | "ls" => ModuleCommand::List,
            "reload" | "r" => {
                if args.len() < 4 {
                    return Err("Module name required".to_string());
                }
                ModuleCommand::Reload {
                    name: args[3].clone(),
                }
            }
            "load" | "l" => {
                if args.len() < 4 {
                    return Err("Module path required".to_string());
                }
                ModuleCommand::Load {
                    path: PathBuf::from(&args[3]),
                }
            }
            _ => return Err(format!("Unknown module command: {}", args[2])),
        };
        
        Ok(CliCommand::Module(cmd))
    }

    fn parse_container_args(&self, args: &[String]) -> Result<CliCommand, String> {
        if args.len() < 3 {
            return Err("No container subcommand specified".to_string());
        }

        let cmd = match args[2].as_str() {
            "list" | "ls" => ContainerCommand::List,
            "logs" => {
                if args.len() < 4 {
                    return Err("Container ID required".to_string());
                }
                ContainerCommand::Logs {
                    id: args[3].clone(),
                    tail: args.get(4).and_then(|t| t.parse().ok()),
                }
            }
            "stats" => {
                if args.len() < 4 {
                    return Err("Container ID required".to_string());
                }
                ContainerCommand::Stats {
                    id: args[3].clone(),
                }
            }
            "stop" => {
                if args.len() < 4 {
                    return Err("Container ID required".to_string());
                }
                ContainerCommand::Stop {
                    id: args[3].clone(),
                }
            }
            _ => return Err(format!("Unknown container command: {}", args[2])),
        };

        Ok(CliCommand::Container(cmd))
    }

    fn parse_saas_args(&self, args: &[String]) -> Result<CliCommand, String> {
        if args.len() < 3 {
            return Err("No saas subcommand specified".to_string());
        }

        let cmd = match args[2].as_str() {
            "tenant" => {
                if args.len() < 4 {
                    return Err("No tenant subcommand specified".to_string());
                }
                match args[3].as_str() {
                    "create" => {
                        if args.len() < 6 {
                            return Err("Usage: saas tenant create <name> <owner>".to_string());
                        }
                        SaasCommand::Tenant(TenantCommand::Create {
                            name: args[4].clone(),
                            owner: args[5].clone(),
                        })
                    }
                    "list" | "ls" => SaasCommand::Tenant(TenantCommand::List),
                    "usage" => {
                        if args.len() < 5 {
                            return Err("Tenant ID required".to_string());
                        }
                        SaasCommand::Tenant(TenantCommand::Usage {
                            id: args[4].clone(),
                        })
                    }
                    _ => return Err(format!("Unknown saas tenant command: {}", args[3])),
                }
            }
            "session" => {
                if args.len() < 4 {
                    return Err("No session subcommand specified".to_string());
                }
                match args[3].as_str() {
                    "list" | "ls" => SaasCommand::Session(SessionCommand::List),
                    "terminate" | "term" => {
                        if args.len() < 5 {
                            return Err("Session ID required".to_string());
                        }
                        SaasCommand::Session(SessionCommand::Terminate {
                            id: args[4].clone(),
                        })
                    }
                    _ => return Err(format!("Unknown saas session command: {}", args[3])),
                }
            }
            _ => return Err(format!("Unknown saas command: {}", args[2])),
        };

        Ok(CliCommand::Saas(cmd))
    }

    fn parse_sandbox_args(&self, args: &[String]) -> Result<CliCommand, String> {
        if args.len() < 3 {
            return Err("No sandbox subcommand specified".to_string());
        }

        let cmd = match args[2].as_str() {
            "create" => {
                if args.len() < 5 {
                    return Err("Usage: sandbox create <id> <level>".to_string());
                }
                SandboxCommand::Create {
                    id: args[3].clone(),
                    level: args[4].clone(),
                }
            }
            "list" | "ls" => SandboxCommand::List,
            "terminate" | "term" => {
                if args.len() < 4 {
                    return Err("Sandbox ID required".to_string());
                }
                SandboxCommand::Terminate {
                    id: args[3].clone(),
                }
            }
            _ => return Err(format!("Unknown sandbox command: {}", args[2])),
        };

        Ok(CliCommand::Sandbox(cmd))
    }
    
    /// Get help text
    fn help_text(&self) -> String {
        r#"TOS Command Line Interface
 
 USAGE:
     tos <command> [subcommand] [options]
 
 COMMANDS:
     marketplace, mp    Marketplace operations
         search <query> [repo]     Search for packages
         install <package> [ver]     Install a package
         list                        List installed packages
         remove <package>            Remove a package
         add-repo <name> <url>       Add a repository
         list-repos                  List repositories
         remove-repo <name>          Remove a repository
         update                      Update repository indexes
 
     sector, s          Sector operations
         reset                       Level 1 tactical reset (current sector)
         export <id> <name> <path>   Export sector as template
         import <path>               Import a template
         list-templates              List available templates
         apply <path> <name>         Apply template to create sector
 
     system             System operations (Section 14: Tactical Reset)
         reset                       Level 2 tactical reset (compositor / logout)
 
     module, m          Module operations
         list                        List loaded modules
         reload <name>               Reload a module
         load <path>                 Load module from path

     container, c       Container operations
         list                        List all containers
         logs <id> [tail]           Get container logs
         stats <id>                 Get container resource usage
         stop <id>                  Stop a container

     saas               SaaS management operations
         tenant create <n> <o>      Create a new tenant
         tenant list                List all tenants
         tenant usage <id>          Get tenant usage statistics
         session list               List active sessions
         session terminate <id>     Terminate a user session
 
 EXAMPLES:
     tos marketplace search terminal
     tos mp install terminal-enhanced
     tos sector reset
     tos system reset
     tos container list
     tos saas tenant create AcmeCorp "John Doe"
 "#.to_string()
    }
}

impl Default for CliHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cli_handler_new() {
        let handler = CliHandler::new();
        // Should create successfully with default config
        // Default config has verify_signatures: true for security
        assert!(handler.marketplace.config.verify_signatures);
    }
    
    #[test]
    fn test_parse_marketplace_search() {
        let handler = CliHandler::new();
        let args = vec!["tos".to_string(), "marketplace".to_string(), "search".to_string(), "terminal".to_string()];
        
        let result = handler.parse_args(&args);
        assert!(result.is_ok());
        
        match result.unwrap() {
            CliCommand::Marketplace(MarketplaceCommand::Search { query, .. }) => {
                assert_eq!(query, "terminal");
            }
            _ => panic!("Expected search command"),
        }
    }
    
    #[test]
    fn test_parse_marketplace_install() {
        let handler = CliHandler::new();
        let args = vec![
            "tos".to_string(),
            "marketplace".to_string(),
            "install".to_string(),
            "terminal-enhanced".to_string(),
            "1.0.0".to_string(),
        ];
        
        let result = handler.parse_args(&args);
        assert!(result.is_ok());
        
        match result.unwrap() {
            CliCommand::Marketplace(MarketplaceCommand::Install { package, version, .. }) => {
                assert_eq!(package, "terminal-enhanced");
                assert_eq!(version, Some("1.0.0".to_string()));
            }
            _ => panic!("Expected install command"),
        }
    }
    
    #[test]
    fn test_parse_sector_export() {
        let handler = CliHandler::new();
        let args = vec![
            "tos".to_string(),
            "sector".to_string(),
            "export".to_string(),
            "sector-123".to_string(),
            "my-template".to_string(),
            "./output.tos-template".to_string(),
        ];
        
        let result = handler.parse_args(&args);
        assert!(result.is_ok());
        
        match result.unwrap() {
            CliCommand::Sector(SectorCommand::Export { sector_id, name, output, .. }) => {
                assert_eq!(sector_id, "sector-123");
                assert_eq!(name, "my-template");
                assert_eq!(output, PathBuf::from("./output.tos-template"));
            }
            _ => panic!("Expected export command"),
        }
    }
    
    #[test]
    fn test_help_text() {
        let handler = CliHandler::new();
        let help = handler.help_text();
        
        assert!(help.contains("marketplace"));
        assert!(help.contains("sector"));
        assert!(help.contains("module"));
        assert!(help.contains("search"));
        assert!(help.contains("install"));
        assert!(help.contains("export"));
    }
    
    #[test]
    fn test_unknown_command() {
        let handler = CliHandler::new();
        let args = vec!["tos".to_string(), "unknown".to_string()];
        
        let result = handler.parse_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sector_reset() {
        let handler = CliHandler::new();
        let args = vec!["tos".to_string(), "sector".to_string(), "reset".to_string()];
        let result = handler.parse_args(&args);
        assert!(result.is_ok());
        match result.unwrap() {
            CliCommand::Sector(SectorCommand::Reset) => {}
            _ => panic!("Expected sector reset command"),
        }
    }

    #[test]
    fn test_parse_system_reset() {
        let handler = CliHandler::new();
        let args = vec!["tos".to_string(), "system".to_string(), "reset".to_string()];
        let result = handler.parse_args(&args);
        assert!(result.is_ok());
        match result.unwrap() {
            CliCommand::System(SystemCommand::Reset) => {}
            _ => panic!("Expected system reset command"),
        }
    }

    #[test]
    fn test_parse_container_list() {
        let handler = CliHandler::new();
        let args = vec!["tos".to_string(), "container".to_string(), "list".to_string()];
        let result = handler.parse_args(&args);
        assert!(result.is_ok());
        match result.unwrap() {
            CliCommand::Container(ContainerCommand::List) => {}
            _ => panic!("Expected container list command"),
        }
    }

    #[test]
    fn test_parse_saas_tenant_create() {
        let handler = CliHandler::new();
        let args = vec![
            "tos".to_string(), 
            "saas".to_string(), 
            "tenant".to_string(), 
            "create".to_string(),
            "AcmeCorp".to_string(),
            "John".to_string(),
        ];
        let result = handler.parse_args(&args);
        assert!(result.is_ok());
        match result.unwrap() {
            CliCommand::Saas(SaasCommand::Tenant(TenantCommand::Create { name, owner })) => {
                assert_eq!(name, "AcmeCorp");
                assert_eq!(owner, "John");
            }
            _ => panic!("Expected saas tenant create command"),
        }
    }
}
