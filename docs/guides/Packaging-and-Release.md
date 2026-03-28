# TOS Beta-0 Packaging & Release Guide

This guide outlines how to build distributions, orchestrate releases, and push artifacts for the Tactical Operating System (TOS) Beta-0 environment.

## Overview

The Beta-0 distribution is composed of two primary ecosystems:
1. **The Brain & Daemons** (Native Rust Services)
2. **The Terminal UI / Visual Face** (SvelteKit Web Subsystem)

Because TOS is fundamentally distributed (with dynamic IPC port assignments via `brain.sock`), packaging must successfully capture **both** layers to function. We've standardized this workflow across four primary package managers (Arch/AUR, Fedora/RPM, Ubuntu/DEB, and macOS/Homebrew), as well as a standalone Tarball Release script.

---

## 1. Automated Tarball Releases

For generating raw, pre-compiled portable archives, we use the integrated release automation script.

make release
# OR for all formats (Debian, Arch, Android):
make release-all
```

**What it does:**
1. Installs Node dependencies (`npm i`) and compiles the static Svelte Web Framework.
2. Triggers a `cargo build --release` across all binaries in the workspace.
3. Bundles the core daemons and the **Native Wayland Face** (`tos-wayland-face`).
4. Includes the **Session Orchestrator** (`tos-session`) and desktop metadata.
5. Generates a compressed tarball: `targets/tos-beta-0-0.1.0-beta.0-linux-x86_64.tar.gz`.

---

## 2. Package Managers Implementation Guide

Pre-configured package definitions exist within `packaging/`.

### Arch Linux / AUR (`PKGBUILD`)
The Arch package pulls directly from the source code, invoking the `Makefile` natively.
```bash
cd packaging/arch
makepkg -si
```
* **Artifacts Installed:** `/usr/bin/tos`, `/usr/bin/tos-brain`, `/usr/bin/tos-wayland-face`, `/usr/bin/tos-session`, `/usr/share/wayland-sessions/tos.desktop`

### Fedora / RHEL (`.rpm`)
We provide a standard `tos.spec` file inside `packaging/rpm`.
```bash
rpmbuild -ba packaging/rpm/tos.spec
```
* Note: Ensure `rpm-build` and `rust`/`cargo` are installed locally to trigger the compile chain.

### Debian / Ubuntu (`.deb`)
The `.deb` file utilizes the standard Debhelper configurations.
```bash
dpkg-buildpackage -us -uc
```
* Rules inside `packaging/deb/rules` will automatically hijack the `dh_auto_build` pipeline to invoke `make build-services`.

### macOS / Homebrew (`tos.rb`)
Homebrew handles both `npm` and `cargo` installations seamlessly via nested CD blocks in the formula.
```bash
brew install --build-from-source packaging/homebrew/tos.rb
```

---

## 3. Local Installation Scripts (`install.sh`)

If you are developing or compiling Beta-0 manually and wish to register it system-wide without a package manager, use the provided install script.

```bash
sudo ./packaging/install.sh
```

**Actions Performed:**
- Invokes `make build-services`.
- Provisions explicit system directories (`/etc/tos`, `/var/log/tos`).
- Copies **all 7 background service daemons** and the **Native Face** directly into your configured `$BINDIR`.
- Scaffolds the Wayland/GDM integration with `tos-session` and `tos.desktop`.

> **Note on Permissions:** `install.sh` assigns `777` permissions to `/var/log/tos` to allow unprivileged subsystem daemons (like `loggerd`) to stream trace logs without forcing `sudo` execution.
