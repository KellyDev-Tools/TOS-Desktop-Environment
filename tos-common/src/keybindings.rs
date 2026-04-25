//! §14.2: Configurable Keyboard Shortcut Mapping Layer
//!
//! Provides a user-configurable mapping from keyboard combinations to
//! `SemanticEvent` action identifiers. Defaults match the §14.2 spec table.
//! Users can remap via Settings → Keyboard, and the Brain detects conflicts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single key combination (e.g. "Ctrl+[", "Alt+1", "Ctrl+Shift+\\").
///
/// Serialized as a human-readable string for storage in settings JSON.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombo {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
    /// The key identifier (e.g. "[", "]", "Space", "t", "1", "Backspace", "\\").
    pub key: String,
}

impl KeyCombo {
    pub fn new(key: &str) -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
            key: key.to_string(),
        }
    }

    pub fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn meta(mut self) -> Self {
        self.meta = true;
        self
    }

    /// Convert to a canonical display string (e.g. "Ctrl+Shift+\\").
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.meta {
            parts.push("Meta");
        }
        parts.push(&self.key);
        parts.join("+")
    }

    /// Parse a display string back into a KeyCombo.
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return None;
        }
        let mut combo = KeyCombo::new("");
        for (i, part) in parts.iter().enumerate() {
            match part.to_lowercase().as_str() {
                "ctrl" => combo.ctrl = true,
                "alt" => combo.alt = true,
                "shift" => combo.shift = true,
                "meta" | "super" | "cmd" => combo.meta = true,
                _ => {
                    // Last non-modifier part is the key
                    if i == parts.len() - 1 {
                        combo.key = part.to_string();
                    } else {
                        return None; // Non-modifier in non-final position
                    }
                }
            }
        }
        if combo.key.is_empty() {
            return None;
        }
        Some(combo)
    }
}

impl std::fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// A keybinding entry: maps a key combo to a semantic action ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    /// The key combination that triggers this action.
    pub combo: KeyCombo,
    /// The semantic action identifier (e.g. "zoom_in", "toggle_bezel").
    pub action: String,
    /// Human-readable description of what this binding does.
    pub description: String,
}

/// The full keybinding map with conflict detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingMap {
    pub bindings: Vec<Keybinding>,
}

impl Default for KeybindingMap {
    fn default() -> Self {
        Self {
            bindings: default_keybindings(),
        }
    }
}

impl KeybindingMap {
    /// Look up the action for a given key combo.
    pub fn lookup(&self, combo: &KeyCombo) -> Option<&str> {
        self.bindings
            .iter()
            .find(|b| &b.combo == combo)
            .map(|b| b.action.as_str())
    }

    /// Set or update a keybinding. Returns any conflicting action that was displaced.
    pub fn set(&mut self, combo: KeyCombo, action: String, description: String) -> Option<String> {
        // Remove existing binding for this combo (conflict)
        let displaced = self
            .bindings
            .iter()
            .find(|b| b.combo == combo && b.action != action)
            .map(|b| b.action.clone());

        self.bindings.retain(|b| b.combo != combo);

        // Also remove any existing combo for this action (rebinding)
        self.bindings.retain(|b| b.action != action);

        self.bindings.push(Keybinding {
            combo,
            action,
            description,
        });

        displaced
    }

    /// Remove a keybinding by action.
    pub fn remove_action(&mut self, action: &str) {
        self.bindings.retain(|b| b.action != action);
    }

    /// Reset to defaults.
    pub fn reset(&mut self) {
        self.bindings = default_keybindings();
    }

    /// Detect conflicts: returns pairs of bindings that share the same combo.
    pub fn conflicts(&self) -> Vec<(String, String)> {
        let mut combo_map: HashMap<&KeyCombo, Vec<&str>> = HashMap::new();
        for b in &self.bindings {
            combo_map.entry(&b.combo).or_default().push(&b.action);
        }
        let mut conflicts = Vec::new();
        for actions in combo_map.values() {
            if actions.len() > 1 {
                for i in 0..actions.len() {
                    for j in (i + 1)..actions.len() {
                        conflicts.push((actions[i].to_string(), actions[j].to_string()));
                    }
                }
            }
        }
        conflicts
    }

