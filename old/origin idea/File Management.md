# File Management Considerations

The file browser is a core component of any desktop. In TOS, file management should feel native to the spatial, zooming metaphor and integrate seamlessly with the command line.

---

# File Management (Level N Context)

The file browser in TOS is not a separate application window, but a native implementation of the **Recursive Zoom Hierarchy**. Navigating the filesystem is functionally identical to navigating the OS structure.

---

## The Spatial File Browser

The filesystem exists as **Level N (Deep Context)**.

### Level 3/N: Directory Navigation

When a user opens a "Folder" or a File Manager app:

1.  **Initial View (Level 3)**: The root or home directory is displayed.
2.  **Zooming In (Level N+1)**: Clicking a directory tile performs a "Zoom In" transition. The directory becomes the new focused context, filling the screen with its children (files/folders).
3.  **Zooming Out (Level N-1)**: A pinch-out or "Back" action transitions to the parent directory.

### Interaction Logic: Execution Mode Toggle

The filesystem view behaves according to a global **Execution Mode** setting, powered by the **TOS Shell API**:

*   **Metadata Injection (OSC Sequences)**: To support the spatial UI, the active **Shell Module** (e.g., the default Fish module) must implement metadata injection. This is typically achieved via function-shadowing (wrapping utilities like `ls`). When executed, the shell appends OSC escape sequences containing JSON metadata (MIME types, thumbnails, permissions), which the compositor intercepts to render visual tiles.
*   **Event Hooks (Real-time Sync)**: Using shell-specific hooks (e.g., `fish_prompt` or `precmd`), the module notifies the compositor of `cwd` changes. This ensures that typing `cd ..` in any supported shell triggers a "Zoom Out" animation.
*   **Execution Modes**: 
    - **Wait-to-Execute (Staged)**: Visual taps pipe the command (e.g., `open image.png`) into the terminal.
    - **Auto-Execute (Direct)**: Tapping a tile runs the action immediately.

---

## Implementation Rules (Shell API)

1.  **Strict Pathing**: The breadcrumb and the `path` stack are synchronized via the Shell API.
2.  **Visual Consistency**: Every directory level uses the same LCARS button grid layout as the Level 2 App Launcher.
3.  **MIME Handling**: The shell provides MIME metadata so the compositor can select the correct Level 3 "Focus" template (e.g., Image Viewer vs. Text Editor).

---

## Integration with Traditional File Managers

For users who prefer a conventional file manager, the system should still support launching Nautilus, Dolphin, or others. These will run in compatibility mode (as described in `App Compatibility.md`) and be wrapped in the LCARS frame.
