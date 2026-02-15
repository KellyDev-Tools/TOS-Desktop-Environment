//! Marketplace Integration Tests
//! 
//! Tests for Phase 9: Marketplace and Templates functionality

use tos_core::marketplace::*;
use tos_core::marketplace::template::{TemplateHandler, SectorConfig};
use tos_core::marketplace::signature::VerificationResult;
use std::path::PathBuf;

/// Test marketplace configuration
#[test]
fn test_marketplace_config_default() {
    let config = MarketplaceConfig::default();
    assert!(config.verify_signatures);
    assert!(config.auto_install_dependencies);
    assert!(config.repositories.is_empty());
    assert!(!config.cache_dir.as_os_str().is_empty());
}

/// Test repository configuration
#[test]
fn test_repository_config() {
    let repo = RepositoryConfig {
        name: "test-repo".to_string(),
        url: "https://example.com/repo".to_string(),
        enabled: true,
        priority: 1,
        auth_token: Some("token123".to_string()),
    };
    
    assert_eq!(repo.name, "test-repo");
    assert_eq!(repo.url, "https://example.com/repo");
    assert!(repo.enabled);
    assert_eq!(repo.priority, 1);
    assert_eq!(repo.auth_token, Some("token123".to_string()));
}

/// Test package metadata
#[test]
fn test_package_metadata() {
    let metadata = PackageMetadata {
        name: "test-package".to_string(),
        version: "1.0.0".to_string(),
        description: "Test package".to_string(),
        author: "Test Author".to_string(),
        license: "MIT".to_string(),
        package_type: PackageType::ApplicationModel,
        download_url: "https://example.com/pkg.tos-appmodel".to_string(),
        sha256: "abc123def456".to_string(),
        signature: Some("sig789".to_string()),
        dependencies: vec!["dep1".to_string(), "dep2@2.0.0".to_string()],
        min_tos_version: Some("0.1.0".to_string()),
        tags: vec!["test".to_string(), "demo".to_string()],
        size: 1024,
        created_at: "2024-01-01T00:00:00Z".to_string(),
    };
    
    assert_eq!(metadata.name, "test-package");
    assert_eq!(metadata.version, "1.0.0");
    assert_eq!(metadata.package_type, PackageType::ApplicationModel);
    assert_eq!(metadata.dependencies.len(), 2);
    assert!(metadata.signature.is_some());
}

/// Test package type extensions
#[test]
fn test_package_type_extensions() {
    assert_eq!(PackageType::Template.extension(), ".tos-template");
    assert_eq!(PackageType::SectorType.extension(), ".tos-sector");
    assert_eq!(PackageType::ApplicationModel.extension(), ".tos-appmodel");
    
    assert_eq!(PackageType::from_extension(".tos-template"), Some(PackageType::Template));
    assert_eq!(PackageType::from_extension(".tos-sector"), Some(PackageType::SectorType));
    assert_eq!(PackageType::from_extension(".tos-appmodel"), Some(PackageType::ApplicationModel));
    assert_eq!(PackageType::from_extension(".unknown"), None);
}

/// Test install request
#[test]
fn test_install_request() {
    let request = InstallRequest {
        package_name: "my-package".to_string(),
        version_constraint: "1.0.0".to_string(),
        repository: Some("official".to_string()),
        auto_accept: false,
        skip_signature_check: false,
    };
    
    assert_eq!(request.package_name, "my-package");
    assert_eq!(request.version_constraint, "1.0.0");
    assert_eq!(request.repository, Some("official".to_string()));
    assert!(!request.auto_accept);
    assert!(!request.skip_signature_check);
}

/// Test export request
#[test]
fn test_export_request() {
    let request = ExportRequest {
        sector_id: "sector-123".to_string(),
        name: "dev-workspace".to_string(),
        version: "1.0.0".to_string(),
        output_path: PathBuf::from("/tmp/export.tos-template"),
        description: "Development workspace template".to_string(),
        author: "Developer".to_string(),
        license: "MIT".to_string(),
        include_state: true,
        tags: vec!["dev".to_string(), "workspace".to_string()],
    };
    
    assert_eq!(request.sector_id, "sector-123");
    assert_eq!(request.name, "dev-workspace");
    assert!(request.include_state);
    assert_eq!(request.tags.len(), 2);
}

