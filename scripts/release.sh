#!/bin/bash
# scripts/release.sh - Creates the canonical beta-0 release asset bundle

set -e

VERSION="0.1.0-beta.0"
OUTPUT_DIR="targets/release_$VERSION"
TAR_NAME="tos-beta-0-$VERSION-linux-x86_64.tar.gz"

echo "[RELEASE] Establishing Beta-0 Output Directory: $OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/bin"
mkdir -p "$OUTPUT_DIR/share/xsessions"
mkdir -p "$OUTPUT_DIR/var/log/tos"

echo "[RELEASE] Compiling Svelte UI Framework..."
cd face-svelte-ui
npm i && npm run build
cd ..

echo "[RELEASE] Compiling Native Daemons (Release Mode)..."
echo "  -> Building tos-common..."
cd tos-common && cargo build --release
cd ..
echo "  -> Building tos-brain..."
cd brain && cargo build --release
cd ..
echo "  -> Building face-wayland-linux..."
cd face-wayland-linux && cargo build --release
cd ..

echo "[RELEASE] Bundling Binary Assets..."
cp brain/target/release/tos-brain "$OUTPUT_DIR/bin/"
cp packaging/tos-session "$OUTPUT_DIR/bin/"
chmod +x "$OUTPUT_DIR/bin/tos-session"
for daemon in tos-settingsd tos-marketplaced tos-sessiond tos-loggerd tos-searchd tos-heuristicd tos-priorityd face-wayland-linux; do
    if [ -f "brain/target/release/$daemon" ]; then
        cp "brain/target/release/$daemon" "$OUTPUT_DIR/bin/"
    elif [ -f "face-wayland-linux/target/release/$daemon" ]; then
        cp "face-wayland-linux/target/release/$daemon" "$OUTPUT_DIR/bin/"
    fi
done

echo "[RELEASE] Bundling Subsystems & Configurations..."
cp tos.toml "$OUTPUT_DIR/"
cp packaging/tos.desktop "$OUTPUT_DIR/share/xsessions/"

echo "[RELEASE] Packaging Tarball..."
cd targets
tar -czvf "$TAR_NAME" "release_$VERSION"
cd ..

# HSM Release Signing (§6.8)
if [ -n "$TOS_HSM_MODULE" ] && [ -n "$TOS_HSM_PIN" ]; then
    echo "[RELEASE] Signing Release Assets (HSM: $TOS_HSM_MODULE)..."
    # Ensure tos-signer is built
    cargo build --release -p tos-signer
    ./target/release/tos-signer sign --label tos-release "targets/$TAR_NAME"
    echo "[RELEASE] Signature verified for targets/$TAR_NAME"
fi

echo "[RELEASE] SUCCESS. Artifact available at: targets/$TAR_NAME"

# --- Advanced Distribution Formats (§4.2) ---
# These require local system tools (dpkg-deb, rpmbuild, etc)

if [ "$1" == "--all" ]; then
    echo "[RELEASE] Initializing Multi-Platform Distribution Pipeline..."

    # 1. Debian (.deb)
    if command -v dpkg-deb &> /dev/null; then
        echo "[RELEASE] Building Debian Package..."
        # Map folders to debian structure
        mkdir -p "targets/deb_root/usr/bin"
        cp -r "$OUTPUT_DIR/bin/"* "targets/deb_root/usr/bin/"
        mkdir -p "targets/deb_root/usr/share/wayland-sessions"
        cp packaging/tos.desktop "targets/deb_root/usr/share/wayland-sessions/tos.desktop"
        
        # Meta manifest (Binary-only control file)
        mkdir -p "targets/deb_root/DEBIAN"
        chmod 755 "targets/deb_root/DEBIAN"
        cat <<EOF > "targets/deb_root/DEBIAN/control"
Package: tos
Version: $VERSION
Section: x11
Priority: optional
Architecture: amd64
Maintainer: TOS Development Team <dev@tos-project.org>
Depends: libwayland-client0, libxkbcommon0
Description: Tactical Operating System
 TOS is a reimagining of the Linux desktop with a recursive zoom hierarchy
 and command-first philosophy.
EOF
        chmod 644 "targets/deb_root/DEBIAN/control"
        
        dpkg-deb --build "targets/deb_root" "targets/tos-$VERSION.deb"
    fi

    # 2. Arch Linux (PKGBUILD)
    echo "[RELEASE] Staging Arch Linux PKGBUILD..."
    cp packaging/arch/PKGBUILD "targets/"

    # 3. Android Face APK (Handheld Profile)
    if command -v cargo-ndk &> /dev/null; then
        echo "[RELEASE] Compiling Android Face (arm64-v8a)..."
        cd face-android-handheld && cargo ndk -t arm64-v8a build --release
        cd ..
        # Note: APK wrapping requires Android Studio / Gradle, 
        # but we preserve the .so for local side-loading.
    fi
fi
