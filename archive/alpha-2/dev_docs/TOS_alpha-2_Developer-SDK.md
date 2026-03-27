# TOS Alpha-2 Developer SDK

**Version:** 1.0 (Alpha-2)  
**Purpose:** This document provides the official guidelines, schemas, and interface contracts for developing third-party extensions for **TOS** (**Terminal On Steroids**).

---

## 1. Introduction

The TOS Ecosystem is built on a "Local First" philosophy. Every extension—from theme to artificial intelligence—runs in its own localized process space and communicates with the Brain over structured IPC or the "Module Contract" protocol.

### 1.1 Scope
This SDK supports the following module types:
- **Themes (`.tos-theme`)**: CSS layouts, icons, and typography.
- **AI Backends (`.tos-ai`)**: LLM adapters using the JSON Boundary Protocol.
- **AI Behaviors (`.tos-aibehavior`)**: Pluggable co-pilot interaction patterns.
- **Shells (`.tos-shell`)**: PTY environments with OSC telemetry (Fish, Zsh, Bash).
- **Terminal Output (`.tos-terminal`)**: Custom rendering logic for terminal canvasses.
- **Application Models (`.tos-appmodel`)**: Metadata for deep Level 3 integration.
- **Bezel Components (`.tos-bezel`)**: Dockable bezel slot components.

---

## 2. Package Anatomy

Every TOS module must adhere to the following directory structure:

```text
package-id/
├── module.toml         # Mandatory Manifest (§3)
├── signature.sig       # Cryptographic signature (for verified distribution)
├── bin/                # Executables (AI engines, shell wrappers)
├── assets/             # Themes, CSS, Icons, Fonts
└── etc/                # Default configuration files
```

---

## 3. The Manifest (`module.toml`)

The `module.toml` is the source of truth for the marketplace and the Brain.

### 3.1 Common Fields
```toml
id = "com.community.tactical-amber"
name = "Tactical Amber"
version = "1.2.0"
type = "theme" # Options: appmodel, terminal, theme, shell, ai, aibehavior, bezel, audio
author = "Sovereign Engineering"
description = "High-contrast tactical theme inspired by deep-space sensors."
icon = "assets/icon.png"
```

### 3.2 Type-Specific Sections

#### Shell Configuration (§1.7)
```toml
[executable]
path = "bin/zsh"
args = ["--login"]

[integration]
osc_directory = true     # Supports OSC 7 or 1337
osc_command_result = true # Supports OSC 9002
```

#### Theme Configuration (§1.6)
```toml
[assets]
css = "assets/theme.css"
icons = "assets/icons/"
fonts = ["assets/fonts/Inter.ttf"]
```

#### AI Capabilities (§1.3)
```toml
[capabilities]
chat             = true
streaming        = true
function_calling = true
vision           = false
latency_profile  = "fast_remote"   # local | fast_remote | slow_remote
```

---

## 4. Module Contracts

### 4.1 The AI Boundary (JSON over Stdin/Stdout)
AI modules are executed as child processes. The Brain communicates via a strict JSON protocol over standard I/O.

**Input (Stdin):**
```json
{
  "prompt": "list all files",
  "context": ["sector:Primary", "path:/home/user"],
  "stream": false
}
```

**Output (Stdout):**
```json
{
  "id": "uuid-123",
  "choice": {
    "role": "assistant", 
    "content": "{\"command\": \"ls -la\", \"explanation\": \"Listing files in long format.\"}"
  },
  "status": "complete"
}
```

### 4.2 Shell Telemetry (OSC Sequences)
Shells must emit standard OSC (Operating System Command) sequences to synchronize with the desktop environment.

- **OSC 7 (Current Directory):** `\x1b]7;file://hostname/path\x07`
- **OSC 1337 (Current Directory):** `\x1b]1337;CurrentDir=/path\x07`
- **OSC 9002 (Command Result):** `\x1b]9002;<command>;<status>;<base64_output>\x07`

---

## 5. Development Workflow

### 5.1 Discovery & Verification
Use the `tos-pkg` utility to test your module locally before submission.

```bash
# Verify the manifest structure
tos-pkg verify ./my-theme-module

# Dry-run loading the module in a mock brain
tos-pkg load ./my-theme-module
```

### 5.2 Signing
Modules distributed via the Marketplace must be signed.

```bash
# Generate a new developer key pair
tos-pkg gen-key --output ./dev-key.pem

# Sign your module
tos-pkg sign --key ./dev-key.pem --module ./dist/my-module.tos-theme
```

---

## 6. Resources & Templates

- **Rust AI Adapter**: [src/brain/module_manager.rs](https://github.com/KellyDev-Tools/TOS-Desktop-Environment/blob/main/alpha-2/src/brain/module_manager.rs)
- **CSS Theme Baseline**: [svelte_ui/src/app.css](https://github.com/KellyDev-Tools/TOS-Desktop-Environment/blob/main/alpha-2/svelte_ui/src/app.css)
- **OSC Script Examples**: `etc/tos-init.fish`