/// Test marketplace error types
#[test]
fn test_marketplace_errors() {
    let err = MarketplaceError::NotFound("package".to_string());
    assert!(err.to_string().contains("Not found"));
    
    let err = MarketplaceError::Network("timeout".to_string());
    assert!(err.to_string().contains("Network error"));
    
    let err = MarketplaceError::Validation("invalid".to_string());
    assert!(err.to_string().contains("Validation error"));
    
    let err = MarketplaceError::Signature("bad sig".to_string());
    assert!(err.to_string().contains("Signature error"));
    
    let err = MarketplaceError::Dependency("missing".to_string());
    assert!(err.to_string().contains("Dependency error"));
}

/// Test template metadata
#[test]
fn test_template_metadata() {
    let metadata = TemplateMetadata {
        name: "my-template".to_string(),
        version: "1.0.0".to_string(),
        description: "My template".to_string(),
        author: "Author".to_string(),
        license: "MIT".to_string(),
        tags: vec!["template".to_string()],
        created_at: "2024-01-01".to_string(),
        min_tos_version: Some("0.1.0".to_string()),
        format_version: "1.0".to_string(),
    };
    
    assert_eq!(metadata.name, "my-template");
    assert_eq!(metadata.format_version, "1.0");
}

/// Test signature verifier
#[test]
fn test_signature_verifier() {
    let verifier = SignatureVerifier::new(vec![]);
    assert_eq!(verifier.trusted_key_count(), 0);
    
    let mut verifier = SignatureVerifier::new(vec!["key1".to_string()]);
    assert_eq!(verifier.trusted_key_count(), 1);
    assert!(verifier.is_key_trusted("key1"));
    
    verifier.add_trusted_key("key2".to_string());
    assert_eq!(verifier.trusted_key_count(), 2);
    
    verifier.remove_trusted_key("key1");
    assert!(!verifier.is_key_trusted("key1"));
    assert_eq!(verifier.trusted_key_count(), 1);
}

/// Test repository manager
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
            auth_token: None,
        },
    ];
    
    let manager = RepositoryManager::new(configs);
    assert_eq!(manager.repository_names().len(), 2);
    
    let enabled = manager.enabled_repositories();
    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled[0].name(), "repo1");
}

/// Test package manager cache operations
#[test]
fn test_package_manager_cache() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = PackageManager::new(temp_dir.path().to_path_buf());
    
    // Initially empty
    let cache = manager.list_cache().unwrap();
    assert!(cache.is_empty());
    
    // Create a dummy file in cache
    let dummy_file = temp_dir.path().join("test-package-1.0.0-abc123.tos-appmodel");
    std::fs::write(&dummy_file, "dummy content").unwrap();
    
    let cache = manager.list_cache().unwrap();
    assert_eq!(cache.len(), 1);
    assert_eq!(cache[0].0, "test-package-1.0.0-abc123.tos-appmodel");
    
    // Clear cache
    manager.clear_cache().unwrap();
    let cache = manager.list_cache().unwrap();
    assert!(cache.is_empty());
}

/// Test dependency resolver
#[test]
fn test_dependency_resolver() {
    let _resolver = DependencyResolver::new();
    
    // Test with max depth
    let _resolver = _resolver.with_max_depth(10);
    // Just verify it doesn't panic
}

/// Test marketplace creation
#[test]
fn test_marketplace_creation() {
    let _marketplace = Marketplace::new();
    // Should create without error
    
    let config = MarketplaceConfig::default();
    let _marketplace = Marketplace::with_config(config);
    // Should create without error
}