    /// Serialize to JSON for IPC/settings.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

/// §14.2: Default keyboard shortcuts from the spec.
fn default_keybindings() -> Vec<Keybinding> {
    vec![
        Keybinding {
            combo: KeyCombo::new("[").ctrl(),
            action: "zoom_out".to_string(),
            description: "Move one level up in hierarchy".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("]").ctrl(),
            action: "zoom_in".to_string(),
            description: "Move one level down into focus".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("Space").ctrl(),
            action: "toggle_bezel".to_string(),
            description: "Expand/Collapse the Top Bezel".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("/").ctrl(),
            action: "set_mode_ai".to_string(),
            description: "Focus prompt and switch to AI mode".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("t").ctrl(),
            action: "new_sector".to_string(),
            description: "Create a new sector".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("1").alt(),
            action: "switch_sector_1".to_string(),
            description: "Switch to sector 1".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("2").alt(),
            action: "switch_sector_2".to_string(),
            description: "Switch to sector 2".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("3").alt(),
            action: "switch_sector_3".to_string(),
            description: "Switch to sector 3".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("4").alt(),
            action: "switch_sector_4".to_string(),
            description: "Switch to sector 4".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("5").alt(),
            action: "switch_sector_5".to_string(),
            description: "Switch to sector 5".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("6").alt(),
            action: "switch_sector_6".to_string(),
            description: "Switch to sector 6".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("7").alt(),
            action: "switch_sector_7".to_string(),
            description: "Switch to sector 7".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("8").alt(),
            action: "switch_sector_8".to_string(),
            description: "Switch to sector 8".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("9").alt(),
            action: "switch_sector_9".to_string(),
            description: "Switch to sector 9".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("m").ctrl(),
            action: "toggle_minimap".to_string(),
            description: "Show/Hide the Tactical Mini-Map".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("Backspace").ctrl().alt(),
            action: "tactical_reset".to_string(),
            description: "Trigger immediate Tactical Reset (Level 4 God Mode)".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("\\").ctrl(),
            action: "split_view".to_string(),
            description: "Split focused pane (auto-orientation)".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("\\").ctrl().shift(),
            action: "split_view_override".to_string(),
            description: "Split focused pane (orientation override)".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("w").ctrl(),
            action: "close_pane".to_string(),
            description: "Close focused pane".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("ArrowUp").ctrl(),
            action: "focus_pane_up".to_string(),
            description: "Move focus to pane above".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("ArrowDown").ctrl(),
            action: "focus_pane_down".to_string(),
            description: "Move focus to pane below".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("ArrowLeft").ctrl(),
            action: "focus_pane_left".to_string(),
            description: "Move focus to pane on left".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("ArrowRight").ctrl(),
            action: "focus_pane_right".to_string(),
            description: "Move focus to pane on right".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new(",").ctrl(),
            action: "open_settings".to_string(),
            description: "Open Settings panel".to_string(),
        },
        // Level switches (Ctrl+1 through Ctrl+4)
        Keybinding {
            combo: KeyCombo::new("1").ctrl(),
            action: "switch_level_1".to_string(),
            description: "Switch to Global Overview (Level 1)".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("2").ctrl(),
            action: "switch_level_2".to_string(),
            description: "Switch to Command Hub (Level 2)".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("3").ctrl(),
            action: "switch_level_3".to_string(),
            description: "Switch to Application Focus (Level 3)".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("4").ctrl(),
            action: "switch_level_4".to_string(),
            description: "Switch to Detail View (Level 4)".to_string(),
        },
        Keybinding {
            combo: KeyCombo::new("0").ctrl(),
            action: "split_equalize".to_string(),
            description: "Equalize all pane weights".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bindings_no_conflicts() {
        let map = KeybindingMap::default();
        let conflicts = map.conflicts();
        assert!(
            conflicts.is_empty(),
            "Default keybindings have conflicts: {:?}",
            conflicts
        );
    }

    #[test]
    fn test_lookup() {
        let map = KeybindingMap::default();
        let combo = KeyCombo::new("[").ctrl();
        assert_eq!(map.lookup(&combo), Some("zoom_out"));
    }

    #[test]
    fn test_set_displaces_conflict() {
        let mut map = KeybindingMap::default();
        let combo = KeyCombo::new("[").ctrl();

        // Remap Ctrl+[ to a custom action
        let displaced = map.set(
            combo.clone(),
            "custom_action".to_string(),
            "Custom".to_string(),
        );
        assert_eq!(displaced, Some("zoom_out".to_string()));
        assert_eq!(map.lookup(&combo), Some("custom_action"));
    }

    #[test]
    fn test_parse_combo() {
        let combo = KeyCombo::parse("Ctrl+Shift+\\").unwrap();
        assert!(combo.ctrl);
        assert!(combo.shift);
        assert!(!combo.alt);
        assert_eq!(combo.key, "\\");
    }

    #[test]
    fn test_display_roundtrip() {
        let combo = KeyCombo::new("Backspace").ctrl().alt();
        let display = combo.display();
        let parsed = KeyCombo::parse(&display).unwrap();
        assert_eq!(combo, parsed);
    }

    #[test]
    fn test_serde_roundtrip() {
        let map = KeybindingMap::default();
        let json = map.to_json();
        let restored = KeybindingMap::from_json(&json).unwrap();
        assert_eq!(map.bindings.len(), restored.bindings.len());
    }

    #[test]
    fn test_reset() {
        let mut map = KeybindingMap::default();
        let original_len = map.bindings.len();
        map.set(
            KeyCombo::new("x").ctrl(),
            "custom".to_string(),
            "test".to_string(),
        );
        assert_ne!(map.bindings.len(), original_len);
        map.reset();
        assert_eq!(map.bindings.len(), original_len);
    }
}
