# TOS

**Terminal On Steroids** — an ambient terminal layer for the Linux desktop
you already have.

> **Direction update — 2026-11-15.** TOS is being refocused. It is no longer
> being built as a standalone desktop environment with its own session and
> compositor. TOS is now an **ambient layer inside a host desktop**
> (GNOME, KDE, Hyprland, Sway, COSMIC): the terminal session as an animated
> background, a glanceable chips overlay, and a prompt overlay that zooms
> or slides in at a moment's notice. The host desktop keeps window and
> application management. See the
> [Scope Split](./docs/spec/TOS_Scope-Split.md) for the component-by-component
> assessment and the [Roadmap](./docs/spec/TOS_Roadmap.md) for the plan.

## Architecture Highlights
- **Headless Brain Daemon**: `tos-brain` holds sessions, sectors, and state.
  It can run on the local machine or on a server — remote is the default
  topology, reached over a mesh VPN (Tailscale/WireGuard).
- **Consolidated Common Core**: The `tos-common` crate defines the unified
  contract between Brain and Face — IPC, sectors, shell/PTY, and the
  platform abstraction (including the remote and Quest/spatial slots).
- **Ambient Face**: One thin face per machine with three surfaces — the
  terminal as animated background, a chips overlay, and the prompt overlay
  (Command Hub). Built on Wayland layer-shell where available; GNOME uses
  a shim, since Mutter does not offer layer-shell to clients.
- **App Placement Routing**: Where an app runs (host vs server) and where
  it displays (host screen vs XR) are independent choices, routed by the
  Brain; server-run GUI apps reach the host via waypipe.
- **Recursive Zoom Hierarchy**: Navigate between system-wide contexts using
  vertical spatial zooming — the background is the zoomed-out state, the
  prompt is zooming in.

## Getting Started
### Build Requirements
- Rust stable
- Node.js 20+ (for the Svelte Face)

### Running the Brain
To start the core Brain in a standalone terminal:
```bash
# Start brain only
./target/debug/tos-brain

# Start brain and automatically spawn all auxiliary services (Orchestration)
./target/debug/tos-brain --orchestrate
```

### Running the Face (Svelte-based Web Interface)
```bash
make run-web
```

### Legacy: standalone session install (parked)
The earlier full-system installation — selecting TOS as a GDM/SDDM session —
is parked under the new direction and is no longer the primary run model:
```bash
make build-all
make install # will ask for sudo password
make restart-gdm
```

## Documentation
- [Scope Split (2026-11-15)](./docs/spec/TOS_Scope-Split.md) — Keep / Delegate / Park / Drop for every component
- [Roadmap](./docs/spec/TOS_Roadmap.md) — the phased plan from the current state to the ambient layer
- [Architecture Specification](./docs/spec/TOS_v0.1.0_Architecture.md) — see its Direction Update for the target topology
- [Developer Reference](./docs/spec/TOS_v0.1.0_Developer.md)
- [Feature Specifications](./docs/spec/TOS_v0.1.0_Features.md)
- [Ecosystem Specification](./docs/spec/TOS_v0.1.0_Ecosystem.md)
- [Visual Design Specification](./docs/spec/TOS_v0.1.0_Visual_Design.md)
- [User Manual](./docs/spec/TOS_v0.1.0_User-Manual.md)
- [Packaging & Release Guide](./docs/guides/Packaging-and-Release.md)

## Status
The project is in **Phase 0 — Clean the Board** of the
[Roadmap](./docs/spec/TOS_Roadmap.md): merge the scope split, remove the
archived prototype generations and stale branches, drop the Wayland face,
add CI, and make the E2E suite runnable — then build the frozen first
slice (Brain + one layer-shell ambient face, local).

Verified foundation (2026-11-15): the full Rust workspace compiles clean;
`tos-common` tests pass 97/97 and all other suites pass except the Wayland
face's test code, which no longer compiles against the current core API.
