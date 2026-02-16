## Linux System Installation Plan

### 1. Display Manager Integration

After compilation, TOS needs to register itself with the system's display manager to appear as a selectable session at login. The approach is universal across display managers.

#### 1.1 The .desktop File Standard

All modern display managers (GDM, SDDM, LightDM, KDM) discover available sessions through `.desktop` files placed in standard directories. The primary location is:

```
/usr/share/xsessions/tos.desktop
```

For Wayland sessions (if TOS supports Wayland directly), also create:

```
/usr/share/wayland-sessions/tos.desktop
```

Contents of tos.desktop:

```ini
[Desktop Entry]
Name=TOS
Comment=Tactical Operating System
Exec=/usr/bin/tos-session
TryExec=/usr/bin/tos-session
Icon=/usr/share/pixmaps/tos-icon.png
Type=Application
DesktopNames=TOS
```

Explanation of fields :

· Exec: The actual command to start TOS
· TryExec: Path to the executable (used for validation)
· DesktopNames: Used by some DMs for categorization
· The file must be readable by all users


### 2. Post-Installation Configuration

#### 2.1 Configuration Files Location

Create standard directories:

```

/etc/tos/           # System-wide configuration
/var/log/tos/       # Log files (TOS Log)
/usr/share/tos/     # Shared data, modules, sector templates
/usr/lib/tos/       # Binaries and libraries

```

#### 2.2 Desktop Entry for Application Menu

Create a desktop entry so TOS can also be launched from the application menu (not just as a session):

```

/usr/share/applications/tos.desktop

```

```ini
[Desktop Entry]
Name=TOS
Comment=Tactical Operating System
Exec=/usr/bin/tos
Icon=tos-icon
Terminal=false
Type=Application
Categories=System;
```

### 3. Packaging for Distribution

TOS should be packaged for multiple distribution formats to reach the widest audience.

#### 3.1 Packaging Strategy

Maintain separate package source directories for each format:

```

tos/
├── packaging/
│   ├── deb/           # Debian/Ubuntu packaging files
│   │   ├── control
│   │   ├── changelog
│   │   ├── copyright
│   │   └── rules
│   ├── rpm/           # Fedora/RHEL/CentOS/SUSE
│   │   └── tos.spec
│   └── arch/          # Arch Linux
│       └── PKGBUILD
└── ... (source code)

```

#### 3.2 Debian/Ubuntu Packages (.deb)

Create a `debian/` directory with these essential files:

**debian/control:**
```

Source: tos
Section: x11
Priority: optional
Maintainer: Your Name <email@example.com>
Build-Depends: debhelper-compat (= 13), rustc, cargo, libwayland-dev, libxkbcommon-dev
Standards-Version: 4.6.0
Homepage: https://tos-project.org

Package: tos
Architecture: any
Depends: ${shlibs:Depends}, ${misc:Depends}, libwayland-client0, libxkbcommon0
Description: Tactical Operating System
TOS is a reimagining of the Linux desktop with a recursive zoom hierarchy
and command-first philosophy. It provides a spatial command platform
for productivity.

```

**debian/rules** (executable):
```make
#!/usr/bin/make -f
%:
    dh $@

override_dh_auto_build:
    cargo build --release --target=release

override_dh_auto_install:
    mkdir -p debian/tos/usr/bin
    cp target/release/tos-wayland debian/tos/usr/bin/tos-session
    # Install desktop files
    mkdir -p debian/tos/usr/share/xsessions
    cp packaging/tos.desktop debian/tos/usr/share/xsessions/
```

Build with:

```bash
dpkg-buildpackage -us -uc
```

### 3.3 RPM Packages (.spec)

**tos.spec:**
```spec
Name: tos
Version: 1.0
Release: 1%{?dist}
Summary: Tactical Operating System
License: GPLv3+
URL: https://tos-project.org
Source0: %{name}-%{version}.tar.gz

BuildRequires: rust, cargo, libwayland-devel, libxkbcommon-devel
Requires: wayland, libxkbcommon

%description
TOS is a reimagining of the Linux desktop with a recursive zoom hierarchy
and command-first philosophy.

%build
cargo build --release

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 target/release/tos-wayland %{buildroot}%{_bindir}/tos-session
mkdir -p %{buildroot}%{_datadir}/xsessions
install -m 644 packaging/tos.desktop %{buildroot}%{_datadir}/xsessions/

%files
%{_bindir}/tos-session
%{_datadir}/xsessions/tos.desktop
```