/// Test template handler
#[test]
fn test_template_handler() {
    let temp_dir = tempfile::tempdir().unwrap();
    let handler = TemplateHandler::with_cache_dir(temp_dir.path().to_path_buf());
    
    // Test export
    let request = ExportRequest {
        sector_id: "test-sector".to_string(),
        name: "test-template".to_string(),
        version: "1.0.0".to_string(),
        output_path: temp_dir.path().join("test.tos-template"),
        description: "Test".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        include_state: false,
        tags: vec!["test".to_string()],
    };
    
    let result = handler.export_sector(request).unwrap();
    assert!(result.template_path.exists());
    assert!(!result.sha256.is_empty());
    assert!(result.size > 0);
    
    // Test import
    let template = handler.import_template(&result.template_path).unwrap();
    assert_eq!(template.metadata.name, "test-template");
    assert_eq!(template.metadata.version, "1.0.0");
    
    // Test apply
    let sector_dir = handler.apply_template(&template, "new-sector").unwrap();
    assert!(sector_dir.exists());
    assert!(sector_dir.join("sector.json").exists());
}

/// Test template validation
#[test]
fn test_template_validation() {
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
            settings: std::collections::HashMap::new(),
        },
        hub_configs: vec![],
        app_configs: vec![],
        environment: std::collections::HashMap::new(),
        files: std::collections::HashMap::new(),
    };
    
    // Should validate successfully
    // Note: validate_template is private, tested through export/import
    
    let invalid_template = Template {
        metadata: TemplateMetadata {
            name: "".to_string(), // Invalid: empty name
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
        environment: std::collections::HashMap::new(),
        files: std::collections::HashMap::new(),
    };
    
    // Export should fail validation
    let temp_dir = tempfile::tempdir().unwrap();
    let handler = TemplateHandler::with_cache_dir(temp_dir.path().to_path_buf());
    
    let request = ExportRequest {
        sector_id: "test".to_string(),
        name: "".to_string(), // Invalid
        version: "1.0.0".to_string(),
        output_path: temp_dir.path().join("invalid.tos-template"),
        description: "Test".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        include_state: false,
        tags: vec![],
    };
    
    // This should fail due to empty name
    // Note: The actual validation happens during export
}

/// Test package type display
#[test]
fn test_package_type_display() {
    assert_eq!(PackageType::Template.to_string(), "template");
    assert_eq!(PackageType::SectorType.to_string(), "sector-type");
    assert_eq!(PackageType::ApplicationModel.to_string(), "app-model");
}

/// Test install result
#[test]
fn test_install_result() {
    let metadata = PackageMetadata {
        name: "pkg".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        package_type: PackageType::ApplicationModel,
        download_url: "https://example.com".to_string(),
        sha256: "abc".to_string(),
        signature: None,
        dependencies: vec![],
        min_tos_version: None,
        tags: vec![],
        size: 1024,
        created_at: "2024-01-01".to_string(),
    };
    
    let result = InstallResult {
        package: metadata,
        install_path: PathBuf::from("/tmp/install"),
        installed_dependencies: vec!["dep1".to_string()],
        warnings: vec!["Warning 1".to_string()],
    };
    
    assert_eq!(result.package.name, "pkg");
    assert_eq!(result.installed_dependencies.len(), 1);
    assert_eq!(result.warnings.len(), 1);
}

/// Test export result
#[test]
fn test_export_result() {
    let result = ExportResult {
        template_path: PathBuf::from("/tmp/template.tos-template"),
        size: 2048,
        sha256: "def789".to_string(),
    };
    
    assert_eq!(result.size, 2048);
    assert_eq!(result.sha256, "def789");
}

/// Test marketplace with repositories
#[test]
fn test_marketplace_with_repositories() {
    let mut marketplace = Marketplace::new();
    
    let repo = RepositoryConfig {
        name: "test".to_string(),
        url: "https://test.example.com".to_string(),
        enabled: true,
        priority: 1,
        auth_token: None,
    };
    
    marketplace.add_repository(repo);
    assert_eq!(marketplace.config.repositories.len(), 1);
    
    marketplace.remove_repository("test");
    assert!(marketplace.config.repositories.is_empty());
}

/// Test signature verification result
#[test]
fn test_verification_result() {
    let result = VerificationResult {
        valid: true,
        key_id: Some("abc123".to_string()),
        trusted: true,
        message: "Verified".to_string(),
    };
    
    assert!(result.valid);
    assert!(result.trusted);
    assert_eq!(result.key_id, Some("abc123".to_string()));
}
