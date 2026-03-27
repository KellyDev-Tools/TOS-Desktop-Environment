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

### Running a Release Compilation
```bash
cd beta-0
./scripts/release.sh
```

**What it does:**
1. Installs Node dependencies (`npm i`) and compiles the static Svelte Web Framework.
2. Triggers a `cargo build --release` across all binaries in the workspace.
3. Groups the core daemons (`tos`, `tos-brain`, `marketplaced`, `settingsd`, `sessiond`, `loggerd`, `searchd`, `heuristicd`, `priorityd`) alongside configuration (`tos.toml` and `.desktop` files) into a monolithic structure.
4. Generates a compressed tarball: `targets/tos-beta-0-0.1.0-beta.0-linux-x86_64.tar.gz`.

---

## 2. Package Managers Implementation Guide

Pre-configured package definitions exist within `beta-0/packaging/`.

### Arch Linux / AUR (`PKGBUILD`)
The Arch package pulls directly from the source code, invoking the `Makefile` natively.
```bash
cd beta-0/packaging/arch
makepkg -si
```
* **Artifacts Installed:** `/usr/bin/tos`, `/usr/bin/tos-brain`, `/usr/share/xsessions/tos.desktop`

### Fedora / RHEL (`.rpm`)
We provide a standard `tos.spec` file inside `packaging/rpm`.
```bash
rpmbuild -ba beta-0/packaging/rpm/tos.spec
```
* Note: Ensure `rpm-build` and `rust`/`cargo` are installed locally to trigger the compile chain.

### Debian / Ubuntu (`.deb`)
The `.deb` file utilizes the standard Debhelper configurations.
```bash
cd beta-0
dpkg-buildpackage -us -uc
```
* Rules inside `packaging/deb/rules` will automatically hijack the `dh_auto_build` pipeline to invoke `make build-services` within Beta-0.

### macOS / Homebrew (`tos.rb`)
Homebrew handles both `npm` and `cargo` installations seamlessly via nested CD blocks in the formula.
```bash
brew install --build-from-source beta-0/packaging/homebrew/tos.rb
```

---

## 3. Local Installation Scripts (`install.sh`)

If you are developing or compiling Beta-0 manually and wish to register it system-wide without a package manager, use the provided install script.

```bash
cd beta-0
sudo ./packaging/install.sh
```

**Actions Performed:**
- Invokes `make build-services`.
- Provisions explicit system directories (`/etc/tos`, `/var/log/tos`).
- Copies **all 7 background service daemons** directly into your configured `$BINDIR`.
- Scaffolds the Wayland/X11 `<display-manager>` integration with `tos.desktop`.

> **Note on Permissions:** `install.sh` assigns `777` permissions to `/var/log/tos` to allow unprivileged subsystem daemons (like `loggerd`) to stream trace logs without forcing `sudo` execution.
