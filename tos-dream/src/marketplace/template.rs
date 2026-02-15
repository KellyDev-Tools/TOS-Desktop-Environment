//! Sector Template System
//! 
//! Handles export and import of sector configurations as templates.
//! Templates are configuration-only packages that can be shared and reused.

use super::{MarketplaceError, ExportRequest, ExportResult, PackageType};
use sha2::Digest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Sector template structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Template metadata
    pub metadata: TemplateMetadata,
    /// Sector configuration
    pub sector_config: SectorConfig,
    /// Command hub configurations
    pub hub_configs: Vec<HubConfig>,
    /// Application configurations
    pub app_configs: Vec<AppConfig>,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Files to include (path -> content)
    pub files: HashMap<String, Vec<u8>>,
}

/// Template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Template name
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Description
    pub description: String,
    /// Author
    pub author: String,
    /// License (SPDX identifier)
    pub license: String,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: String,
    /// Minimum TOS version required
    pub min_tos_version: Option<String>,
    /// Template format version
    pub format_version: String,
}

/// Sector configuration snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorConfig {
    /// Sector name
    pub name: String,
    /// Sector type identifier
    pub sector_type: String,
    /// Default hub mode
    pub default_mode: String,
    /// Command favorites
    pub command_favorites: Vec<String>,
    /// Directory bookmarks
    pub directory_bookmarks: Vec<String>,
    /// Custom settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// Command hub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubConfig {
    /// Hub identifier
    pub id: String,
    /// Hub name
    pub name: String,
    /// Initial mode
    pub initial_mode: String,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Working directory
    pub working_directory: Option<String>,
    /// Command history (optional)
    pub command_history: Option<Vec<String>>,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application identifier
    pub id: String,
    /// Application class
    pub app_class: String,
    /// Application model to use
    pub app_model: Option<String>,
    /// Launch command or parameters
    pub launch_config: LaunchConfig,
    /// Window state
    pub window_state: WindowState,
}

/// Application launch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    /// Launch type
    pub launch_type: LaunchType,
    /// Command to execute (for CLI apps)
    pub command: Option<String>,
    /// Working directory
    pub working_directory: Option<String>,
    /// Environment variables
    pub environment: HashMap<String, String>,
}

/// Launch type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaunchType {
    /// Command line application
    #[serde(rename = "cli")]
    Cli,
    /// GUI application
    #[serde(rename = "gui")]
    Gui,
    /// Web application
    #[serde(rename = "web")]
    Web,
    /// Embedded widget
    #[serde(rename = "widget")]
    Widget,
}

/// Window state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    /// Window geometry
    pub geometry: Geometry,
    /// Whether window is maximized
    pub maximized: bool,
    /// Whether window is fullscreen
    pub fullscreen: bool,
    /// Whether window is always on top
    pub always_on_top: bool,
}

/// Geometry definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Geometry {
    /// X position
    pub x: i32,
    /// Y position
    pub y: i32,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

/// Template handler for export/import operations
pub struct TemplateHandler {
    /// Cache directory for templates
    cache_dir: PathBuf,
}

impl TemplateHandler {
    /// Create a new template handler
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let cache_dir = PathBuf::from(format!("{}/.cache/tos/templates", home));
        
