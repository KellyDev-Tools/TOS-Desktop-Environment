# TOS as user services (Phase 1 slice)

The Phase 1 direction: TOS is an **ambient layer inside your existing
desktop session**, not a session of its own. So the Brain and the face
run as ordinary **systemd user services** — no root, no display
manager integration, no TOS login session. GNOME/KDE keep managing
windows and apps; TOS haunts the edges.

## Install and start

```sh
packaging/systemd-user/install.sh
systemctl --user enable --now tos.target
```

The installer builds release binaries into `~/.local/bin`, the face
UI into `~/.local/share/tos/face/`, and the units into
`~/.config/systemd/user/`. It is safe to re-run.

`tos.target` is the one simple start: it pulls in the Brain, the face
UI server, and the ambient face, and each is `PartOf` it — so
`systemctl --user stop tos.target` stops all three, and enabling the
target starts TOS at every login.

## The units

| Unit | What it is |
| --- | --- |
| `tos-brain.service` | The headless Brain daemon: sessions, terminals, the Discovery Gate |
| `tos-face-ui.service` | Serves the face build over loopback HTTP (127.0.0.1:18701) — WebKitGTK will not load the build's ES module imports from a file path (found in the 2026-11-16 NixOS smoke test), so the face loads its UI from this server |
| `tos-face-ambient.service` | The layer-shell face: background terminal, chips strip, prompt overlay. Skips itself when no Wayland session is present |

Summon or dismiss the prompt from any hotkey your DE lets you bind:

```sh
tos-face-ambient --toggle-prompt    # also: --show-prompt / --hide-prompt
```

## Brain endpoints and config

- `/tmp/brain.sock` — Discovery Gate (daemon registration, port map)
- TCP **7000** — anchor / remote server
- WebSocket **7001** — face connections

Config is read from, in order: `--config <path>`, `$TOS_CONFIG`,
`~/.config/tos/tos.toml`, `./tos.toml`, built-in defaults. State and
settings live under the XDG config/data directories (`~/.config/tos`,
`~/.local/share/tos`).

## Caveats (honest, as of Phase 1)

- The face needs a **layer-shell compositor** (KDE Plasma, Sway,
  Hyprland, COSMIC). GNOME/Mutter does not offer layer-shell to
  regular clients — the GNOME path is a Phase 4 shim. (The 2026-11-16
  smoke test ran it under Sway nested inside a GNOME session.)
- **NixOS + WebKitGTK:** WebKit needs glib-networking's GIO modules
  for TLS. If the face shows certificate or network errors, point it
  at them for your user session before starting the target:
  `systemctl --user set-environment GIO_EXTRA_MODULES=$(echo /nix/store/*-glib-networking-*/lib/gio/modules | awk '{print $NF}')`.
  The face unit already sets `WEBKIT_DISABLE_DMABUF_RENDERER=1`,
  which NixOS (and many other systems) needs for the face to render.
- The Brain currently binds 7000/7001 on **all interfaces** (it was
  built remote-first). Fine on a trusted home LAN; Phase 2 narrows
  this to the tailnet/mesh as the default topology.
- The satellite daemons (sessiond, settingsd, …) are **parked** by
  the scope split and are not part of this install.
