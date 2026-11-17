#!/usr/bin/env bash
# Install the TOS Phase 1 slice as ordinary systemd *user* services:
# the Brain daemon and the ambient layer-shell face, running inside
# your normal desktop session. No root, no display manager, no TOS
# session. Safe to re-run (it rebuilds and reinstalls in place).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BIN_DIR="$HOME/.local/bin"
UNIT_DIR="$HOME/.config/systemd/user"
FACE_DIR="$HOME/.local/share/tos/face"

cd "$REPO_ROOT"

echo "==> Building tos-brain and tos-face-ambient (release)"
cargo build --release -p tos-brain -p tos-face-ambient

echo "==> Building the Svelte face"
if command -v npm >/dev/null 2>&1; then
    (cd face-svelte-ui && npm install && npm run build)
else
    echo "    npm not found — skipping the face build."
    echo "    Build face-svelte-ui elsewhere and copy its build/ to $FACE_DIR"
fi

echo "==> Installing binaries to $BIN_DIR"
install -Dm755 target/release/tos-brain "$BIN_DIR/tos-brain"
install -Dm755 target/release/tos-face-ambient "$BIN_DIR/tos-face-ambient"

if [ -d face-svelte-ui/build ]; then
    echo "==> Installing face UI to $FACE_DIR"
    rm -rf "$FACE_DIR"
    mkdir -p "$FACE_DIR"
    cp -r face-svelte-ui/build/. "$FACE_DIR/"
fi

echo "==> Installing user units to $UNIT_DIR"
install -Dm644 packaging/systemd-user/tos-brain.service "$UNIT_DIR/tos-brain.service"
install -Dm644 packaging/systemd-user/tos-face-ui.service "$UNIT_DIR/tos-face-ui.service"
install -Dm644 packaging/systemd-user/tos-face-ambient.service "$UNIT_DIR/tos-face-ambient.service"
install -Dm644 packaging/systemd-user/tos.target "$UNIT_DIR/tos.target"

systemctl --user daemon-reload

cat <<'EOF'

Done. Start TOS (Brain, face UI server, and the ambient face) with
one command — and enable it to start at every login:

    systemctl --user enable --now tos.target

Stop everything with `systemctl --user stop tos.target`. Check on
them with `systemctl --user status tos-brain tos-face-ui tos-face-ambient`
and `journalctl --user -u tos-brain -f`.

Bind a hotkey in your DE to summon the prompt:

    tos-face-ambient --toggle-prompt
EOF
