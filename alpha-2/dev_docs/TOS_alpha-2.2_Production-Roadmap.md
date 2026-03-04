# TOS Alpha-2.2 Production-Hardening Roadmap

This roadmap defines the transition from Alpha-2.1 (Experimental/Mocked) to Alpha-2.2 (Production/Stable). It prioritizes replacing base64 placeholders, pattern-based logic, and UI-level intercepts with true system-level integrations.

## 1. High-Fidelity Visual Layer (§6 & §16)
- [ ] **Global Console Implementation:** Update `web_ui/app.js` and `style.css` to implement the "System Output Area" (Level 1 Middle Layer).
    - Render Brain terminal log behind sector tiles.
    - Implement the "Bring Terminal to Front" bezel toggle logic.
- [ ] **Kinetic Zoom Transitions (§2.1):** Implement the z-axis zoom animation between Levels 1 and 2.
    - Animate sector tile borders expanding to become the Tactical Bezel.
    - Apply depth-blur/fade to background layers (Global Map/Brain Console).
- [ ] **Tiered Thumbnailing System:**
    - **Sector Tiles (Level 1):** Render dynamic thumbnails of active hubs/apps within the tile interior.
    - **App Tiles (Level 2 ACT):** Implement 10Hz live thumbnails for running apps.
    - **Inactive Chips (Level 2 ACT):** Fallback to static app icons and names for non-running applications.
    - **Generic Fallback:** Implement a symbolic placeholder for system processes lacking both icons and frame buffers.
- [ ] **Secondary Select Infrastructure (§5.4):**
    - Implement the long-press/right-click trigger for all chip types.
    - Create the "Tactical Context Menu" glassmorphism UI component.
    - Implement backend IPC handlers for [Signal], [Renice], and [Inspect] actions.
- [ ] **Predictive Interaction & Predictive Logic (§10):**
    - Implement **Autocomplete-to-Chip**: Real-time shell/path completion resulting in clickable left/right chips.
    - Implement the **Implicit Correction Trigger**: Hook into shell error states (`127` / `command not found`) to trigger typo-matching chips.
    - Implement the **Heuristic Sector Renaming**: Logic to update sector names based on `Cwd` or `ActiveApp`.
- [ ] **Kinetic Sector Borders:** Implement dynamic CSS border animations for sector tiles.
    - **Solid Green/Red:** For last command exit status.
    - **Sliding Gradient:** For active PTY tasks in the hub.
- [ ] **Wayland Frame Captures:** Replace the `base64` mock thumbnails in `src/brain/sector/mod.rs` with actual frame buffer fetches.
    - Utilize the **DMABUF Native Path** to share sub-surface textures with the UI thread at 10Hz.

## 2. Intelligence & Search Maturity (§18.3 & §19)
- [ ] **Vector Search Engine:** Replace the "token-overlap" algorithm in `src/services/search.rs` with a local vector embedding search (e.g., using `fastembed` or a local vector store).
- [ ] **Production LLM Bridge:** Transition `src/services/ai/mod.rs` from hardcoded string fallbacks to a module-driven system.
    - Support configurable OpenAI/Anthropic/Ollama endpoints.
    - Implement real function-calling for staging shell commands.
- [ ] **OSC-Exclusive Mode Switching:** Deprecate "String Sniffing" in `ipc_handler.rs`.
    - Force all mode transitions (CMD -> DIR, etc.) to be driven by `OSC 7` and `OSC 9004` sequences emitted by the shell, ensuring compatibility with all command aliases.

## 3. Security & Foundation (§17)
- [ ] **Kernel-Level Shell Sandboxing:** Implement optional `bwrap` (Bubblewrap) profiles for the primary sector shell (not just marketplace modules).
    - Replace regex-based intercept list with OS-level namespace restrictions (Read-only `/etc`, isolated `/tmp`).
- [ ] **Tactile Confirmation UI:** Implement the functional "Confirmation Slider" in the UI for dangerous commands.
    - Connect `update_confirmation_progress` IPC to a drag-controlled element in the bezel.

## 4. Native Platform Faces (§15)
To transition from the current **Web-Simulator UI** to production-grade native performance, native "Face" delegates must be completed. 

**PRIORITY 1: Native Linux Face (Wayland Shell):**
- [ ] Replace the `Face` struct's `println!` simulation with the `LinuxRenderer` (Wayland).
- [ ] Implement real **wlr-layer-shell** surface management in `src/platform/linux/`.
- [ ] Native GL/Vulkan composition of Sector tiles and Hub viewports.
- [ ] **Local-First Connectivity:** Attempt connection to local socket (`/tmp/tos.brain.sock`) first; fallback to Remote Client login if not found.

**PRIORITY 2: Native OpenXR Face (Quest/VisionPro):**
- [ ] Populate `src/platform/xr/` with the OpenXR context initialization.
- [ ] Implement **World Space Compositing** for the cylindrical "Cockpit" viewport.
- [ ] 3D spatial positioning of sectoral glass panels.
- [ ] **Local-First Connectivity:** Direct memory sharing if running on local OS; fallback to encrypted stream if remote.

**PRIORITY 3: Native Android Face:**
- [ ] Populate `src/platform/android/` with the NDK-based surface rendering logic.
- [ ] Integration with Android choreographer for 90Hz+ smooth persistence.
- [ ] **Local-First Connectivity:** Background local Brain detection; fallback to remote tactical linking.

## 5. System Infrastructure & Refinement
- [ ] **`tos-protocol` Extraction:** Move all `common/` state and IPC structs into a shared library to stabilize the Face-Brain contract.
- [ ] **Unified Visual Token System:** Create `assets/design_tokens.json` and update both `index.css` and `LinuxRenderer` to consume these values.
- [ ] **Headless Brain Testing:** Build a `test-protocol` suite that validates Brain state deltas without requiring a renderer.
- [ ] **Standalone Heuristic Service:** Extract predictive logic from the Brain into a separate `tos-heuristic` service.
- [ ] **Level 6 (Tactical Reset) Implementation:**
    - Develop the **Wireframe Diagnostic Renderer** (low-poly, high-performance view).
    - Implement the **Global Process Kill-Switch** logic in the Brain (with re-auth & tactile confirmation).
    - Implement **Prompt Interlocking** (locking the prompt during Level 6).
    - Hook the **Auto-Trigger logic** for high-latency or deadlock states.

## Cross-Feature Dependencies
- **Kinetic Sector Borders** relies on the PTY exit code telemetry being broadcast via IPC versioned state.
- **Wayland Frame Captures** requires the Face renderer to support `dmabuf` texture bindings.
- **Vector Search Engine** requires the background indexer to generate and cache embeddings on file changes.
