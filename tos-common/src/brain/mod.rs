pub mod cortex_registry;
pub mod hierarchy;
pub mod ipc_handler;
pub mod module_manager;
pub mod renderer_manager;
pub mod sector;
pub mod shell;

use self::ipc_handler::IpcHandler;
use self::module_manager::ModuleManager;
use self::shell::ShellApi;
use crate::TosState;
use std::sync::{Arc, Mutex};
use std::thread;
use std::process::Command;

pub struct Brain {
    pub state: Arc<Mutex<TosState>>,
    pub ipc: Arc<IpcHandler>,
    pub shell: Arc<Mutex<ShellApi>>,
    pub services: Arc<crate::services::ServiceManager>,
    pub modules: Arc<ModuleManager>,
    pub cortex: Arc<Mutex<crate::brain::cortex_registry::CortexRegistry>>,
}

impl Brain {
    pub fn new() -> anyhow::Result<Self> {
        // Load TOS config (tos.toml) from CLI/env/XDG/cwd/defaults.
        let config = crate::config::TosConfig::load();
        let sessions_dir = config.sessions_dir();

        let mut state_val = TosState::default();
        let live_path = sessions_dir.join("_live.tos-session");
        let mut restored = false;
        if !cfg!(test) {
            if let Ok(content) = std::fs::read_to_string(&live_path) {
                if let Ok(mut live_state) = serde_json::from_str::<TosState>(&content) {
                    // §13.2: Reset transient execution state on restore to avoid stale 'is_running' flags.
                    for sector in &mut live_state.sectors {
                        for hub in &mut sector.hubs {
                            hub.is_running = false;
                        }
                    }
                    state_val = live_state;
                    restored = true;
                }
            }
        }

        let sid = state_val.sectors[0].id;
        let hid = state_val.sectors[0].hubs[0].id;
        let state = Arc::new(Mutex::new(state_val));

        let services = Arc::new(crate::services::ServiceManager::with_config(&config));
        services.set_state(state.clone());
        let modules = Arc::new(ModuleManager::new(std::path::PathBuf::from("./modules")));
        let cortex = Arc::new(Mutex::new(crate::brain::cortex_registry::CortexRegistry::new(modules.clone())));
        services.ai.set_module_manager(modules.clone());
        services.ai.set_cortex_registry(cortex.clone());
        services.lsp.set_module_manager(modules.clone());
        services.bezel.set_module_manager(modules.clone());
        services.audio.set_module_manager(modules.clone());

        let shell_obj = ShellApi::new(
            state.clone(),
            modules.clone(),
            services.ai.clone(),
            services.heuristic.clone(),
            sid,
            hid,
        )?;
        let shell = Arc::new(Mutex::new(shell_obj));
        let ipc = Arc::new(IpcHandler::new(
            state.clone(),
            shell.clone(),
            services.clone(),
        ));

        services.set_ipc(ipc.clone());

        let mut loaded_settings = None;
        match services.settings.load() {
            Ok(loaded) => {
                loaded_settings = Some(loaded);
            }
            Err(e) => {
                services
                    .logger
                    .log(&format!("Failed to load settings: {}", e), 3);
            }
        }

        if let Some(settings) = loaded_settings {
            {
                let mut lock = state.lock().unwrap();
                lock.settings = settings;
            }
            services.logger.log("Persistent settings loaded.", 1);
        }

        // --- Initialize AI Behaviors ---
        {
            let mut lock = state.lock().unwrap();
            
            // Record initial timeline snapshot (§19.1)
            services.timeline.record_snapshot(&lock);
            lock.timeline_history_len = services.timeline.len();

            services.ai.register_defaults(&mut lock);
        }

        // Silent restore: suppress the boot notification.
        if !restored {
            services.logger.log("Brain Core Initialized.", 2);
            services.audio.play_earcon("system_ready");
        } else {
            services.logger.log("Session restored silently.", 1);
        }

        // Spawn the background logic thread for state heartbeats
        let state_clock = state.clone();
        let svc_clock = services.clone();
        thread::spawn(move || {
            let mut tick = 0;
            let mut last_alert_level = 0;
            loop {
                thread::sleep(std::time::Duration::from_secs(1));
                tick += 1;

                if let Ok(mut lock) = state_clock.lock() {
                    lock.brain_time = chrono::Local::now().format("%H:%M:%S").to_string();
                    lock.version += 1;

                    // Periodic Sector Maintenance
                    if tick % 5 == 0 {
                        // Refresh tactical priorities (§21)
                        let sector_ids: Vec<uuid::Uuid> =
                            lock.sectors.iter().map(|s| s.id).collect();
                        for sid in sector_ids {
                            if let Ok(score) = svc_clock.priority.calculate_priority(sid) {
                                if let Some(sector) = lock.sectors.iter_mut().find(|s| s.id == sid)
                                {
                                    sector.priority = score.rank;
                                }
                            }
                        }
                    }

                    // Refresh activity listing and process snapshots (1Hz)
                    crate::brain::sector::SectorManager::refresh_activity_listing(
                        &mut lock,
                        Some(&svc_clock.capture),
                    );

                    // Alert level adaptation (§23.2)
                    let current_alert_level = lock.sectors.iter().map(|s| s.priority).max().unwrap_or(1);
                    let auto_alert_audio = lock.settings.global.get("tos.audio.auto_alert_adaptation").map(|s| s == "true").unwrap_or(true);
                    
                    if auto_alert_audio && current_alert_level != last_alert_level {
                        last_alert_level = current_alert_level;
                        if current_alert_level >= 5 {
                            svc_clock.audio.play_ambient("alert_red");
                            svc_clock.audio.set_volume(crate::services::audio::AudioLayer::Ambient, 0.15);
                        } else if current_alert_level >= 3 {
                            svc_clock.audio.play_ambient("alert_yellow");
                            svc_clock.audio.set_volume(crate::services::audio::AudioLayer::Ambient, 0.08);
                        } else {
                            svc_clock.audio.play_ambient("hum_normal");
                            svc_clock.audio.set_volume(crate::services::audio::AudioLayer::Ambient, 0.05);
                        }
                    }

                    // Drain AI Offline Queue (§4.9)
                    if tick % 10 == 0 {
                        let ai_svc = svc_clock.ai.clone();
                        if let Ok(handle) = tokio::runtime::Handle::try_current() {
                            handle.spawn(async move {
                                let _ = ai_svc.drain_queue().await;
                            });
                        }
                    }

                    // Update Bezel Components (§1.10)
                    svc_clock.bezel.update_state(&mut lock);

                    // Record timeline snapshot (§19.1)
                    svc_clock.timeline.record_snapshot(&lock);
                    lock.timeline_history_len = svc_clock.timeline.len();
                }
            }
        });

        Ok(Self {
            state,
            ipc,
            shell,
            services,
            modules,
            cortex,
        })
    }

