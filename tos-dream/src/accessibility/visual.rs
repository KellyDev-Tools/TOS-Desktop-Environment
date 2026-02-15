//! Visual Accessibility - High Contrast, Themes, and Color Filters
//! 
//! Provides visual accessibility features including high contrast modes,
//! colorblind filters, font scaling, and focus indicators.

use super::{AccessibilityConfig, ColorblindFilter, ThemeMode};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Visual accessibility manager
#[derive(Debug)]
pub struct VisualAccessibility {
    config: Arc<RwLock<AccessibilityConfig>>,
    current_theme: Arc<RwLock<Theme>>,
}

/// Theme definition with CSS generation
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub mode: ThemeMode,
    pub colors: ColorScheme,
    pub fonts: FontSettings,
    pub focus: FocusSettings,
}

/// Color scheme for a theme
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub secondary: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    pub border: String,
    pub highlight: String,
    pub disabled: String,
    pub link: String,
    pub visited: String,
}

/// Font settings
#[derive(Debug, Clone)]
pub struct FontSettings {
    pub family: String,
    pub base_size: f32,
    pub scale_factor: f32,
    pub line_height: f32,
    pub weight_normal: u16,
    pub weight_bold: u16,
    pub monospace_family: String,
}

/// Focus indicator settings
#[derive(Debug, Clone)]
pub struct FocusSettings {
    pub width: u32,
    pub style: String,
    pub color: String,
    pub offset: i32,
    pub animation: bool,
}

impl VisualAccessibility {
    /// Create a new visual accessibility manager
    pub async fn new(config: Arc<RwLock<AccessibilityConfig>>) -> Self {
        let cfg = config.read().await;
        let theme = Self::create_theme(&cfg);
        
        Self {
            config,
            current_theme: Arc::new(RwLock::new(theme)),
        }
    }
    
    /// Get CSS classes for current accessibility settings
    pub async fn get_css_classes(&self) -> String {
        let config = self.config.read().await;
        let theme = self.current_theme.read().await;
        
        let mut classes = vec![
            format!("theme-{}", theme_name(&theme.mode)),
            format!("font-scale-{}", scale_class(config.font_scale)),
        ];
        
        if config.high_contrast {
            classes.push("high-contrast".to_string());
        }
        
        if let Some(filter) = config.colorblind_filter {
            classes.push(format!("colorblind-{:?}", filter).to_lowercase());
        }
        
        classes.join(" ")
    }
    
