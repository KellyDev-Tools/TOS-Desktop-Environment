# Accessibility & Input-Agnostic Design

A truly modern desktop must be usable by everyone. TOS is built on an **input-agnostic** philosophy: while the LCARS aesthetic is touch-friendly, every core interaction is designed to be equally efficient via keyboard, mouse, voice, and assistive devices.

---

## CSS-Driven Theming in Settings

Since the UI is web-based (HTML/CSS/JS via WebView), accessibility modes are implemented as CSS theme overrides, toggleable via the settings panel.

### High-Contrast Modes
*   **Theme Selection**: The settings panel includes dedicated high-contrast color palettes (e.g., black background with bright yellow/white text).
*   **LCARS Adaptation**: High-contrast variants maintain the LCARS aesthetic using bolder, more saturated versions of signature colors.

### UI Scaling & Large Text
*   **Global Scaling Slider**: A slider in the settings panel allows users to scale the entire UI (via CSS variables like `--font-size-base`) to benefit users with visual impairments.
*   **Dynamic Resizing**: All elements, including LCARS elbows and panels, scale proportionally.

### Simplified Zoom Navigation
*   **Reduced Motion**: A toggle to disable the vertical scale/opacity interpolation, replacing it with a simple cross-fade between hierarchy levels.
*   **Sticky Zoom**: An option to lock the zoom level at Level 2 (Launcher) or Level 3 (Focus) to prevent accidental transitions via gestures.

---

## Screen Reader & Navigation Settings

*   **ARIA & Semantic HTML**: The UI uses semantic HTML5 and ARIA roles to ensure compatibility with screen readers like Orca.
*   **Focus Visibility Settings**: Options to customize the thickness and color of the keyboard focus indicator.
*   **Logical Navigation**: Settings to define custom tab orders or navigation shortcuts.

---

## Input & Gesture Customization

The settings panel provides granular control for users with motor impairments:

*   **Gesture Thresholds**: Adjust sensitivity for swipe/pinch distance and velocity.
*   **Dwell Click**: A toggleable feature to trigger a click by hovering over an element for a configurable duration.
*   **Input Mapping**: Full support for remapping keyboard-only navigation and switch devices, ensuring no interaction is locked to a specific input method.
