# TOS Module System - Phase 8 Implementation

The TOS Module System provides hot-loadable Application Models and Sector Types with optional containerization and scripting support.

## Overview

The module system allows extending TOS functionality without recompiling the core system. Modules can be written in:

- **Rust** - Compiled to dynamic libraries (.so/.dll)
- **JavaScript** - Executed via embedded runtime
- **Lua** - Executed via embedded runtime

## Module Types

### Application Model (`app-model`)
Customizes application integration at Level 3 (Application Focus):
- Custom bezel actions
- Decoration policies (Suppress, Overlay, Native)
- Command handlers
- Thumbnail generation
- Custom CSS styling

### Sector Type (`sector-type`)
Defines sector default behavior:
- Command favorites
- Default hub mode
- Environment variables
- Interesting directory patterns
- Associated application models

### Hybrid (`hybrid`)
Combines both Application Model and Sector Type capabilities.

## Module Structure

```
my-module/
├── module.toml          # Module manifest
├── module.json          # Alternative manifest format
├── libmy_module.so     # Rust dylib (for Rust modules)
├── main.js              # JavaScript entry (for JS modules)
└── main.lua             # Lua entry (for Lua modules)
```

## Manifest Format (TOML)

```toml
name = "my-module"
version = "1.0.0"
description = "My custom TOS module"
author = "Your Name"
license = "MIT"
type = "app-model"  # or "sector-type" or "hybrid"
entry = "libmy_module.so"
language = "rust"  # or "javascript", "lua"

# Permissions required by the module
permissions = ["network", "filesystem", "process"]

# Containerization (optional)
[container]
backend = "bubblewrap"  # or "firejail", "docker", "podman", "none"
network = false
read_only_paths = ["/usr/share"]
read_write_paths = ["/tmp"]

# Module-specific configuration
[config]
app_class = "custom.app"
decoration_policy = "overlay"

[[config.bezel_actions]]
id = "custom-action"
label = "Custom"
icon = "⚡"
command = "custom_cmd"
priority = 50

[settings]
custom_setting = true
```

## Installation

Modules are loaded from these directories (in order):

1. `~/.local/share/tos/modules/` - User modules
2. `/usr/share/tos/modules/` - System modules
3. `/usr/local/share/tos/modules/` - Local system modules
4. `./modules/` - Development modules (current directory)

## Hot-Reloading

Enable hot-reloading to automatically reload modules when their files change:

```rust
// In your TOS initialization
state.enable_module_hot_reload()?;
```

Then periodically process file system events:

```rust
state.process_module_fs_events();
```

## API

### Module Management

```rust
// List loaded modules
let modules = state.list_modules();
for (name, status) in modules {
    println!("{}: {:?}", name, status);
}

// Reload a specific module
state.reload_module("my-module")?;
```

### Creating Custom Application Models

```rust
use tos_core::modules::app_model::AppModel;

let mut model = AppModel::default_for_app_class("my.app");
model.add_command_handler("custom_cmd", |cmd, state| {
    // Handle the command
    Some("Command executed".to_string())
});
```

### Creating Custom Sector Types

```rust
use tos_core::modules::sector_type::SectorTypeImpl;

let sector_type = SectorTypeImpl::development_type();
let favorites = sector_type.get_command_favourites();
```

## Example Modules

See `examples/modules/` for complete examples:

- `terminal-enhanced/` - Enhanced terminal with custom actions
- `dev-workspace/` - Development sector with git integration
- `dashboard-widget/` - JavaScript dashboard widget

## Security

### Permissions

Modules must declare required permissions:
- `network` - Network access
- `filesystem` - File system access
- `process` - Process management
- `clipboard` - Clipboard access
- `notifications` - Notification access
- `audio` - Audio access
- `camera` - Camera access
- `microphone` - Microphone access
- `display` - Display access
- `input` - Input device access

### Containerization

Modules can be sandboxed using:
- **Bubblewrap** - Lightweight namespace sandboxing
- **Firejail** - Security sandbox with profiles
- **Docker/Podman** - Container-based isolation

## Scripting

### JavaScript Template

```javascript
const TOS = {
    name: "my-module",
    version: "1.0.0",
    
    onLoad: function(state) {
        console.log("Module loaded");
    },
    
    onUnload: function(state) {
        console.log("Module unloaded");
    },
    
    render: function(level) {
        if (level === "ApplicationFocus") {
            return "<div>Custom overlay</div>";
        }
        return null;
    },
    
    handleCommand: function(cmd) {
        if (cmd === "my-cmd") {
            return "Executed!";
        }
        return null;
    }
};

module.exports = TOS;
```

### Lua Template

```lua
local M = {}

M.name = "my-module"
M.version = "1.0.0"

function M.on_load(state)
    print("Module loaded")
end

function M.on_unload(state)
    print("Module unloaded")
end

function M.render(level)
    if level == "ApplicationFocus" then
        return "<div>Custom overlay</div>"
    end
    return nil
end

function M.handle_command(cmd)
    if cmd == "my-cmd" then
        return "Executed!"
    end
    return nil
end

return M
```

## Testing

Run module system tests:

```bash
cd tos-dream
cargo test modules::
```

## Future Enhancements

- Full JavaScript runtime (quickjs/deno_core)
- Full Lua runtime (mlua)
- Module marketplace integration
- Digital signature verification
- Dependency auto-installation
- Module update notifications
