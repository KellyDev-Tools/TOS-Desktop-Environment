//! Auditory Interface - Sound Themes
//!
//! Manages installable sound themes for the TOS auditory interface.
//! Themes can be installed from the marketplace and customize all earcons.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::system::audio::earcons::{EarconEvent, EarconCategory};

/// A sound theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundTheme {
    /// Theme metadata
    pub metadata: ThemeMetadata,
    /// Sound definitions for each earcon event
    pub sounds: HashMap<EarconEvent, SoundDefinition>,
    /// Category-specific settings
    pub category_settings: HashMap<EarconCategory, CategorySettings>,
    /// Master theme settings
    pub master_settings: MasterSettings,
}

/// Theme metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMetadata {
    /// Theme identifier (unique)
    pub id: String,
    /// Display name
    pub name: String,
    /// Theme version
    pub version: String,
    /// Author/creator
    pub author: String,
    /// Theme description
    pub description: String,
    /// License
    pub license: String,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Minimum TOS version required
    pub min_tos_version: String,
    /// Theme preview sound (for theme browser)
    pub preview_sound: Option<String>,
}

/// Sound definition for a specific earcon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundDefinition {
    /// Event this sound is for
    pub event: EarconEvent,
    /// Sound file path (relative to theme directory)
    pub file_path: String,
    /// Volume multiplier (0.0 - 1.0, relative to category)
    pub volume: f32,
    /// Playback speed (1.0 = normal)
    pub speed: f32,
    /// Whether to loop the sound
    pub loop_sound: bool,
    /// Fade in duration
    pub fade_in_ms: u32,
    /// Fade out duration
    pub fade_out_ms: u32,
    /// Sound pitch shift in semitones
    pub pitch_shift: i8,
}

/// Category-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySettings {
    /// Category this applies to
    pub category: EarconCategory,
    /// Base volume for the category
    pub base_volume: f32,
    /// Whether sounds in this category use spatial audio
    pub spatial_audio: bool,
    /// Reverb amount (0.0 - 1.0)
    pub reverb: f32,
    /// Sound filter/effect to apply
    pub filter: Option<String>,
}

/// Master theme settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterSettings {
    /// Master volume multiplier
    pub master_volume: f32,
    /// Global playback speed
    pub global_speed: f32,
    /// Whether to compress dynamic range
    pub compression: bool,
    /// Maximum polyphony (concurrent sounds)
    pub max_polyphony: usize,
    /// Debounce time between identical sounds (ms)
    pub debounce_ms: u32,
}

impl Default for MasterSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            global_speed: 1.0,
            compression: false,
            max_polyphony: 8,
            debounce_ms: 50,
        }
    }
}

impl CategorySettings {
    /// Create category settings for a specific category
    pub fn for_category(category: EarconCategory) -> Self {
        Self {
            category,
            base_volume: category.default_volume(),
            spatial_audio: matches!(category, EarconCategory::Collaboration),
            reverb: 0.0,
            filter: None,
        }
    }
}

impl Default for CategorySettings {
    fn default() -> Self {
        Self::for_category(EarconCategory::Navigation)
    }
}

impl SoundDefinition {
    /// Create a default sound definition for an event
    pub fn default_for(event: EarconEvent) -> Self {
        Self {
            event,
            file_path: format!("{}.wav", event.sound_pattern()),
            volume: 1.0,
            speed: 1.0,
            loop_sound: false,
            fade_in_ms: 0,
            fade_out_ms: 0,
            pitch_shift: 0,
        }
    }
}

