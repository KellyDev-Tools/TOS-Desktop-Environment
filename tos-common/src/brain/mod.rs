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
                if let Ok(live_state) = serde_json::from_str::<TosState>(&content) {
                    state_val = live_state;
                    restored = true;
                }
            }
        }

        let sid = state_val.sectors[0].id;
        let hid = state_val.sectors[0].hubs[0].id;
        let state = Arc::new(Mutex::new(state_val));

        let services = Arc::new(crate::services::ServiceManager::with_config(&config));
        services.lsp.set_state(state.clone());
        let modules = Arc::new(ModuleManager::new(std::path::PathBuf::from("./modules")));
        let cortex = Arc::new(Mutex::new(crate::brain::cortex_registry::CortexRegistry::new(modules.clone())));
        services.ai.set_module_manager(modules.clone());
        services.ai.set_cortex_registry(cortex.clone());

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
}
