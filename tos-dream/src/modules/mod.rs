//! TOS Module System - Phase 8 & 9 Implementation
//! 
//! Provides hot-loadable Application Models and Sector Types
//! with optional containerization and scripting support.
//! 
//! Phase 9 adds Marketplace and Templates support for package
//! management, repository handling, and digital signatures.

pub mod manifest;
pub mod registry;
pub mod loader;
pub mod app_model;
pub mod sector_type;
pub mod script;

// Phase 9: Marketplace and Templates
pub use crate::marketplace::{
    Marketplace,
    MarketplaceConfig,
    MarketplaceError,
    RepositoryConfig,
    PackageMetadata,
    InstallRequest,
    InstallResult,
    ExportRequest,
    ExportResult,
    PackageType,
    Template,
    TemplateMetadata,
    SignatureVerifier,
    DependencyResolver,
    RepositoryManager,
    PackageManager,
};

// Re-export commonly used types
pub use manifest::{
    ModuleManifest, 
    ModuleType, 
    ContainerConfig, 
    ContainerBackend,
    ManifestError
};
pub use registry::{
    ModuleRegistry, 
    ModuleInfo, 
    ModuleState
};
pub use loader::ModuleLoader;
pub use app_model::{
    AppModel, 
    AppModelRegistry,
    BezelAction,
    DecorationPolicy
};
pub use sector_type::{
    SectorTypeImpl, 
    SectorTypeRegistry,
    CommandFavorite,
    InterestingDirectory,
    DirectoryPatternType
};
pub use script::{
    ScriptEngine,
    ScriptLanguage,
    ScriptError,
    ScriptEngineFactory,
    generate_module_template
};
