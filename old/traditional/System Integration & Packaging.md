# System Integration & Packaging

For TOS to be adopted as a traditional desktop environment, it must integrate deeply with the existing Linux ecosystem and distribution standards.

---

## 1. Distribution Strategy

TOS should be deliverable in two formats for traditional users:

### A. The "TOS Edition" (Standalone ISO)
*   **Approach**: A custom Linux distribution (based on Arch Linux or Fedora) that ships with TOS as the default and only desktop environment.
*   **Benefit**: Perfectly tuned kernel parameters for the `wgpu` rendering pipeline and pre-configured Fish shell modules.

### B. The DE Package (Modular Install)
*   **Approach**: Available via system package managers (e.g., `pacman -S tos-desktop`).
*   **Benefit**: Allows users to install TOS alongside GNOME or KDE and switch between them at the login screen.

---

## 2. Integrated Configuration Management

TOS moves away from hidden Dotfiles for its core UI, utilizing a unified LCARS-styled settings daemon.

*   **TOS-Settings-Daemon**: A background service that bridges the web-based UI (CSS themes) with system configurations (XKB keyboard layouts, monitor configurations via `wlr-output-management`).
*   **Unified Schema**: All TOS-specific settings are stored in a centralized location (e.g., `~/.config/tos/state.json`) which the compositor and shell modules read to maintain sync.

---

## 3. Desktop Entry Integration

To handle standard Linux apps (Firefox, GIMP, etc.):

*   **XDG Desktop Portal**: TOS implements the `org.freedesktop.portal.Desktop` interface. When an app requests to open a file, TOS opens its **Spatial File Browser (Level N)** instead of a generic file picker.
*   **Launcher Sync**: The Level 2 Launcher automatically parses `/usr/share/applications/` to populate the LCARS button grid, automatically categorizing apps into Sectors based on their metadata (e.g., "Graphics", "Development").

---

## 4. Hardware Awareness

In a traditional installation, TOS must handle hardware events gracefully:

*   **Hotplugging Monitors**: Dynamically creating new **Viewports** when a HDMI/DisplayPort cable is plugged in, and redistributing Sectors according to the saved layout.
*   **Power Management**: Dimming the LCARS panels (via CSS opacity) when the system is idle and handling ACPI events for laptop lid closure.
*   **VRAM Optimization**: On local hardware, TOS utilizes **Vulkan memory heaps** to keep thumbnails for all active Sectors in video memory for instant Level 1 global switching.
