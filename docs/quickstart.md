# Quickstart — run TOS

The real run path, as it exists today: three systemd **user**
services on your own desktop. (The README's older `make run-web`
path starts the legacy web face for UI development — it is not
the product.) Read [concepts.md](concepts.md) first if the words
Brain, face, or chips are new.

## What you're starting

| Unit | What it does |
|---|---|
| `tos-brain.service` | The Brain daemon (`~/.local/bin/tos-brain`) |
| `tos-face-ui.service` | Serves the face build over loopback HTTP, 127.0.0.1:18701 |
| `tos-face-ambient.service` | The layer-shell face: background, chips, prompt |
| `tos.target` | Pulls in all three — the one start/stop command |

## Prerequisites

- A Wayland session on a compositor with **layer-shell** support
  (Sway, KDE, Hyprland…). **GNOME/Mutter has no layer-shell** —
  run the face inside a nested Sway session instead; this is
  expected, not a failure.
- Rust toolchain and Node (to build the Brain and the face),
  `python3` (the UI server uses it), systemd user sessions.

## Install and start

```sh
packaging/systemd-user/install.sh   # builds + installs, no root
systemctl --user enable --now tos.target
```

`install.sh` builds the Brain and the face, installs the binaries
to `~/.local/bin`, the units to `~/.config/systemd/user/`, and
the face build to `~/.local/share/tos/face/`. From then on TOS
starts at login; `systemctl --user stop tos.target` stops all of
it. Details, config order, and the prompt-summon hotkey binding
live in
[`packaging/systemd-user/README.md`](../packaging/systemd-user/README.md).

Check it's alive:

```sh
systemctl --user status tos.target
journalctl --user -u tos-brain -f    # Brain logs
```

## Smoke checklist

Each line is pass/fail. If one fails, the trap table below is
ordered to match.

1. The terminal session appears as your desktop background, with
   a working shell prompt.
2. Type `ls` and press Enter — it runs exactly as typed, and the
   output renders cleanly (no doubled or garbled glyphs).
3. The chips strip shows your home directory's entries;
   dotfiles are hidden.
4. Type a few letters of a filename — the strip narrows to
   matches (prefix matches first).
5. Tap a directory chip — a `cd` into it happens in the
   terminal, and the strip refreshes to the new directory.
6. Flip the strip's `.*` switch — dotfiles appear; flip it back.
7. Summon the prompt with your configured hotkey — it appears as
   its own overlay, not attached to the terminal. Dismiss it.
8. Restart the face (`systemctl --user restart
   tos-face-ambient`) — the terminal's scrollback replays; your
   shell session itself never restarted.

## Known traps

| Symptom | Cause / fix |
|---|---|
| Face never appears on GNOME | Mutter has no layer-shell. Run nested Sway, or use a compositor that supports it. |
| Typed text renders doubled or garbled, but commands run as typed | You're on an old build with the DOM renderer. Current builds use the canvas renderer; rebuild and reinstall. |
| Face window is blank | The UI server (127.0.0.1:18701) isn't up — WebKitGTK cannot load the face build from a file path. Check `tos-face-ui.service`. |
| Face flickers or fails to start under WebKitGTK | The unit sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` for this; if you run the face by hand, set it too. |
| A remote face can't connect at all | `remote_access` defaults to off, which is loopback-only and fails closed. See the remote section of the [Roadmap](roadmap.md) and `tos.toml`'s `[remote]` settings. |
| A remote face connects but commands bounce with `ERROR: … not available over a remote connection` | That's the three-stream gate working as designed — see [concepts.md](concepts.md), trace 3. |

## The dev loop

- `make check` / `make test` — the gates (see
  [development/standards.md](development/standards.md)).
- `make run-web-dev` — the Svelte face in a browser for UI
  iteration, against mock state. Fine for components; it proves
  nothing about the real face — the smoke checklist above does.
- Linux UI work happens on the NixOS host — see
  [development/nixos.md](development/nixos.md).