impl SoundTheme {
    /// Create a new minimal theme with default settings
    pub fn new(id: &str, name: &str, author: &str) -> Self {
        let metadata = ThemeMetadata {
            id: id.to_string(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            author: author.to_string(),
            description: format!("{} sound theme", name),
            license: "MIT".to_string(),
            tags: vec!["default".to_string()],
            min_tos_version: "0.1.0".to_string(),
            preview_sound: None,
        };
        
        let mut sounds = HashMap::new();
        let mut category_settings = HashMap::new();
        
        // Initialize all events with default sounds
        for event in Self::all_events() {
            sounds.insert(event, SoundDefinition::default_for(event));
        }
        
        // Initialize category settings
        for category in Self::all_categories() {
            category_settings.insert(category, CategorySettings::for_category(category));
        }
        
        Self {
            metadata,
            sounds,
            category_settings,
            master_settings: MasterSettings::default(),
        }
    }
    
    /// Get all earcon events
    fn all_events() -> Vec<EarconEvent> {
        vec![
            // Navigation
            EarconEvent::ZoomIn,
            EarconEvent::ZoomOut,
            EarconEvent::LevelChange,
            EarconEvent::FocusChange,
            EarconEvent::SplitViewCreated,
            EarconEvent::SplitViewClosed,
            // Command Feedback
            EarconEvent::CommandAccepted,
            EarconEvent::CommandError,
            EarconEvent::CommandCompleted,
            EarconEvent::DangerousCommandWarning,
            EarconEvent::AutoCompleteSuggestion,
            // System Status
            EarconEvent::Notification,
            EarconEvent::TacticalAlert,
            EarconEvent::BatteryLow,
            EarconEvent::BatteryCritical,
            EarconEvent::PerformanceWarning,
            // Collaboration
            EarconEvent::UserJoined,
            EarconEvent::UserLeft,
            EarconEvent::CursorShared,
            EarconEvent::FollowingStarted,
            EarconEvent::FollowingEnded,
            // Bezel/UI
            EarconEvent::BezelExpand,
            EarconEvent::BezelCollapse,
            EarconEvent::ButtonHover,
            EarconEvent::ModeSwitch,
            EarconEvent::ToggleHiddenFiles,
        ]
    }
    
    /// Get all earcon categories
    fn all_categories() -> Vec<EarconCategory> {
        vec![
            EarconCategory::Navigation,
            EarconCategory::CommandFeedback,
            EarconCategory::SystemStatus,
            EarconCategory::Collaboration,
            EarconCategory::BezelUi,
        ]
    }
    
    /// Get sound definition for an event
    pub fn get_sound(&self, event: EarconEvent) -> Option<&SoundDefinition> {
        self.sounds.get(&event)
    }
    
    /// Get mutable sound definition for an event
    pub fn get_sound_mut(&mut self, event: EarconEvent) -> Option<&mut SoundDefinition> {
        self.sounds.get_mut(&event)
    }
    
    /// Set sound definition for an event
    pub fn set_sound(&mut self, event: EarconEvent, sound: SoundDefinition) {
        self.sounds.insert(event, sound);
    }
    
    /// Get category settings
    pub fn get_category_settings(&self, category: EarconCategory) -> Option<&CategorySettings> {
        self.category_settings.get(&category)
    }
    
    /// Calculate final volume for an event
    pub fn calculate_volume(&self, event: EarconEvent) -> f32 {
        let master = self.master_settings.master_volume;
        
        let category = event.category();
        let category_vol = self.category_settings
            .get(&category)
            .map(|s| s.base_volume)
            .unwrap_or(1.0);
        
        let sound_vol = self.sounds
            .get(&event)
            .map(|s| s.volume)
            .unwrap_or(1.0);
        
        (master * category_vol * sound_vol).clamp(0.0, 1.0)
    }
    
    /// Save theme to a directory
    pub fn save_to_directory(&self, path: &Path) -> Result<(), ThemeError> {
        std::fs::create_dir_all(path)?;
        
        // Save theme manifest
        let manifest_path = path.join("theme.toml");
        let manifest = toml::to_string_pretty(self)?;
        std::fs::write(manifest_path, manifest)?;
        
        Ok(())
    }
    
    /// Load theme from a directory
    pub fn load_from_directory(path: &Path) -> Result<Self, ThemeError> {
        let manifest_path = path.join("theme.toml");
        let manifest_content = std::fs::read_to_string(manifest_path)?;
        let theme: SoundTheme = toml::from_str(&manifest_content)?;
        Ok(theme)
    }
    
    /// Create the "Default" theme
    pub fn default_theme() -> Self {
        Self::new("default", "Default", "TOS Team")
    }
    
    /// Create the "Minimal" theme (fewer sounds, quieter)
    pub fn minimal_theme() -> Self {
        let mut theme = Self::new("minimal", "Minimal", "TOS Team");
        theme.metadata.description = "Minimal sound theme with reduced audio feedback".to_string();
        theme.metadata.tags = vec!["minimal".to_string(), "quiet".to_string()];
        
        // Reduce volumes across the board
        theme.master_settings.master_volume = 0.5;
        
        // Disable some UI sounds
        for event in [EarconEvent::ButtonHover, EarconEvent::ModeSwitch] {
            if let Some(sound) = theme.sounds.get_mut(&event) {
                sound.volume = 0.0;
            }
        }
        
        theme
    }
    
    /// Create the "LCARS" theme (Star Trek inspired)
    pub fn lcars_theme() -> Self {
        let mut theme = Self::new("lcars", "LCARS", "TOS Team");
        theme.metadata.description = "Star Trek LCARS-inspired sound theme".to_string();
        theme.metadata.tags = vec!["lcars".to_string(), "scifi".to_string()];
        
        // LCARS-style sounds would have specific file mappings
        for event in theme.sounds.values_mut() {
            event.file_path = format!("lcars/{}.wav", event.event.sound_pattern());
        }
        
        theme
    }
    
    /// Validate the theme (check all required sounds exist)
    pub fn validate(&self, base_path: &Path) -> Result<(), ThemeError> {
        for (event, sound) in &self.sounds {
            if sound.volume > 0.0 {
                let sound_path = base_path.join(&sound.file_path);
                if !sound_path.exists() {
                    return Err(ThemeError::MissingSoundFile {
                        event: format!("{:?}", event),
                        path: sound_path,
                    });
                }
            }
        }
        Ok(())
    }
}

/// Theme manager handles loading and switching themes
#[derive(Debug)]
pub struct ThemeManager {
    /// Loaded themes
    themes: HashMap<String, SoundTheme>,
    /// Currently active theme ID
    active_theme: String,
    /// Theme search paths
    search_paths: Vec<PathBuf>,
    /// Default theme as fallback
    default_theme: SoundTheme,
}

/// Errors that can occur with themes
#[derive(Debug)]
pub enum ThemeError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    TomlSerialize(toml::ser::Error),
    MissingSoundFile { event: String, path: PathBuf },
    ThemeNotFound(String),
    InvalidTheme(String),
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Toml(e) => write!(f, "TOML parse error: {}", e),
            Self::TomlSerialize(e) => write!(f, "TOML serialize error: {}", e),
            Self::MissingSoundFile { event, path } => {
                write!(f, "Missing sound file for {}: {}", event, path.display())
            }
            Self::ThemeNotFound(id) => write!(f, "Theme not found: {}", id),
            Self::InvalidTheme(msg) => write!(f, "Invalid theme: {}", msg),
        }
    }
}