        Self { cache_dir }
    }
    
    /// Create with custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }
    
    /// Export a sector as a template
    pub fn export_sector(&self, request: ExportRequest) -> Result<ExportResult, MarketplaceError> {
        // Create cache directory if needed
        std::fs::create_dir_all(&self.cache_dir)?;
        
        // Build template structure
        let template = self.build_template(&request)?;
        
        // Create output path
        let output_path = if request.output_path.is_absolute() {
            request.output_path.clone()
        } else {
            std::env::current_dir()?.join(&request.output_path)
        };
        
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Serialize template
        let template_json = serde_json::to_string_pretty(&template)?;
        
        // Create ZIP archive
        self.create_template_archive(&output_path, &template, &template_json)?;
        
        // Compute checksum
        let mut file = std::fs::File::open(&output_path)?;
        let mut hasher = sha2::Sha256::new();
        let mut buffer = [0u8; 8192];
        
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        let sha256 = hex::encode(hasher.finalize());
        let size = std::fs::metadata(&output_path)?.len();
        
        tracing::info!("Exported sector {} as template: {}", request.sector_id, output_path.display());
        
        Ok(ExportResult {
            template_path: output_path,
            size,
            sha256,
        })
    }
    
    /// Import a template from file
    pub fn import_template(&self, path: &Path) -> Result<Template, MarketplaceError> {
        if !path.exists() {
            return Err(MarketplaceError::NotFound(
                format!("Template file not found: {}", path.display())
            ));
        }
        
        // Check file extension
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        if extension != "tos-template" {
            return Err(MarketplaceError::Validation(
                format!("Invalid template file extension: .{}", extension)
            ));
        }
        
        // Extract and parse template
        let template = self.extract_template_archive(path)?;
        
        // Validate template
        self.validate_template(&template)?;
        
        tracing::info!("Imported template: {} v{}", template.metadata.name, template.metadata.version);
        
        Ok(template)
    }
    
    /// Apply a template to create a new sector
    pub fn apply_template(
        &self,
        template: &Template,
        sector_name: &str,
    ) -> Result<PathBuf, MarketplaceError> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let sectors_dir = PathBuf::from(format!("{}/.local/share/tos/sectors", home));
        let sector_dir = sectors_dir.join(sector_name);
        
        // Create sector directory
        std::fs::create_dir_all(&sector_dir)?;
        
        // Write sector configuration
        let config_path = sector_dir.join("sector.json");
        let config_file = std::fs::File::create(&config_path)?;
        serde_json::to_writer_pretty(config_file, &template.sector_config)?;
        
        // Write hub configurations
        for hub in &template.hub_configs {
            let hub_path = sector_dir.join(format!("hub_{}.json", hub.id));
            let hub_file = std::fs::File::create(&hub_path)?;
            serde_json::to_writer_pretty(hub_file, hub)?;
        }
        
        // Write application configurations
        for app in &template.app_configs {
            let app_path = sector_dir.join(format!("app_{}.json", app.id));
            let app_file = std::fs::File::create(&app_path)?;
            serde_json::to_writer_pretty(app_file, app)?;
        }
        
        // Write included files
        for (file_path, content) in &template.files {
            let full_path = sector_dir.join(file_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut file = std::fs::File::create(&full_path)?;
            file.write_all(content)?;
        }
        
        tracing::info!("Applied template {} to create sector: {}", template.metadata.name, sector_name);
        
        Ok(sector_dir)
    }
    
    /// Build template from export request
    fn build_template(&self, request: &ExportRequest) -> Result<Template, MarketplaceError> {
        let now = chrono::Utc::now().to_rfc3339();
        
        let metadata = TemplateMetadata {
            name: request.name.clone(),
            version: request.version.clone(),
            description: request.description.clone(),
            author: request.author.clone(),
            license: request.license.clone(),
            tags: request.tags.clone(),
            created_at: now,
            min_tos_version: Some("0.1.0".to_string()),
            format_version: "1.0".to_string(),
        };
        
        // Create default sector config (in real implementation, this would capture actual sector state)
        let sector_config = SectorConfig {
            name: request.name.clone(),
            sector_type: "default".to_string(),
            default_mode: "command".to_string(),
            command_favorites: vec![
                "ls".to_string(),
                "git status".to_string(),
                "cargo build".to_string(),
            ],
            directory_bookmarks: vec![
                "~".to_string(),
                "~/Projects".to_string(),
            ],
            settings: HashMap::new(),
        };
        
        // Create sample hub config
        let hub_configs = vec![
            HubConfig {
                id: "main".to_string(),
                name: "Main Hub".to_string(),
                initial_mode: "command".to_string(),
                environment: HashMap::new(),
                working_directory: Some("~".to_string()),
                command_history: if request.include_state {
                    Some(vec!["ls -la".to_string(), "git status".to_string()])
                } else {
                    None
                },
            },
        ];
        
        let template = Template {
            metadata,
            sector_config,
            hub_configs,
            app_configs: Vec::new(),
            environment: HashMap::new(),
            files: HashMap::new(),
        };
        
        Ok(template)
    }
    
    /// Create template ZIP archive
    fn create_template_archive(
        &self,
        output_path: &Path,
        template: &Template,
        template_json: &str,
    ) -> Result<(), MarketplaceError> {
        let file = std::fs::File::create(output_path)?;
        let mut zip = zip::ZipWriter::new(file);
        
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        
        // Write template.json
        zip.start_file("template.json", options)?;
        zip.write_all(template_json.as_bytes())?;
        
        // Write metadata as separate file for easy inspection
        let metadata_json = serde_json::to_string_pretty(&template.metadata)?;
        zip.start_file("metadata.json", options)?;
        zip.write_all(metadata_json.as_bytes())?;
        
        // Write included files
        for (path, content) in &template.files {
            zip.start_file(path, options)?;
            zip.write_all(content)?;
        }
        
        zip.finish()?;
        
        Ok(())
    }
    
    /// Extract template from ZIP archive
    fn extract_template_archive(&self, path: &Path) -> Result<Template, MarketplaceError> {
        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        // Read template.json
        let mut template_file = archive.by_name("template.json")
            .map_err(|_| MarketplaceError::Validation(
                "Template archive missing template.json".to_string()
            ))?;
        
        let mut template_json = String::new();
        template_file.read_to_string(&mut template_json)?;
        
        let template: Template = serde_json::from_str(&template_json)
            .map_err(|e| MarketplaceError::Parse(
                format!("Failed to parse template.json: {}", e)
            ))?;
        
        Ok(template)
    }
    
    /// Validate template structure
    fn validate_template(&self, template: &Template) -> Result<(), MarketplaceError> {
        // Check required fields
        if template.metadata.name.is_empty() {
            return Err(MarketplaceError::Validation(
                "Template name cannot be empty".to_string()
            ));
        }
        
        if template.metadata.version.is_empty() {
            return Err(MarketplaceError::Validation(
                "Template version cannot be empty".to_string()
            ));
        }
        
        // Validate version format (basic semver check)
        if !template.metadata.version.contains('.') {
            return Err(MarketplaceError::Validation(
                format!("Invalid version format: {}", template.metadata.version)
            ));
        }
        
        // Check format version compatibility
        if template.metadata.format_version != "1.0" {
            tracing::warn!(
                "Template format version {} may not be fully compatible",
                template.metadata.format_version
            );
        }
        
        Ok(())
    }
    
    /// List cached templates
    pub fn list_cached_templates(&self) -> Result<Vec<TemplateMetadata>, MarketplaceError> {
        let mut templates = Vec::new();
        
        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "tos-template") {
                        // Try to read metadata
                        if let Ok(template) = self.import_template(&path) {
                            templates.push(template.metadata);
                        }
                    }
                }
            }
        }
        
        Ok(templates)
    }
    
    /// Get cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

