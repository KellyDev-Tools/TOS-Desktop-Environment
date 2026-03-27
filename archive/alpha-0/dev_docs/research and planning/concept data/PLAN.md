 # Project Plan: LCARS Spatial Desktop Environment (SDE)

## 1. Executive Summary
A Linux-based Desktop Environment (DE) merging the **LCARS** aesthetic with an **Infinite Canvas** navigation system. The interface replaces traditional window management with a zoom-based spatial hierarchy, anchored by a persistent command-prompt at the base of the viewport.

---

## 2. Technical Stack
* **Base OS:** Linux (Distribution agnostic; targeted at Arch or Fedora).
* **Display Protocol:** **Wayland** (Required for high-performance compositing and scaling).
* **Compositor Framework:** **Smithay (Rust)** or **wlroots (C)**. These allow for custom rendering of window buffers on a 3D plane.
* **UI Framework:** **Qt 6 / QML**. QML’s declarative nature is ideal for the fluid, curved animations required by the LCARS design language.
* **Primary Shell:** **Nushell**. Its structured data output allows the UI to easily parse and display file/directory info in LCARS grids.

---

## 3. Core Functional Modules

### A. The Persistent Command Bar (The "Footer")
* **Location:** Bottom 10% of the screen.
* **Function:** Active terminal input.
* **Integration:** A "Command Overlay" populated by shell aliases. Tapping an overlay button injects the string directly into the prompt buffer.

### B. The Infinite Canvas (The "Workspace")
* **Navigation:** Pinch-to-zoom and two-finger pan.
* **Scaling:** Windows are textures on a coordinate system $(x, y, z)$. 
* **Transition:** Selecting an app icon triggers a "Focus Zoom," where the $z$-axis value increases until the window bounds match the viewport.

### C. The Help/Metadata Grid
* **Trigger:** Command entry or file selection.
* **Display:** Dynamic LCARS sidebars (elbows) populating with options from `--help` manpages or file
* 