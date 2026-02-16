// Basic Input Abstraction
// In a full Wayland compositor, this would wrap libinput events.
// Here, we bridge winit events to our logic core.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyCode {
    Escape,
    Space,
    Enter,
    Up,
    Down,
    Left,
    Right,
    Char(char),
    Unknown,
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    // A key press event
    KeyDown(KeyCode),
    // A key release event
    KeyUp(KeyCode),
    // Mouse movement (delta x, delta y)
    MouseMove { dx: f64, dy: f64 },
    // Mouse click
    MouseClick,
    // A high-level command from the terminal entry
    Command(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_variants_exist() {
        // Test all KeyCode variants can be created
        let _escape = KeyCode::Escape;
        let _space = KeyCode::Space;
        let _enter = KeyCode::Enter;
        let _up = KeyCode::Up;
        let _down = KeyCode::Down;
        let _left = KeyCode::Left;
        let _right = KeyCode::Right;
        let _char = KeyCode::Char('a');
        let _unknown = KeyCode::Unknown;
    }

    #[test]
    fn test_keycode_char_variant() {
        let key_a = KeyCode::Char('a');
        let key_z = KeyCode::Char('z');
        let key_1 = KeyCode::Char('1');
        
        // KeyCodes should be copyable
        let key_copy = key_a;
        assert_eq!(key_a, key_copy);
        
        // Different chars should not be equal
        assert_ne!(key_a, key_z);
        assert_ne!(key_a, key_1);
    }

    #[test]
    fn test_keycode_equality() {
        assert_eq!(KeyCode::Escape, KeyCode::Escape);
        assert_eq!(KeyCode::Space, KeyCode::Space);
        assert_eq!(KeyCode::Char('x'), KeyCode::Char('x'));
        
        assert_ne!(KeyCode::Escape, KeyCode::Space);
        assert_ne!(KeyCode::Up, KeyCode::Down);
        assert_ne!(KeyCode::Char('a'), KeyCode::Char('b'));
    }

    #[test]
    fn test_input_event_keydown() {
        let event = InputEvent::KeyDown(KeyCode::Escape);
        
        match event {
            InputEvent::KeyDown(KeyCode::Escape) => {}, // Success
            _ => panic!("Expected KeyDown(Escape)"),
        }
    }

    #[test]
    fn test_input_event_keyup() {
        let event = InputEvent::KeyUp(KeyCode::Enter);
        
        match event {
            InputEvent::KeyUp(KeyCode::Enter) => {}, // Success
            _ => panic!("Expected KeyUp(Enter)"),
        }
    }

    #[test]
    fn test_input_event_mouse_move() {
        let event = InputEvent::MouseMove { dx: 10.5, dy: -5.2 };
        
        match event {
            InputEvent::MouseMove { dx, dy } => {
                assert_eq!(dx, 10.5);
                assert_eq!(dy, -5.2);
            },
            _ => panic!("Expected MouseMove"),
        }
    }

    #[test]
    fn test_input_event_mouse_click() {
        let event = InputEvent::MouseClick;
        
        match event {
            InputEvent::MouseClick => {}, // Success
            _ => panic!("Expected MouseClick"),
        }
    }

    #[test]
    fn test_input_event_command() {
        let cmd = "zoom 2".to_string();
        let event = InputEvent::Command(cmd.clone());
        
        match event {
            InputEvent::Command(s) => assert_eq!(s, "zoom 2"),
            _ => panic!("Expected Command"),
        }
    }

    #[test]
    fn test_input_event_cloneable() {
        let event1 = InputEvent::Command("test".to_string());
        let event2 = event1.clone();
        
        // Both should be Command variants
        match (event1, event2) {
            (InputEvent::Command(s1), InputEvent::Command(s2)) => {
                assert_eq!(s1, s2);
            },
            _ => panic!("Both should be Command variants"),
        }
    }

    #[test]
    fn test_keycode_in_different_events() {
        let key = KeyCode::Space;
        
        let down = InputEvent::KeyDown(key);
        let up = InputEvent::KeyUp(key);
        
        // Verify both events contain the same key
        match down {
            InputEvent::KeyDown(KeyCode::Space) => {},
            _ => panic!("Expected KeyDown(Space)"),
        }
        
        match up {
            InputEvent::KeyUp(KeyCode::Space) => {},
            _ => panic!("Expected KeyUp(Space)"),
        }
    }
}
