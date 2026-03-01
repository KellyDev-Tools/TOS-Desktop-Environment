# TOS Alpha-2 Ecosystem & Modules Specification

**Purpose:** This document defines the plugin architecture, module types, sandboxing rules, and marketplace systems for the Tactical Operating System (TOS). It is spun off from the Core Architecture Specification to maintain focus.

For core system execution rules, see the [Architecture Specification](./TOS_alpha-2_Architecture-Specification.md).
For visual layout and themes, see the [Face Specification](./TOS_alpha-2_Display-Face-Specification.md).

---

## 1. Modules & Packages

Modules are platform‑specific plugins (`.so` on Linux, `.apk` or dynamic modules on Android) that extend TOS functionality. 

TOS employs a dual‑tier trust model for modules:
1. **Standard Tier (Sandboxed):** Most modules run in an isolated environment and must declare required permissions in a manifest (`module.toml`). 
2. **System Tier (Trusted):** Shell Modules and native Sector Types are trusted by the user and run without TOS‑enforced sandboxing to ensure full local system access.

### 1.1 Application Model
Customizes an application’s integration at Level 3. Manifest includes: name, version, type = "app-model", icon, permissions, capabilities (bezel actions, searchable content, etc.).

### 1.2 Sector Type
Defines a sector’s default behaviour: command favourites, interesting directories, environment, available hub modes, default guest role, associated Application Models.

### 1.3 AI Backend Modules
Package type `.tos-ai`. Manifest includes: capabilities (chat, function_calling, vision, streaming), connection details (protocol, endpoint, auth_type), permissions, configuration options (model, temperature, etc.).

Example:
```toml
name = "OpenAI GPT-4"
version = "1.0.0"
type = "ai-backend"
capabilities.chat = true
connection.default_endpoint = "https://api.openai.com/v1/chat/completions"
auth_type = "api-key"
permissions.network = ["api.openai.com"]
``` 

### 1.4 Module Isolation & Permissions
- Modules run in sandbox with limited access (network filtered, filesystem restricted).
- Permissions are displayed to user during installation; user grants/denies.
- Dangerous capabilities (e.g., local file access) require explicit consent.

### 1.5 Terminal Output Modules
Terminal Output Modules are a new class of installable components that define how terminal output is visually presented within Command Hubs and the System Output Area at Level 1. They are packaged as `.tos-terminal` files.

#### 1.5.1 Module Interface
A Terminal Output Module must implement a well‑defined interface (Rust trait or FFI) that allows the Face to:
- Initialize a new instance for a given **context** (sector terminal or system output). The context determines whether the instance is interactive (accepts input and emits interaction events) or read‑only (for system background).
- Receive a stream of lines, each with metadata:
  - Text content (UTF‑8, receives raw output including ANSI codes; the module is responsible for rendering or stripping them as needed).
  - Timestamp.
  - Exit status of the command that produced the line (if applicable).
  - Whether the line is part of a command echo or output.
  - Priority/importance level (for highlighting).
- **Render the output:** Modules render to a platform-appropriate surface provided by the Face.
  - **Web Profile:** Render into a sandboxed DOM element.
  - **Native Profile:** Render into a shared-memory buffer (e.g., DMABUF on Linux or EGLImage on Android) which the Face then composites into the GPU pipeline. Direct raw GPU access is prohibited for sandboxed modules.
- Handle user interactions (only if context is interactive):
  - Click/tap on a line (with coordinates) → return line index and optional context actions.
  - Scroll requests (delta, to top/bottom).
  - Hover events (for tooltips).
- Provide configuration options (exposed via the Settings Daemon).

The Face is responsible for compositing the rendered output with chip regions, bezel, and other overlays.

#### 1.5.2 Built‑in and Optional Modules
TOS includes a default **Rectangular Terminal Module** and supports advanced variants like the **Cinematic Triangular Terminal**. The visual configuration, animation policies, and pinwheel layouts for these modules are defined in the [Face Specification §5.1](./TOS_alpha-2_Display-Face-Specification.md).

#### 1.5.3 Installation and Switching
- Users browse the Marketplace for Terminal Output Modules.
- After installation, the module appears in the Settings panel under "Appearance → Terminal Output".
- Users can select the active module globally, or per‑sector (if the module supports it).
- Switching modules takes effect immediately in all open Command Hubs (existing terminal history is re‑rendered by the new module).

### 1.6 Theme Modules
Theme Modules define the visual appearance of TOS across all levels. They are packaged as `.tos-theme` files. For the complete scope of what a Theme Module controls (Color palette, Typography, Iconography, Audio integration), see the [Face Specification §6.1](./TOS_alpha-2_Display-Face-Specification.md).

**Manifest example (`module.toml`):**
```toml
name = "Star Trek: TNG"
version = "1.0.0"
type = "theme"
description = "Classic LCARS color scheme from The Next Generation"
author = "TOS Community"
icon = "tng.png"

[assets]
css = "theme.css"               # Main stylesheet
fonts = ["lcars.ttf"]            # Optional custom fonts
icons = "icons/"                 # Directory with SVG icons

[capabilities]
supports_high_contrast = true    # Theme can adapt to high‑contrast mode
supports_reduced_motion = true   # Respects reduced‑motion setting
```

Interface: The Face applies the theme by loading the manifest and assets. The CSS file defines CSS custom properties (variables) that are injected into the UI's root. Icons are referenced by name and loaded from the module's asset directory. The theme may also provide JavaScript hooks for dynamic theming (e.g., animated transitions).

