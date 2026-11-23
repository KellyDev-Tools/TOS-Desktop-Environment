//! App placement (Architecture §26.4): where an app runs — the
//! face's host machine or the Brain's machine — is independent of
//! where it displays. The Brain routes prompt submissions using
//! desktop-entry catalogues: its own machine's, read from the XDG
//! data directories, and the face host's, synced by the face.
//!
//! This module is the decision half: catalogue parsing and
//! scanning, override parsing, and resolution. Execution happens
//! at the face's edge, on a launch directive the IPC layer emits;
//! nothing here spawns a process.

use serde::{Deserialize, Serialize};

/// One GUI app from a machine's desktop-entry catalogue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppEntry {
    /// The desktop file's stem (`firefox` for `firefox.desktop`).
    pub id: String,
    /// The entry's display name.
    pub name: String,
    /// The entry's `Exec` line, field codes included; the
    /// executing edge strips them.
    pub exec: String,
}

/// Where a launch runs. `Session` is not a placement decision —
/// it is the absence of one: the submission takes the terminal
/// path, on the Brain's machine, as it always has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Placement {
    Session,
    Host,
    ServerDisplay,
}

impl Placement {
    pub fn as_str(&self) -> &'static str {
        match self {
            Placement::Session => "session",
            Placement::Host => "host",
            Placement::ServerDisplay => "server",
        }
    }

    pub fn parse(s: &str) -> Option<Placement> {
        match s.trim().to_ascii_lowercase().as_str() {
            "host" | "here" => Some(Placement::Host),
            "server" | "server-display" | "server_display" => {
                Some(Placement::ServerDisplay)
            }
            "session" => Some(Placement::Session),
            _ => None,
        }
    }

    /// The other machine, for a relaunch. Session has no other.
    pub fn opposite(&self) -> Option<Placement> {
        match self {
            Placement::Host => Some(Placement::ServerDisplay),
            Placement::ServerDisplay => Some(Placement::Host),
            Placement::Session => None,
        }
    }
}

/// How prominent the placement choice is, per
/// `tos.placement.prominence`. All three levels are real; which
/// one is right is an open experiment, decided by use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prominence {
    Invisible,
    Glyph,
    Explicit,
}

impl Prominence {
    pub fn parse_setting(value: Option<&str>) -> Prominence {
        match value.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
            Some("invisible") => Prominence::Invisible,
            Some("explicit") => Prominence::Explicit,
            _ => Prominence::Glyph,
        }
    }
}

/// A GUI launch waiting for its placement to be chosen
/// (prominence `explicit`). Lives in Brain state so any face can
/// present and answer it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingPlacement {
    pub id: String,
    /// The submission as typed (override stripped).
    pub command: String,
    pub app_name: String,
    /// The exec each machine would run; `None` where that
    /// machine does not have the app.
    pub host_exec: Option<String>,
    pub server_exec: Option<String>,
}

/// One executed launch, in the registry faces render glyphs
/// from and relaunches act on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchRecord {
    pub id: String,
    /// The catalogue id of the app (its desktop-file stem).
    pub app_id: String,
    pub app: String,
    pub exec: String,
    /// `Placement::as_str` of the machine that ran it.
    pub placement: String,
    /// Local time, display-formatted like the rest of state.
    pub at: String,
}

/// The Brain's instruction to a face: act on this launch, on
/// your machine. Emitted on the state channel; a face honours a
/// directive once, by id, and always logs that it did.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchDirective {
    pub id: String,
    pub app: String,
    pub exec: String,
    /// `Placement::as_str`: `host` (spawn locally) or `server`
    /// (spawn via waypipe to the Brain's machine).
    pub placement: String,
}

/// A resolved GUI launch: the app, and the exec line each
/// machine would run for it.
#[derive(Debug, Clone, PartialEq)]
pub struct LaunchMatch {
    /// The catalogue id of the matched app.
    pub app_id: String,
    pub app_name: String,
    pub host_exec: Option<String>,
    pub server_exec: Option<String>,
}