impl std::error::Error for ThemeError {}

impl From<std::io::Error> for ThemeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for ThemeError {
    fn from(e: toml::de::Error) -> Self {
        Self::Toml(e)
    }
}

impl From<toml::ser::Error> for ThemeError {
    fn from(e: toml::ser::Error) -> Self {
        Self::TomlSerialize(e)
    }
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new() -> Self {
        let default_theme = SoundTheme::default_theme();
        let mut themes = HashMap::new();
        themes.insert(default_theme.metadata.id.clone(), default_theme.clone());
        
        Self {
            themes,
            active_theme: "default".to_string(),
            search_paths: Vec::new(),
            default_theme,
        }
    }
    
    /// Add a theme search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }
    
    /// Set default search paths (user and system directories)
    pub fn set_default_paths(&mut self) {
        if let Some(data_dir) = dirs::data_dir() {
            self.search_paths.push(data_dir.join("tos/themes"));
        }
        self.search_paths.push(PathBuf::from("/usr/share/tos/themes"));
        self.search_paths.push(PathBuf::from("/usr/local/share/tos/themes"));
    }
    
    /// Scan search paths and load all available themes
    pub fn scan_and_load(&mut self) -> Result<Vec<String>, ThemeError> {
        let mut loaded = Vec::new();
        
        for path in &self.search_paths {
            if !path.exists() {
                continue;
            }
            
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let theme_dir = entry.path();
                if theme_dir.is_dir() {
                    match SoundTheme::load_from_directory(&theme_dir) {
                        Ok(theme) => {
                            let id = theme.metadata.id.clone();
                            self.themes.insert(id.clone(), theme);
                            loaded.push(id);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load theme from {}: {}", theme_dir.display(), e);
                        }
                    }
                }
            }
        }
        