    /// Spawns auxiliary daemons (settingsd, loggerd, etc.) for a full TOS session.
    pub fn spawn_daemons(&self) -> anyhow::Result<()> {
        let daemons = [
            "tos-settingsd",
            "tos-loggerd",
            "tos-sessiond",
            "tos-marketplaced",
            "tos-priorityd",
            "tos-heuristicd",
            "tos-searchd",
        ];

        // Get the directory of the current executable to find siblings
        let current_exe = std::env::current_exe()?;
        let bin_dir = current_exe.parent().unwrap();

        for daemon in daemons {
            let mut cmd = if let Ok(path) = std::env::var("TOS_BIN_DIR") {
                Command::new(std::path::Path::new(&path).join(daemon))
            } else if bin_dir.join(daemon).exists() {
                Command::new(bin_dir.join(daemon))
            } else {
                Command::new(daemon) // Fallback to PATH
            };

            // Also check for the 'searchd' alias if tos-searchd isn't found
            if daemon == "tos-searchd" && !bin_dir.join(daemon).exists() {
                if bin_dir.join("searchd").exists() {
                    cmd = Command::new(bin_dir.join("searchd"));
                }
            }

            let mut child = cmd;
            
            // §20.2: Path resolution fallbacks for restricted environments (GDM/systemd)
            if !std::path::Path::new(daemon).exists() && bin_dir.join(daemon).exists() == false {
                let fallbacks = ["/usr/local/bin", "/usr/bin", "/bin"];
                for base in fallbacks {
                    let path = format!("{}/{}", base, daemon);
                    if std::path::Path::new(&path).exists() {
                        child = Command::new(path);
                        break;
                    }
                }
            }

            match child.spawn() {
                Ok(child_proc) => {
                    tracing::info!("[BRAIN] Spawned daemon {}: PID {}", daemon, child_proc.id());
                }
                Err(e) => {
                    tracing::error!("[BRAIN] Failed to spawn daemon {}: {}", daemon, e);
                }
            }
        }

        Ok(())
    }
}