    /// Generate CSS for the current theme
    pub async fn generate_css(&self) -> String {
        let config = self.config.read().await;
        let theme = self.current_theme.read().await;
        
        let css = format!(
            r#"
/* TOS Accessibility Theme CSS */
/* Generated for mode: {:?} */

:root {{
    /* Colors */
    --tos-bg: {bg};
    --tos-fg: {fg};
    --tos-accent: {accent};
    --tos-secondary: {secondary};
    --tos-success: {success};
    --tos-warning: {warning};
    --tos-error: {error};
    --tos-info: {info};
    --tos-border: {border};
    --tos-highlight: {highlight};
    --tos-disabled: {disabled};
    --tos-link: {link};
    --tos-visited: {visited};
    
    /* Typography */
    --tos-font-family: {font_family};
    --tos-font-mono: {font_mono};
    --tos-base-size: {base_size}px;
    --tos-scale: {scale};
    --tos-line-height: {line_height};
    
    /* Focus */
    --tos-focus-width: {focus_width}px;
    --tos-focus-style: {focus_style};
    --tos-focus-color: {focus_color};
    --tos-focus-offset: {focus_offset}px;
}}

/* High Contrast Overrides */
.high-contrast {{
    --tos-bg: #000000 !important;
    --tos-fg: #ffffff !important;
    --tos-accent: #ffff00 !important;
    --tos-border: #ffffff !important;
    --tos-focus-color: #00ffff !important;
}}

/* Font Scale Classes */
.font-scale-small {{ --tos-scale: 0.875; }}
.font-scale-normal {{ --tos-scale: 1.0; }}
.font-scale-large {{ --tos-scale: 1.25; }}
.font-scale-xlarge {{ --tos-scale: 1.5; }}

/* Colorblind Filters */
.colorblind-deuteranopia {{
    filter: url('#deuteranopia-filter');
}}
.colorblind-protanopia {{
    filter: url('#protanopia-filter');
}}
.colorblind-tritanopia {{
    filter: url('#tritanopia-filter');
}}
.colorblind-achromatopsia {{
    filter: grayscale(100%) contrast(150%);
}}

/* Focus Indicators */
*:focus {{
    outline: var(--tos-focus-width) var(--tos-focus-style) var(--tos-focus-color);
    outline-offset: var(--tos-focus-offset);
}}

.focus-visible:focus-visible {{
    outline: var(--tos-focus-width) var(--tos-focus-style) var(--tos-focus-color);
    outline-offset: var(--tos-focus-offset);
}}

/* Reduced Motion */
@media (prefers-reduced-motion: reduce) {{
    *, *::before, *::after {{
        animation-duration: 0.01ms !important;
        animation-iteration-count: 1 !important;
        transition-duration: 0.01ms !important;
    }}
}}

/* Screen Reader Only Content */
.sr-only {{
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border-width: 0;
}}

/* Skip Link */
.skip-link {{
    position: absolute;
    top: -40px;
    left: 0;
    background: var(--tos-accent);
    color: var(--tos-bg);
    padding: 8px;
    text-decoration: none;
    z-index: 100;
}}

.skip-link:focus {{
    top: 0;
}}
"#,
            bg = theme.colors.background,
            fg = theme.colors.foreground,
            accent = theme.colors.accent,
            secondary = theme.colors.secondary,
            success = theme.colors.success,
            warning = theme.colors.warning,
            error = theme.colors.error,
            info = theme.colors.info,
            border = theme.colors.border,
            highlight = theme.colors.highlight,
            disabled = theme.colors.disabled,
            link = theme.colors.link,
            visited = theme.colors.visited,
            font_family = theme.fonts.family,
            font_mono = theme.fonts.monospace_family,
            base_size = theme.fonts.base_size,
            scale = theme.fonts.scale_factor,
            line_height = theme.fonts.line_height,
            focus_width = theme.focus.width,
            focus_style = theme.focus.style,
            focus_color = theme.focus.color,
            focus_offset = theme.focus.offset,
        );
        
        css
    }
    
    /// Generate SVG filters for colorblind simulation
    pub fn generate_colorblind_filters() -> String {
        r#"
<svg xmlns="http://www.w3.org/2000/svg" style="display: none;">
    <defs>
        <!-- Deuteranopia (Green-weak) Filter -->
        <filter id="deuteranopia-filter">
            <feColorMatrix type="matrix" values="
                0.625 0.375 0     0 0
                0.7   0.3   0     0 0
                0     0.3   0.7   0 0
                0     0     0     1 0"/>
        </filter>
        
        <!-- Protanopia (Red-weak) Filter -->
        <filter id="protanopia-filter">
            <feColorMatrix type="matrix" values="
                0.567 0.433 0     0 0
                0.558 0.442 0     0 0
                0     0.242 0.758 0 0
                0     0     0     1 0"/>
        </filter>
        
        <!-- Tritanopia (Blue-weak) Filter -->
        <filter id="tritanopia-filter">
            <feColorMatrix type="matrix" values="
                0.95  0.05  0     0 0
                0     0.433 0.567 0 0
                0     0.475 0.525 0 0
                0     0     0     1 0"/>
        </filter>
    </defs>
</svg>
"#.to_string()
    }
    
