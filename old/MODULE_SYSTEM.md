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

## Phase 9: Marketplace and Templates

The TOS Marketplace provides package management, repository handling, and digital signature verification for modules and sector templates.

### Package Types

#### Sector Template (`.tos-template`)
Configuration-only export from any sector:
- Sector settings and preferences
- Command hub configurations
- Environment variables
- Directory bookmarks
- Command favorites

#### Sector Type Package (`.tos-sector`)
Module package containing:
- Compiled or scripted sector type implementation
- Manifest with metadata
- Optional containerization configuration

#### Application Model Package (`.tos-appmodel`)
Module package containing:
- Application model implementation
- Custom bezel actions
- Decoration policies
- Command handlers

### Repository System

Repositories are JSON-based indexes served over HTTPS:

```json
{
  "repository": {
    "name": "tos-official",
    "description": "Official TOS Module Repository",
    "base_url": "https://marketplace.tos.dev/packages/",
    "last_updated": "2024-01-15T10:00:00Z",
    "tos_version": "0.1.0"
  },
  "packages": [
    {
      "name": "terminal-enhanced",
      "versions": [
        {
          "version": "1.0.0",
          "download_path": "terminal-enhanced-1.0.0.tos-appmodel",
          "sha256": "abc123...",
          "signature": "sig456...",
          "dependencies": [],
          "size": 10240
        }
      ],
      "package_type": "app-model",
      "description": "Enhanced terminal",
      "author": "TOS Team",
      "license": "MIT",
      "tags": ["terminal", "productivity"]
    }
  ],
  "version": "1.0"
}
```

### Configuration

Configure repositories in `~/.config/tos/marketplace.json`:

```json
{
  "repositories": [
    {
      "name": "official",
      "url": "https://marketplace.tos.dev",
      "enabled": true,
      "priority": 1
    }
  ],
  "verify_signatures": true,
  "auto_install_dependencies": true,
  "trusted_keys": []
}
```

### API Usage

#### Search Packages

```rust
use tos_core::marketplace::{Marketplace, InstallRequest};

let marketplace = Marketplace::new();
let results = marketplace.search("terminal").await?;
```

#### Install Package

```rust
let request = InstallRequest {
    package_name: "terminal-enhanced".to_string(),
    version_constraint: "1.0.0".to_string(),
    repository: None, // Search all repos
    auto_accept: false,
    skip_signature_check: false,
};

let result = marketplace.install(request).await?;
println!("Installed to: {}", result.install_path.display());
```

#### Export Sector Template

```rust
use tos_core::marketplace::{ExportRequest, TemplateHandler};

let handler = TemplateHandler::new();
let request = ExportRequest {
    sector_id: "my-sector".to_string(),
    name: "dev-workspace".to_string(),
    version: "1.0.0".to_string(),
    output_path: PathBuf::from("dev-workspace.tos-template"),
    description: "Development workspace".to_string(),
    author: "Developer".to_string(),
    license: "MIT".to_string(),
    include_state: false,
    tags: vec!["dev".to_string()],
};

let result = handler.export_sector(request)?;
println!("Exported: {} ({} bytes)", result.template_path.display(), result.size);
```

#### Import Template

```rust
let template = handler.import_template(Path::new("dev-workspace.tos-template"))?;
let sector_dir = handler.apply_template(&template, "new-sector")?;
```

### Security

#### Signature Verification

Packages can be signed with Minisign (Ed25519):

```rust
use tos_core::marketplace::SignatureVerifier;

let mut verifier = SignatureVerifier::new(vec![]);
verifier.add_trusted_key("RWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBYm0+Ng0h0i0".to_string());
verifier.save_trusted_key("official", "RWQf6L...")?;
```

#### Trust on First Use (TOFU)

```rust
use tos_core::marketplace::TofuKeyManager;

let mut tofu = TofuKeyManager::new();
if tofu.record_key("key_id", "public_key") {
    println!("New key recorded");
}
```

#### Checksum Verification

All packages include SHA256 checksums verified during download:

```rust
let package_manager = PackageManager::new(cache_dir);
let path = package_manager.download(&metadata).await?; // Verifies checksum
```

### Dependency Resolution

Dependencies are automatically resolved and installed:

```rust
use tos_core::marketplace::DependencyResolver;

let resolver = DependencyResolver::new();
let deps = resolver.resolve(&package, &repository_manager).await?;
```

Dependency format in manifests:
```toml
dependencies = ["base-lib@1.0.0", "utils@>=2.0.0", "optional-lib@^3.0"]
```

### Lock Files

Generate lock files for reproducible installations:

```rust
use tos_core::marketplace::LockFile;

let lockfile = LockFile::new(packages);
lockfile.save(Path::new("tos.lock"))?;
```

### CLI Commands (Future)

```bash
# Search marketplace
tos marketplace search <query>

# Install package
tos marketplace install <package>[@<version>]

# Export sector as template
tos sector export <sector-name> --name <template-name>

# List installed packages
tos marketplace list

# Update packages
tos marketplace update
```

## Future Enhancements

- Full JavaScript runtime (quickjs/deno_core)
- Full Lua runtime (mlua)
- WebAssembly module support
- Module sandboxing improvements
- Peer-to-peer package sharing
- Automated security scanning
