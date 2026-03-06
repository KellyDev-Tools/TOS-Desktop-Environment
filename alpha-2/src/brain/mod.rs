pub mod ipc_handler;
pub mod hierarchy;
pub mod sector;
pub mod state;
pub mod shell;
pub mod module_manager;

use std::sync::{Arc, Mutex};
use std::thread;
use crate::common::TosState;
use self::ipc_handler::IpcHandler;
use self::shell::ShellApi;
use self::module_manager::ModuleManager;

pub struct Brain {
    pub state: Arc<Mutex<TosState>>,
    pub ipc: Arc<IpcHandler>,
    pub shell: Arc<Mutex<ShellApi>>,
    pub services: Arc<crate::services::ServiceManager>,
    pub modules: Arc<ModuleManager>,
}

impl Brain {
    pub fn new() -> anyhow::Result<Self> {
        let mut state_val = TosState::default();
        let sessions_dir = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp")).join("tos/sessions");
        let live_path = sessions_dir.join("_live.tos-session");
        if let Ok(content) = std::fs::read_to_string(&live_path) {
            if let Ok(live_state) = serde_json::from_str::<TosState>(&content) {
                state_val = live_state;
            }
        }
        
        let sid = state_val.sectors[0].id;
        let hid = state_val.sectors[0].hubs[0].id;
        let state = Arc::new(Mutex::new(state_val));
        
        let services = Arc::new(crate::services::ServiceManager::new());
        let modules = Arc::new(ModuleManager::new(std::path::PathBuf::from("./modules")));
        services.ai.set_module_manager(modules.clone());
        
        let shell_obj = ShellApi::new(state.clone(), modules.clone(), sid, hid)?;
        let shell = Arc::new(Mutex::new(shell_obj));
        let ipc = Arc::new(IpcHandler::new(state.clone(), shell.clone(), services.clone()));
        
        services.set_ipc(ipc.clone());
        
        let mut loaded_settings = None;
        match services.settings.load() {
            Ok(loaded) => {
                loaded_settings = Some(loaded);
            }
            Err(e) => {
                services.logger.log(&format!("Failed to load settings: {}", e), 3);
            }
        }

        if let Some(settings) = loaded_settings {
            {
                let mut lock = state.lock().unwrap();
                lock.settings = settings;
            }
            services.logger.log("Persistent settings loaded.", 1);
        }

        services.logger.log("Brain Core Initialized.", 2);
        services.audio.play_earcon("system_ready");
        
        // Spawn the background logic thread for state heartbeats
        let state_clock = state.clone();
        let svc_clock = services.clone();
        thread::spawn(move || {
            let mut tick = 0;
            loop {
                thread::sleep(std::time::Duration::from_secs(1));
                tick += 1;
                
                if let Ok(mut lock) = state_clock.lock() {
                    lock.brain_time = chrono::Local::now().format("%H:%M:%S").to_string();
                    lock.version += 1;

                    // Refresh tactical priorities every 5 ticks (§21)
                    if tick % 5 == 0 {
                        let sector_ids: Vec<uuid::Uuid> = lock.sectors.iter().map(|s| s.id).collect();
                        for sid in sector_ids {
                            if let Ok(score) = svc_clock.priority.calculate_priority(sid) {
                                if let Some(sector) = lock.sectors.iter_mut().find(|s| s.id == sid) {
                                    sector.priority = score.rank;
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(Self { state, ipc, shell, services, modules })
    }
}