Build with:

```bash
rpmbuild -ba tos.spec
```

### 3.4 Arch Linux PKGBUILD

**PKGBUILD:**
```bash
pkgname=tos
pkgver=1.0
pkgrel=1
pkgdesc="Tactical Operating System"
arch=('x86_64')
url="https://tos-project.org"
license=('GPL3')
depends=('wayland' 'libxkbcommon')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$srcdir/$pkgname-$pkgver"
    cargo build --release
}

package() {
    cd "$srcdir/$pkgname-$pkgver"
    install -Dm755 target/release/tos-wayland "$pkgdir/usr/bin/tos-session"
    install -Dm644 packaging/tos.desktop "$pkgdir/usr/share/xsessions/tos.desktop"
}

### 3.5 Universal Install Script

For users who prefer to compile from source, provide an installation script:

```bash
#!/bin/bash
# install.sh - Install TOS after compilation

set -e

PREFIX="${PREFIX:-/usr/local}"
BINDIR="${PREFIX}/bin"
DATADIR="${PREFIX}/share"
XSESSIONSDIR="${DATADIR}/xsessions"

# Build (if not already built)
cargo build --release

# Create directories
sudo mkdir -p "$BINDIR" "$XSESSIONSDIR" "/etc/tos" "/var/log/tos"

# Install binary
sudo cp target/release/tos-wayland "$BINDIR/tos-session"

# Install session desktop file
sudo cp packaging/tos.desktop "$XSESSIONSDIR/"

# Install configuration (if any)
[ -f tos.conf ] && sudo cp tos.conf /etc/tos/

# Set permissions
sudo chmod 755 "$BINDIR/tos-session"
sudo chmod 644 "$XSESSIONSDIR/tos.desktop"

# Detect and restart display manager
if systemctl is-active display-manager.service >/dev/null 2>&1; then
    echo "Restarting display manager..."
    sudo systemctl try-restart display-manager.service
fi

echo "TOS installed successfully. Select 'TOS' from your login screen session menu."

### 3.6 Distribution-Specific Considerations

| Distribution Family | Package Format | Key Tools  |
|-------------------|----------------|-----------|
| **Debian/Ubuntu** | `.deb` | `dpkg-buildpackage`, `debhelper` |
| **Fedora/RHEL** | `.rpm` | `rpmbuild`, `mock` for isolated builds |
| **CentOS/Rocky** | `.rpm` | Same as Fedora, adjust dependencies |
| **openSUSE** | `.rpm` | Use `rpmbuild` with SUSE-specific macros |
| **Arch/Manjaro** | `.pkg.tar.zst` | `makepkg`, `PKGBUILD` |
| **NixOS** | `.nix` | Create Nix expression |

### 4. Build Automation

Consider using a packaging automation tool like **apkg** to manage multiple distribution formats:

```bash
# apkg configuration example
apkg get-package-source deb   # Generate Debian source package
apkg build rpm                # Build RPM package
apkg build arch               # Build Arch package
```

Alternatively, cargo-deb for Rust-specific Debian packaging:

```bash
cargo install cargo-deb
cargo deb
```

### 5. Installation Checklist

- [ ] Create `.desktop` file in `/usr/share/xsessions/`
- [ ] Create session wrapper script in `/usr/bin/`
- [ ] Set correct permissions (755 for executables, 644 for desktop files)
- [ ] Detect active display manager and restart if needed
- [ ] Create configuration directories (`/etc/tos/`, `/var/log/tos/`)
- [ ] Package for all target distributions (deb, rpm, PKGBUILD)
- [ ] Provide universal install script for source builds
- [ ] Document manual steps for XDM fallback

This plan ensures TOS integrates seamlessly with any Linux distribution's display manager and can be packaged and distributed through official channels.