impl LaunchMatch {
    pub fn exec_for(&self, placement: Placement) -> Option<&str> {
        match placement {
            Placement::Host => self.host_exec.as_deref(),
            Placement::ServerDisplay => self.server_exec.as_deref(),
            Placement::Session => None,
        }
    }

    /// The default resolution with no override and no explicit
    /// choice pending: host if the host has it (both-present
    /// included — display and GPU are there), otherwise the
    /// server.
    pub fn default_placement(&self) -> Placement {
        if self.host_exec.is_some() {
            Placement::Host
        } else {
            Placement::ServerDisplay
        }
    }
}

/// Parse one `.desktop` file's contents into an entry. Only
/// launchable applications qualify: `Type=Application`, an
/// `Exec`, and not `NoDisplay` / `Hidden`.
pub fn parse_desktop_entry(id: &str, contents: &str) -> Option<AppEntry> {
    let mut in_entry = false;
    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut is_application = false;
    let mut hidden = false;
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "Type" => is_application = value.trim() == "Application",
            "Name" if name.is_none() => name = Some(value.trim().to_string()),
            "Exec" if exec.is_none() => exec = Some(value.trim().to_string()),
            "NoDisplay" | "Hidden" => {
                hidden = value.trim().eq_ignore_ascii_case("true")
            }
            _ => {}
        }
    }
    if !is_application || hidden {
        return None;
    }
    Some(AppEntry {
        id: id.to_string(),
        name: name?,
        exec: exec?,
    })
}

/// The `applications` directories to scan, most specific first:
/// `XDG_DATA_HOME`, then each of `XDG_DATA_DIRS`.
fn catalogue_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(home) = std::env::var("XDG_DATA_HOME") {
        if !home.is_empty() {
            dirs.push(std::path::PathBuf::from(home));
        }
    } else if let Ok(home) = std::env::var("HOME") {
        dirs.push(std::path::PathBuf::from(home).join(".local/share"));
    }
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    for d in data_dirs.split(':') {
        if !d.is_empty() {
            dirs.push(std::path::PathBuf::from(d));
        }
    }
    dirs.into_iter().map(|d| d.join("applications")).collect()
}

/// Scan this machine's desktop-entry catalogue. Earlier
/// directories win an id collision (user entries shadow system
/// ones, as launchers do).
pub fn scan_catalogue() -> Vec<AppEntry> {
    scan_catalogue_in(&catalogue_dirs())
}

pub fn scan_catalogue_in(dirs: &[std::path::PathBuf]) -> Vec<AppEntry> {
    let mut out: Vec<AppEntry> = Vec::new();
    for dir in dirs {
        let Ok(read) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut paths: Vec<_> = read.flatten().map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if out.iter().any(|e| e.id == stem) {
                continue;
            }
            let Ok(contents) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Some(entry) = parse_desktop_entry(stem, &contents) {
                out.push(entry);
            }
        }
    }
    out
}

/// Split an explicit override prefix off a submission:
/// `here: …` forces host, `server: …` forces server-display.
/// Returns the override (if any) and the remaining command.
pub fn parse_override(command: &str) -> (Option<Placement>, &str) {
    let trimmed = command.trim_start();
    for (prefix, placement) in [
        ("here:", Placement::Host),
        ("server:", Placement::ServerDisplay),
    ] {
        if let Some(rest) = trimmed
            .get(..prefix.len())
            .filter(|p| p.eq_ignore_ascii_case(prefix))
            .and_then(|_| trimmed.get(prefix.len()..))
        {
            return (Some(placement), rest.trim());
        }
    }
    (None, trimmed)
}

fn find_in<'a>(catalogue: &'a [AppEntry], token: &str) -> Option<&'a AppEntry> {
    catalogue.iter().find(|e| {
        e.id.eq_ignore_ascii_case(token) || e.name.eq_ignore_ascii_case(token)
    })
}

