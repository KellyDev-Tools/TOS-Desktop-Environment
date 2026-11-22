# Concepts — TOS on one page

The mental model. Everything else in the documentation assumes
you hold this page in your head; the [spec](spec/architecture.md)
is where each idea is defined in full.

## The picture

```
┌─ Host desktop (GNOME / KDE / Sway …) ──────────────────────┐
│  windows · apps · session · settings — not TOS's job        │
│                                                             │
│  ┌─ Face (one process, three layer-shell surfaces) ──────┐  │
│  │  background   the terminal session, always there      │  │
│  │  chips        a touch-friendly strip of shortcuts     │  │
│  │  prompt       an input overlay, summoned on demand    │  │
│  └──────────────────────┬────────────────────────────────┘  │
└─────────────────────────┼───────────────────────────────────┘
                    WebSocket (wss)
┌─────────────────────────┴───────────────────────────────────┐
│  Brain (headless Rust daemon)                               │
│   · owns the session: one shell in a PTY                    │
│   · owns the state: cwd, directory listing, chip state      │
│   · may run on this machine — or on a server                │
└─────────────────────────┬───────────────────────────────────┘
                     your shell (zsh / bash / fish)
```

TOS is an **ambient layer**, not a desktop environment. The host
keeps everything a desktop normally does; TOS adds a terminal
that lives as your background and two overlays that help you
drive it.

## The ten terms

| Term | What it means |
|---|---|
| **Brain** | The headless daemon that owns the session and all state. Faces are replaceable; the Brain is the system. |
| **Face** | The visible client: today, a GTK4 + WebKitGTK layer-shell app rendering the Svelte UI. A face can be local or remote, on a screen or (later) XR. |
| **Surfaces** | The face's three layers: **background** (terminal), **chips**, **prompt**. They act independently; only the background is always present. |
| **Terminal** | A completely normal terminal — a real shell in a real PTY. Nothing about it is simulated, scraped, or reinterpreted. |
| **Command Hub** | The Brain-side owner of session state: current directory, listing, chip state, prompt handling. If the Brain knows it, the Hub holds it. |
| **Chip** | A tappable shortcut in the chips strip, derived from state the Brain already holds (today: the current directory's entries). Tapping a chip *types into the terminal* — see the traces below. |
| **Chip generation** | Every listing refresh mints a new generation of chips. Taps against an old generation are deprecated, never acted on blindly — the directory may have changed under them. |
| **Prompt** | An independent overlay with its own input bar, docked to nothing. Submitting sends a command to the Brain; it is not the terminal's input line and doesn't borrow it. |
| **Three streams** | The only things a *remote* face may touch: **terminal, chip state, prompt**. Everything else the Brain can do is refused over remote connections. Local connections are unrestricted. |
| **Parked** | Deliberately frozen, not abandoned and not buildable: the satellite daemons (search, heuristics, priority, marketplace, logging), the Electron/Android faces, packaging, the AI module. Each revives only when the [Roadmap](roadmap.md) says so; the [Vision](vision.md) ledger names what revives each one. |

## The invariants

Five rules everything else obeys (from [AGENTS.md](../AGENTS.md)):

1. The terminal stays a completely normal terminal.
2. Terminal output is never scraped or sniffed to drive features.
3. Chips derive only from state the Brain already holds.
4. The prompt is an independent overlay, docked to nothing.
5. Remote = three streams only; the gate is the connection, not
   the command.

## Three traces

The fastest way to understand TOS is to follow three events end
to end.

### 1. A keystroke

You type `ls` in the background terminal.

1. The face's terminal (xterm, canvas renderer) sends the bytes
   as a `terminal_input_hex` command over the WebSocket.
2. The Brain's `PtyShell::write` — the single funnel for *all*
   terminal input — writes them to the PTY. An input-line model
   watches at this same funnel, tracking what you're typing so
   the chips can filter against it.
3. Your shell runs `ls`. Output streams back as `pty_output`
   events to the face (and into a 256 KiB scrollback buffer, so a
   reconnecting face can be replayed the whole truth).
4. The shell reports its working directory (OSC 7 after the
   command; the Brain also checks `/proc/<pid>/cwd`), the Command
   Hub refreshes the directory listing, and a **new chip
   generation** — the directory's entries — goes out to the face.
5. As you keep typing the next command, the strip filters live:
   prefix matches first, then substring matches, directories
   first within each class. Dotfiles stay hidden unless you flip
   the strip's `.*` switch.

### 2. A chip tap

You tap the **Documents** chip.

1. The face sends the literal text `cd 'Documents'` followed by
   Enter — as terminal input, through the same funnel as your
   keystrokes. **There is no chip API that changes directories.**
   Chips type; the terminal obeys.
2. A *file* chip instead inserts the shell-quoted filename plus a
   trailing space, mid-command, unsent — ready for you to finish
   the command. A *breadcrumb* chip types `cd '<ancestor>'`.
3. The one exception: the `.*` dotfiles switch is panel state,
   not terminal content, so it talks to the Brain directly
   (`chips_set_show_hidden`).

### 3. A remote refusal

A face on another machine connects to a Brain whose
`remote_access` is on (WebSocket port 7001, over the tailnet),
and sends `system_reset`.

1. Before dispatch, the Brain's remote gate checks the command's
   prefix against the ten allowed three-stream prefixes
   (`terminal_input_hex`, `terminal_resize`, `terminal_signal`,
   `prompt_submit`, `force_prompt_submit`, `get_state`,
   `get_state_delta`, `chips_set_show_hidden`, `tos_ports`,
   `face_register`).
2. `system_reset` isn't among them, so the sender gets a loud
   refusal: *"ERROR: system_reset is not available over a remote
   connection (three-stream narrowing: terminal, chip state,
   prompt only)"*.
3. The same command over the local socket works fine. The gate
   judges the connection, not the command.

## Where things live

| Concept | Code | Exercised by |
|---|---|---|
| Brain daemon | `tos-brain/`, `tos-common/src/brain/` | `cargo test --workspace` |
| PTY + session | `tos-common/src/brain/shell/pty.rs` | shell integration tests |
| Input-line model | `tos-common/src/brain/shell/input_line.rs` | unit tests + chips E2E |
| Remote gate | `tos-common/src/platform/remote_server.rs` | narrowing E2E spec |
| Brain config | `tos-common/src/config.rs` (`remote_access`, `bind_address`, ports 7000/7001) | remote E2E spec |
| Face surfaces | `face-ambient-linux/` (layer-shell), socket `$XDG_RUNTIME_DIR/tos-face.sock` | slice E2E spec |
| Chips + prompt UI | `face-svelte-ui/src/lib/components/AcceleratorChips.svelte`, `PromptInputBar.svelte` | gated Playwright specs |
| Running it | `packaging/systemd-user/` (`tos.target`) | [quickstart.md](quickstart.md) |

Next: [start-here.md](start-here.md) for the route through the
rest, [quickstart.md](quickstart.md) to run it.
