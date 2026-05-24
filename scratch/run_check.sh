#!/usr/bin/env bash
nix-shell -p gnumake rustc cargo pkg-config openssl alsa-lib libxkbcommon wayland --run 'make check'