Permissions: Typically none, as themes are static assets. If a theme includes custom fonts or icons, they are bundled and do not require additional permissions. However, if a theme wishes to access external resources (e.g., web fonts), it must declare network permissions.

Installation and switching:
· User browses the Marketplace for Theme Modules.
· After installation, the theme appears in Settings → Appearance → Theme.
· Users can select the active theme globally; per‑sector theme overrides are possible if the theme supports it (via the sector's settings).
· Switching themes takes effect immediately (UI reloads with new styles).

Built‑in themes: TOS ships with at least two default themes: a light and a dark variant of the LCARS design, plus a high‑contrast accessibility theme. 

### 1.7 Shell Modules
Shell Modules provide different shell implementations that can be used within Command Hubs. They are packaged as `.tos-shell` files and include:
- The shell executable (or a wrapper script) that TOS will spawn for each sector's PTY.
- Integration scripts to enable OSC communication.
- Default configuration files (e.g., `.bashrc`, `.zshrc`, `config.fish`) that set up aliases, prompt, and environment variables.
- Metadata describing the shell's capabilities (e.g., supports directory notifications, command result capture, etc.).

**Manifest example (`module.toml`):**
```toml
name = "Zsh"
version = "5.9"
type = "shell"
description = "Z shell with powerline support"
icon = "zsh.png"

[executable]
path = "bin/zsh" # Relative path within module
args = ["--login"] # Default arguments

[integration]
osc_directory = true # Supports OSC 1337;CurrentDir
osc_command_result = true # Supports OSC 9002 (with base64)
osc_suggestions = false # (future) Supports command suggestions

[configuration]
default_env = { LANG = "en_US.UTF-8" }
rc_file = "etc/zshrc" # Default rc file to source
```

Interface: The Brain, when creating a new sector's Command Hub, reads the selected shell module, spawns the executable with the given arguments, and attaches the PTY. The shell's output is fed to the Terminal Output Module, and input from the prompt is written to the PTY. The Brain also listens for OSC sequences emitted by the shell and updates state accordingly (e.g., directory changes, command results).

Permissions: Shell modules run as user processes with the same privileges as any shell. They are not sandboxed by TOS (the user's shell is trusted). However, if a shell module includes additional binaries or scripts, they inherit the user's permissions. The module may declare permissions for documentation purposes only.

Installation and switching:
· User installs shell modules from the Marketplace.
· The default shell can be set in Settings → System → Default Shell.
· Per‑sector shell selection is possible via Sector Overrides (if the sector type allows or user overrides).
· Switching shells for an existing sector requires a sector reset (or creating a new hub).

Built‑in shell: TOS includes a reference shell module (Fish) with full OSC integration. Additional modules (Bash, Zsh) are available via the Marketplace, with community‑maintained integration scripts.

### 1.8 Bezel Component Modules
Bezel Components are modular UI elements that can be installed via the marketplace and docked into any available **Tactical Bezel Slot** (Top, Left, or Right). Note that the Bottom Bezel (Prompt Segment) is a static assembly and does not support component docking. They utilize the **Slot Projection** mechanism to expand their presence into the viewport when triggered.

For a complete list of core system components (e.g., Tactical Mini-Map, Resource Telemetry, Brain Connection Status) and their default slot assignments, refer to the [Face Specification §4](./TOS_alpha-2_Display-Face-Specification.md).

### 1.9 Relationship with Other Modules
- **Sector Types** may specify a preferred shell (e.g., a development sector might default to Zsh).
- **Application Models** are shell‑agnostic; they interact with the Brain, not directly with the shell.
- **Terminal Output Modules** render the shell's output, regardless of which shell is used.
- **Theme Modules** affect the appearance of all UI elements, including terminal output.
- **AI Backend Modules** can be invoked from the command line (via the AI mode) and their responses appear in the terminal output.

All modules coexist within the modular service architecture, communicating with the Brain via IPC. The Brain coordinates the instantiation and lifecycle of each module type, ensuring that permissions are enforced and that modules are properly sandboxed.

---

## 2. Sector Templates and Marketplace

### 2.1 Package Types & Manifests
- **Sector Template** (`.tos-template`): Configuration only.
- **Sector Type** (`.tos-sector`): Module with code.
- **Application Model** (`.tos-appmodel`): Module.
- **AI Backend** (`.tos-ai`): Module.
- **Terminal Output Module** (`.tos-terminal`): Module.
- **Theme Module** (`.tos-theme`): Assets and CSS.
- **Shell Module** (`.tos-shell`): Executable and scripts.
- **Audio Theme** (`.tos-audio`): Sounds.

Manifest (`module.toml`) includes name, version, type, icon, permissions, dependencies, configuration schema.

### 2.2 Installation Flow & Permissions
1. Discovery (Search, Marketplace, direct file open).
2. Details panel with description, permissions, dependencies.
3. Permission review (user grants/denies; optional session‑only grant).
4. Dependency resolution.
5. Installation (files copied to `~/.local/share/tos/` or equivalent).
6. Post‑install notification; immediate availability.

### 2.3 Discovery (Search, AI, Updates)
- Search Mode includes packages as a domain.
- AI‑assisted discovery (“I need a Git integration”).
- Update alerts (Yellow Alert) for installed modules; update details show permission changes.

### 2.4 Creating & Sharing Packages
- Export sector as template.
- Developer tools for packaging modules.
- Submission to repositories (optional signature verification).