        Ok(loaded)
    }
    
    /// Get a theme by ID
    pub fn get_theme(&self, id: &str) -> Option<&SoundTheme> {
        self.themes.get(id)
    }
    
    /// Get the currently active theme
    pub fn active_theme(&self) -> &SoundTheme {
        self.themes.get(&self.active_theme)
            .unwrap_or(&self.default_theme)
    }
    
    /// Switch to a different theme
    pub fn switch_theme(&mut self, id: &str) -> Result<(), ThemeError> {
        if self.themes.contains_key(id) {
            self.active_theme = id.to_string();
            tracing::info!("Switched to sound theme: {}", id);
            Ok(())
        } else {
            Err(ThemeError::ThemeNotFound(id.to_string()))
        }
    }
    
    /// Install a theme from a directory
    pub fn install_theme(&mut self, source_path: &Path) -> Result<String, ThemeError> {
        let theme = SoundTheme::load_from_directory(source_path)?;
        let id = theme.metadata.id.clone();
        
        // Copy to user themes directory
        if let Some(data_dir) = dirs::data_dir() {
            let themes_dir = data_dir.join("tos/themes");
            let target_path = themes_dir.join(&id);
            std::fs::create_dir_all(&target_path)?;
            
            // Copy all files
            for entry in std::fs::read_dir(source_path)? {
                let entry = entry?;
                let source = entry.path();
                let target = target_path.join(entry.file_name());
                
                if source.is_file() {
                    std::fs::copy(&source, &target)?;
                }
            }
            
            self.themes.insert(id.clone(), theme);
            tracing::info!("Installed theme: {}", id);
            Ok(id)
        } else {
            Err(ThemeError::InvalidTheme("Could not determine data directory".to_string()))
        }
    }
    
    /// Get list of available theme IDs
    pub fn available_themes(&self) -> Vec<&String> {
        self.themes.keys().collect()
    }
    
    /// Get list of available themes with metadata
    pub fn available_themes_with_metadata(&self) -> Vec<&ThemeMetadata> {
        self.themes.values().map(|t| &t.metadata).collect()
    }
    
    /// Uninstall a theme
    pub fn uninstall_theme(&mut self, id: &str) -> Result<(), ThemeError> {
        if id == "default" {
            return Err(ThemeError::InvalidTheme("Cannot uninstall default theme".to_string()));
        }
        
        // Remove from filesystem
        if let Some(data_dir) = dirs::data_dir() {
            let theme_dir = data_dir.join("tos/themes").join(id);
            if theme_dir.exists() {
                std::fs::remove_dir_all(&theme_dir)?;
            }
        }
        
        // Remove from memory
        self.themes.remove(id);
        
        // Switch to default if this was the active theme
        if self.active_theme == id {
            self.active_theme = "default".to_string();
        }
        
        tracing::info!("Uninstalled theme: {}", id);
        Ok(())
    }
    
    /// Reload the active theme
    pub fn reload_active_theme(&mut self) -> Result<(), ThemeError> {
        let active_id = self.active_theme.clone();
        if let Some(data_dir) = dirs::data_dir() {
            let theme_dir = data_dir.join("tos/themes").join(&active_id);
            if theme_dir.exists() {
                let theme = SoundTheme::load_from_directory(&theme_dir)?;
                self.themes.insert(active_id, theme);
            }
        }
        Ok(())
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_theme_creation() {
        let theme = SoundTheme::new("test", "Test Theme", "Test Author");
        assert_eq!(theme.metadata.id, "test");
        assert_eq!(theme.metadata.name, "Test Theme");
        assert!(!theme.sounds.is_empty());
    }
    
    #[test]
    fn test_default_theme() {
        let theme = SoundTheme::default_theme();
        assert_eq!(theme.metadata.id, "default");
        assert!(theme.sounds.contains_key(&EarconEvent::ZoomIn));
    }
    
    #[test]
    fn test_minimal_theme() {
        let theme = SoundTheme::minimal_theme();
        assert_eq!(theme.metadata.id, "minimal");
        assert_eq!(theme.master_settings.master_volume, 0.5);
    }
    
    #[test]
    fn test_volume_calculation() {
        let theme = SoundTheme::default_theme();
        let volume = theme.calculate_volume(EarconEvent::ZoomIn);
        assert!(volume > 0.0 && volume <= 1.0);
    }
    
    #[test]
    fn test_theme_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let theme = SoundTheme::default_theme();
        
        // Save
        theme.save_to_directory(temp_dir.path()).unwrap();
        
        // Load
        let loaded = SoundTheme::load_from_directory(temp_dir.path()).unwrap();
        assert_eq!(loaded.metadata.id, theme.metadata.id);
    }
    
    #[test]
    fn test_theme_manager() {
        let mut manager = ThemeManager::new();
        
        // Should have default theme
        assert!(manager.get_theme("default").is_some());
        assert_eq!(manager.active_theme().metadata.id, "default");
        
        // Add built-in themes
        let minimal = SoundTheme::minimal_theme();
        let minimal_id = minimal.metadata.id.clone();
        manager.themes.insert(minimal_id.clone(), minimal);
        
        // Switch theme
        manager.switch_theme(&minimal_id).unwrap();
        assert_eq!(manager.active_theme().metadata.id, minimal_id);
        
        // List themes
        let themes = manager.available_themes();
        assert!(themes.contains(&&"default".to_string()));
        assert!(themes.contains(&&minimal_id));
    }
    
    #[test]
    fn test_theme_validation() {
        let temp_dir = TempDir::new().unwrap();
        let theme = SoundTheme::default_theme();
        
        // Save without actual sound files
        theme.save_to_directory(temp_dir.path()).unwrap();
        
        // Validation should fail (missing sound files)
        let result = theme.validate(temp_dir.path());
        assert!(result.is_err());
    }
}
