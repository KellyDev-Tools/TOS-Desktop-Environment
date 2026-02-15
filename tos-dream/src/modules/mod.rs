//! TOS Module System - Phase 8 Implementation
//! 
//! Provides hot-loadable Application Models and Sector Types
//! with optional containerization and scripting support.

pub mod manifest;
pub mod registry;
pub mod loader;
pub mod app_model;
pub mod sector_type;
pub mod script;

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