    /// Get ARIA role for a component type
    pub fn get_aria_role(component_type: &str) -> &'static str {
        match component_type {
            "sector" => "region",
            "hub" => "main",
            "application" => "application",
            "bezel" => "toolbar",
            "prompt" => "textbox",
            "suggestion" => "listbox",
            "button" => "button",
            "menu" => "menu",
            "menuitem" => "menuitem",
            "dialog" => "dialog",
            "alert" => "alert",
            "status" => "status",
            "navigation" => "navigation",
            "tab" => "tab",
            "tabpanel" => "tabpanel",
            "tree" => "tree",
            "treeitem" => "treeitem",
            _ => "generic",
        }
    }
    
    /// Generate ARIA attributes for a component
    pub fn generate_aria_attributes(
        component_type: &str,
        label: &str,
        state: Option<&str>,
        expanded: Option<bool>,
        selected: Option<bool>,
    ) -> String {
        let role = Self::get_aria_role(component_type);
        let mut attrs = format!(r#"role="{}" aria-label="{}""#, role, label);
        
        if let Some(s) = state {
            attrs.push_str(&format!(r#" aria-state="{}""#, s));
        }
        
        if let Some(e) = expanded {
            attrs.push_str(&format!(r#" aria-expanded="{}""#, e));
        }
        
        if let Some(s) = selected {
            attrs.push_str(&format!(r#" aria-selected="{}""#, s));
        }
        
        attrs
    }
    
    /// Update theme based on configuration changes
    pub async fn refresh_theme(&self) {
        let config = self.config.read().await;
        let new_theme = Self::create_theme(&config);
        
        let mut theme = self.current_theme.write().await;
        *theme = new_theme;
    }
    
    /// Create a theme from configuration
    fn create_theme(config: &AccessibilityConfig) -> Theme {
        let colors = if config.high_contrast {
            Self::high_contrast_colors()
        } else {
            match config.theme_mode {
                ThemeMode::Light => Self::light_colors(),
                ThemeMode::Dark | ThemeMode::System => Self::dark_colors(),
                ThemeMode::HighContrast => Self::high_contrast_colors(),
            }
        };
        
        let fonts = FontSettings {
            family: "system-ui, -apple-system, sans-serif".to_string(),
            base_size: 16.0,
            scale_factor: config.font_scale,
            line_height: 1.5,
            weight_normal: 400,
            weight_bold: 700,
            monospace_family: "ui-monospace, monospace".to_string(),
        };
        
        let focus = FocusSettings {
            width: if config.high_contrast { 4 } else { 2 },
            style: "solid".to_string(),
            color: if config.high_contrast {
                "#00ffff".to_string()
            } else {
                "#ff9900".to_string()
            },
            offset: 2,
            animation: !config.high_contrast,
        };
        
        Theme {
            name: format!("{:?}", config.theme_mode).to_lowercase(),
            mode: config.theme_mode,
            colors,
            fonts,
            focus,
        }
    }
    
    /// Dark theme colors (default TOS look)
    fn dark_colors() -> ColorScheme {
        ColorScheme {
            background: "#1a1a2e".to_string(),
            foreground: "#e0e0e0".to_string(),
            accent: "#ff9900".to_string(),
            secondary: "#9999cc".to_string(),
            success: "#66cc66".to_string(),
            warning: "#ffcc66".to_string(),
            error: "#cc6666".to_string(),
            info: "#66ccff".to_string(),
            border: "#444466".to_string(),
            highlight: "#ffcc00".to_string(),
            disabled: "#666666".to_string(),
            link: "#99ccff".to_string(),
            visited: "#cc99ff".to_string(),
        }
    }
    
    /// Light theme colors
    fn light_colors() -> ColorScheme {
        ColorScheme {
            background: "#f5f5f5".to_string(),
            foreground: "#1a1a2e".to_string(),
            accent: "#cc6600".to_string(),
            secondary: "#666699".to_string(),
            success: "#339933".to_string(),
            warning: "#cc9900".to_string(),
            error: "#cc3333".to_string(),
            info: "#0066cc".to_string(),
            border: "#cccccc".to_string(),
            highlight: "#ff9900".to_string(),
            disabled: "#999999".to_string(),
            link: "#0066cc".to_string(),
            visited: "#663399".to_string(),
        }
    }
    
    /// High contrast theme colors
    fn high_contrast_colors() -> ColorScheme {
        ColorScheme {
            background: "#000000".to_string(),
            foreground: "#ffffff".to_string(),
            accent: "#ffff00".to_string(),
            secondary: "#00ffff".to_string(),
            success: "#00ff00".to_string(),
            warning: "#ffff00".to_string(),
            error: "#ff0000".to_string(),
            info: "#00ffff".to_string(),
            border: "#ffffff".to_string(),
            highlight: "#ffff00".to_string(),
            disabled: "#666666".to_string(),
            link: "#00ffff".to_string(),
            visited: "#ff00ff".to_string(),
        }
    }
    
    /// Get colorblind-friendly palette
    pub fn colorblind_safe_colors(filter: ColorblindFilter) -> Vec<String> {
        // Colorblind-safe palettes from ColorBrewer
        match filter {
            ColorblindFilter::Deuteranopia | ColorblindFilter::Protanopia => {
                // Blue-Orange safe for red-green colorblindness
                vec![
                    "#1f77b4".to_string(), // Blue
                    "#ff7f0e".to_string(), // Orange
                    "#2ca02c".to_string(), // Green (distinguishable)
                    "#d62728".to_string(), // Red (distinguishable)
                    "#9467bd".to_string(), // Purple
                    "#8c564b".to_string(), // Brown
                ]
            }
            ColorblindFilter::Tritanopia => {
                // Red-Green safe for blue-yellow colorblindness
                vec![
                    "#e41a1c".to_string(), // Red
                    "#377eb8".to_string(), // Blue
                    "#4daf4a".to_string(), // Green
                    "#984ea3".to_string(), // Purple
                    "#ff7f00".to_string(), // Orange
                    "#ffff33".to_string(), // Yellow
                ]
            }
            ColorblindFilter::Achromatopsia => {
                // High contrast grayscale
                vec![
                    "#000000".to_string(),
                    "#333333".to_string(),
                    "#666666".to_string(),
                    "#999999".to_string(),
                    "#cccccc".to_string(),
                    "#ffffff".to_string(),
                ]
            }
        }
    }
}

fn theme_name(mode: &ThemeMode) -> String {
    match mode {
        ThemeMode::System => "system".to_string(),
        ThemeMode::Light => "light".to_string(),
        ThemeMode::Dark => "dark".to_string(),
        ThemeMode::HighContrast => "high-contrast".to_string(),
    }
}

fn scale_class(scale: f32) -> String {
    if scale < 0.9 {
        "small".to_string()
    } else if scale < 1.1 {
        "normal".to_string()
    } else if scale < 1.4 {
        "large".to_string()
    } else {
        "xlarge".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_visual_accessibility() {
        let config = Arc::new(RwLock::new(AccessibilityConfig::default()));
        let visual = VisualAccessibility::new(config).await;
        
        let classes = visual.get_css_classes().await;
        assert!(classes.contains("theme-"));
    }

    #[test]
    fn test_aria_roles() {
        assert_eq!(VisualAccessibility::get_aria_role("button"), "button");
        assert_eq!(VisualAccessibility::get_aria_role("dialog"), "dialog");
        assert_eq!(VisualAccessibility::get_aria_role("unknown"), "generic");
    }

    #[test]
    fn test_aria_attributes() {
        let attrs = VisualAccessibility::generate_aria_attributes(
            "button",
            "Submit",
            Some("active"),
            Some(true),
            None,
        );
        assert!(attrs.contains(r#"role="button""#));
        assert!(attrs.contains(r#"aria-label="Submit""#));
        assert!(attrs.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn test_colorblind_filters() {
        let filters = VisualAccessibility::generate_colorblind_filters();
        assert!(filters.contains("deuteranopia-filter"));
        assert!(filters.contains("protanopia-filter"));
        assert!(filters.contains("tritanopia-filter"));
    }

    #[test]
    fn test_colorblind_safe_colors() {
        let colors = VisualAccessibility::colorblind_safe_colors(ColorblindFilter::Deuteranopia);
        assert_eq!(colors.len(), 6);
        assert!(colors[0].starts_with('#'));
    }
}