impl Default for TemplateHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Template exporter interface
pub trait TemplateExporter {
    /// Export current sector state to template
    fn export(&self, request: ExportRequest) -> Result<ExportResult, MarketplaceError>;
}

/// Template importer interface
pub trait TemplateImporter {
    /// Import template from file
    fn import(&self, path: &Path) -> Result<Template, MarketplaceError>;
    
    /// Apply template to create sector
    fn apply(&self, template: &Template, name: &str) -> Result<PathBuf, MarketplaceError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    
    fn create_test_request() -> ExportRequest {
        ExportRequest {
            sector_id: "test-sector".to_string(),
            name: "test-template".to_string(),
            version: "1.0.0".to_string(),
            output_path: PathBuf::from("/tmp/test-template.tos-template"),
            description: "Test template".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            include_state: false,
            tags: vec!["test".to_string()],
        }
    }
    
    #[test]
    fn test_template_metadata() {
        let metadata = TemplateMetadata {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            license: "MIT".to_string(),
            tags: vec!["test".to_string()],
            created_at: "2024-01-01".to_string(),
            min_tos_version: None,
            format_version: "1.0".to_string(),
        };
        
        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.format_version, "1.0");
    }
    
    #[test]
    fn test_sector_config() {
        let config = SectorConfig {
            name: "My Sector".to_string(),
            sector_type: "development".to_string(),
            default_mode: "command".to_string(),
            command_favorites: vec!["ls".to_string()],
            directory_bookmarks: vec!["~".to_string()],
            settings: HashMap::new(),
        };
        
        assert_eq!(config.name, "My Sector");
        assert_eq!(config.default_mode, "command");
    }
    
    #[test]
    fn test_launch_type_serialization() {
        let cli = LaunchType::Cli;
        let json = serde_json::to_string(&cli).unwrap();
        assert_eq!(json, "\"cli\"");
        
        let deserialized: LaunchType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, LaunchType::Cli);
    }
    
    #[test]
    fn test_template_handler_new() {
        let handler = TemplateHandler::new();
        assert!(!handler.cache_dir().as_os_str().is_empty());
    }
    
    #[test]
    fn test_export_and_import_template() {
        let temp_dir = tempfile::tempdir().unwrap();
        let template_path = temp_dir.path().join("test.tos-template");
        
        let handler = TemplateHandler::with_cache_dir(temp_dir.path().to_path_buf());
        
        let request = ExportRequest {
            sector_id: "test-sector".to_string(),
            name: "my-template".to_string(),
            version: "1.0.0".to_string(),
            output_path: template_path.clone(),
            description: "A test template".to_string(),
            author: "Test".to_string(),
            license: "MIT".to_string(),
            include_state: false,
            tags: vec!["test".to_string()],
        };
        
        // Export
        let result = handler.export_sector(request).unwrap();
        assert!(result.template_path.exists());
        assert!(!result.sha256.is_empty());
        
        // Import
        let template = handler.import_template(&result.template_path).unwrap();
        assert_eq!(template.metadata.name, "my-template");
        assert_eq!(template.metadata.version, "1.0.0");
    }
    
    #[test]
    fn test_validate_template() {
        let handler = TemplateHandler::new();
        
        let valid_template = Template {
            metadata: TemplateMetadata {
                name: "valid".to_string(),
                version: "1.0.0".to_string(),
                description: "Test".to_string(),
                author: "Test".to_string(),
                license: "MIT".to_string(),
                tags: vec![],
                created_at: "2024-01-01".to_string(),
                min_tos_version: None,
                format_version: "1.0".to_string(),
            },
            sector_config: SectorConfig {
                name: "Test".to_string(),
                sector_type: "default".to_string(),
                default_mode: "command".to_string(),
                command_favorites: vec![],
                directory_bookmarks: vec![],
                settings: HashMap::new(),
            },
            hub_configs: vec![],
            app_configs: vec![],
            environment: HashMap::new(),
            files: HashMap::new(),
        };
        
        assert!(handler.validate_template(&valid_template).is_ok());
        
        let invalid_template = Template {
            metadata: TemplateMetadata {
                name: "".to_string(),
                version: "1.0.0".to_string(),
                description: "Test".to_string(),
                author: "Test".to_string(),
                license: "MIT".to_string(),
                tags: vec![],
                created_at: "2024-01-01".to_string(),
                min_tos_version: None,
                format_version: "1.0".to_string(),
            },
            sector_config: valid_template.sector_config.clone(),
            hub_configs: vec![],
            app_configs: vec![],
            environment: HashMap::new(),
            files: HashMap::new(),
        };
        
        assert!(handler.validate_template(&invalid_template).is_err());
    }
}