/// Resolve a submission (override already stripped) to a GUI
/// launch, if it is one: its first token names a catalogue app
/// on either machine. The launch exec is the entry's `Exec`
/// plus any arguments the user typed after the app token.
pub fn resolve_launch(
    command: &str,
    server_catalogue: &[AppEntry],
    host_catalogue: &[AppEntry],
) -> Option<LaunchMatch> {
    let mut parts = command.trim().splitn(2, char::is_whitespace);
    let token = parts.next()?;
    if token.is_empty() {
        return None;
    }
    let args = parts.next().map(str::trim).unwrap_or("");
    let with_args = |entry: &AppEntry| {
        if args.is_empty() {
            entry.exec.clone()
        } else {
            format!("{} {}", entry.exec, args)
        }
    };
    let server = find_in(server_catalogue, token);
    let host = find_in(host_catalogue, token);
    if server.is_none() && host.is_none() {
        return None;
    }
    let primary = server.or(host);
    Some(LaunchMatch {
        app_id: primary.map(|e| e.id.clone()).unwrap_or_default(),
        app_name: primary.map(|e| e.name.clone()).unwrap_or_default(),
        host_exec: host.map(with_args),
        server_exec: server.map(with_args),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const FIREFOX: &str = "[Desktop Entry]\nType=Application\nName=Firefox\nExec=firefox %u\n";
    const TERM: &str =
        "[Desktop Entry]\nType=Application\nName=Terminal\nExec=kgx\nNoDisplay=true\n";

    fn entry(id: &str, name: &str, exec: &str) -> AppEntry {
        AppEntry {
            id: id.into(),
            name: name.into(),
            exec: exec.into(),
        }
    }

    #[test]
    fn parses_a_launchable_entry() {
        let e = parse_desktop_entry("firefox", FIREFOX).unwrap();
        assert_eq!(e.name, "Firefox");
        assert_eq!(e.exec, "firefox %u");
    }

    #[test]
    fn nodisplay_and_non_apps_are_not_launchable() {
        assert!(parse_desktop_entry("kgx", TERM).is_none());
        let link = "[Desktop Entry]\nType=Link\nName=X\nExec=x\n";
        assert!(parse_desktop_entry("x", link).is_none());
    }

    #[test]
    fn scan_reads_dirs_in_order_and_shadows() {
        let base = std::env::temp_dir().join(format!(
            "tos-placement-scan-{}",
            std::process::id()
        ));
        let user = base.join("user/applications");
        let sys = base.join("sys/applications");
        std::fs::create_dir_all(&user).unwrap();
        std::fs::create_dir_all(&sys).unwrap();
        let mut f = std::fs::File::create(user.join("firefox.desktop")).unwrap();
        write!(f, "{}", FIREFOX.replace("firefox %u", "firefox-user %u")).unwrap();
        let mut f = std::fs::File::create(sys.join("firefox.desktop")).unwrap();
        write!(f, "{}", FIREFOX).unwrap();
        let mut f = std::fs::File::create(sys.join("htop.desktop")).unwrap();
        write!(
            f,
            "[Desktop Entry]\nType=Application\nName=Htop\nExec=htop\n"
        )
        .unwrap();
        let cat = scan_catalogue_in(&[user, sys]);
        std::fs::remove_dir_all(&base).ok();
        assert_eq!(cat.len(), 2);
        let ff = cat.iter().find(|e| e.id == "firefox").unwrap();
        assert_eq!(ff.exec, "firefox-user %u", "user dir shadows system");
    }

    #[test]
    fn overrides_strip_and_force() {
        assert_eq!(
            parse_override("here: firefox"),
            (Some(Placement::Host), "firefox")
        );
        assert_eq!(
            parse_override("SERVER: htop"),
            (Some(Placement::ServerDisplay), "htop")
        );
        assert_eq!(parse_override("ls -la"), (None, "ls -la"));
        assert_eq!(parse_override("server-room: x"), (None, "server-room: x"));
    }

    #[test]
    fn resolution_matrix() {
        let server = vec![entry("firefox", "Firefox", "firefox %u")];
        let host = vec![
            entry("firefox", "Firefox", "firefox %u"),
            entry("code", "Code", "code"),
        ];
        // Both present → host by default.
        let m = resolve_launch("firefox", &server, &host).unwrap();
        assert_eq!(m.default_placement(), Placement::Host);
        // Host-only.
        let m = resolve_launch("code --new-window", &server, &host).unwrap();
        assert_eq!(m.host_exec.as_deref(), Some("code --new-window"));
        assert!(m.server_exec.is_none());
        // Server-only.
        let host_empty: Vec<AppEntry> = vec![];
        let m = resolve_launch("Firefox", &server, &host_empty).unwrap();
        assert_eq!(m.default_placement(), Placement::ServerDisplay);
        // No match → not a launch.
        assert!(resolve_launch("ls -la", &server, &host).is_none());
    }

    #[test]
    fn desktop_entry_parsing_details() {
        // Keys outside [Desktop Entry] do not count.
        let grouped = "[Desktop Entry]\nType=Application\nName=A\nExec=a\n\
                       [Desktop Action new]\nName=B\nExec=b\n";
        let e = parse_desktop_entry("a", grouped).unwrap();
        assert_eq!(e.name, "A");
        assert_eq!(e.exec, "a");
        // A localized Name is not the Name; the plain one wins
        // wherever it appears.
        let localized = "[Desktop Entry]\nType=Application\nName[fr]=Renard\nName=Firefox\nExec=firefox\n";
        assert_eq!(parse_desktop_entry("f", localized).unwrap().name, "Firefox");
        // Missing Type or Exec: not launchable.
        let no_type = "[Desktop Entry]\nName=X\nExec=x\n";
        assert!(parse_desktop_entry("x", no_type).is_none());
        let no_exec = "[Desktop Entry]\nType=Application\nName=X\n";
        assert!(parse_desktop_entry("x", no_exec).is_none());
        // Comment lines and blank lines are inert.
        let commented = "# a comment\n\n[Desktop Entry]\nType=Application\nName=X\nExec=x\n";
        assert!(parse_desktop_entry("x", commented).is_some());
    }

    #[test]
    fn override_and_matching_edges() {
        // No space after the colon still overrides.
        assert_eq!(
            parse_override("here:firefox"),
            (Some(Placement::Host), "firefox")
        );
        // A word merely starting with the letters is not an override.
        assert_eq!(parse_override("hereford: x"), (None, "hereford: x"));
        // Matching is by id or by display name, case-insensitively.
        let server = vec![entry("my-app", "My App", "my-app")];
        assert!(resolve_launch("MY-APP", &server, &[]).is_some());
        assert!(resolve_launch("my app", &server, &[]).is_none(),
            "a name with a space cannot be a first token");
        // Extra whitespace between the token and its args collapses.
        let m = resolve_launch("my-app   --flag", &server, &[]).unwrap();
        assert_eq!(m.server_exec.as_deref(), Some("my-app --flag"));
        // An empty submission is not a launch.
        assert!(resolve_launch("   ", &server, &[]).is_none());
    }

    #[test]
    fn placement_vocabulary() {
        assert_eq!(Placement::parse("here"), Some(Placement::Host));
        assert_eq!(Placement::parse("SERVER"), Some(Placement::ServerDisplay));
        assert_eq!(Placement::parse("nowhere"), None);
        assert_eq!(Placement::Host.opposite(), Some(Placement::ServerDisplay));
        assert_eq!(Placement::ServerDisplay.opposite(), Some(Placement::Host));
        assert_eq!(Placement::Session.opposite(), None);
        let m = LaunchMatch {
            app_id: "a".into(),
            app_name: "A".into(),
            host_exec: Some("ha".into()),
            server_exec: None,
        };
        assert_eq!(m.exec_for(Placement::Host), Some("ha"));
        assert_eq!(m.exec_for(Placement::ServerDisplay), None);
        assert_eq!(m.exec_for(Placement::Session), None);
    }

    #[test]
    fn prominence_defaults_to_glyph() {
        assert_eq!(Prominence::parse_setting(None), Prominence::Glyph);
        assert_eq!(
            Prominence::parse_setting(Some("explicit")),
            Prominence::Explicit
        );
        assert_eq!(
            Prominence::parse_setting(Some("invisible")),
            Prominence::Invisible
        );
        assert_eq!(
            Prominence::parse_setting(Some("bogus")),
            Prominence::Glyph
        );
    }
}
