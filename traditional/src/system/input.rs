// Basic Input Abstraction
// In a full Wayland compositor, this would wrap libinput events.
// Here, we bridge winit events to our logic core.

#[derive(Debug, Clone, Copy)]
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
