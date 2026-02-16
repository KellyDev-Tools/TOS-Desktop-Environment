# The Dream: TOS (Tactical Operating System)

The goal is to create an **input-agnostic** functional desktop environment with an LCARS-inspired user interface for a fully functional Linux desktop. The core of this interaction is a **Recursive Zoom Hierarchy**.

## The Vision: Depth Over Panning
Instead of an infinite canvas that requires panning, TOS uses **Vertical Depth**.
1.  **Zoom Out** to see the big picture (Level 1: Monitors/Sectors).
2.  **Zoom In** to your desktop (Level 2: App Launcher).
3.  **Zoom Deeper** into your work (Level 3: Full-screen Application).

## Key Features*   
*   **Persistent Unified Prompt**: A terminal command entry point that is always quickly available. It supports a **Wait-to-Execute** staging pattern: visual interactions (like tapping a file) populate the prompt for review before manual execution.
*   **Input-Agnostic Design**: While the LCARS aesthetic is naturally touch-friendly (large targets, clear grouping), TOS is designed to be fully functional via mouse, keyboard, voice, and switch-devices.
*   **Window Management via Tiling**: Supported through a **Split** mechanism. Users can split their view into two or more sectors, each maintaining independent zoom depth.
*   **Fluid Navigation**:
    - **Touch**: Pinch-to-zoom / Tap.
    - **Mouse**: Scroll-wheel zoom / Click.
    - **Keyboard**: Enter (Zoom In) / Escape (Zoom Out).

## Technical Foundation
*   **Language**: Rust (Back-end) with a Webview/CSS UI (Front-end).
*   **Display Server**: Wayland.
*   **Modular Shell Architecture**: 
    - **TOS Shell API**: A standardized set of OSC escape sequences and event protocols that the compositor uses to synchronize the spatial UI with the terminal state.
    - **Shell Modules**: Individual plugins or configurations for specific shells (Fish, Zsh, Bash) that implement the TOS Shell API. 
    - **Official Module**: **Fish** is the default reference implementation, providing the deepest "out-of-the-box" integration.
    - **Agnosticism**: Users can swap the underlying shell as long as a corresponding TOS Module is active to drive the metadata injection.
*   **Desktop Parity**: TOS is a full environment providing all features of platforms like GNOME or KDE.
