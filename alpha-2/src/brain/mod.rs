pub mod ipc_handler;
pub mod hierarchy;
pub mod sector;
pub mod state;
pub mod shell;

use std::sync::{Arc, Mutex};
use crate::common::TosState;
use self::ipc_handler::IpcHandler;
use self::shell::ShellApi;

pub struct Brain {
    pub state: Arc<Mutex<TosState>>,
    pub ipc: Arc<IpcHandler>,
    pub shell: Arc<Mutex<ShellApi>>,
    pub services: Arc<crate::services::ServiceManager>,
}

impl Brain {
    pub fn new() -> anyhow::Result<Self> {
        let state_val = TosState::default();
        let sid = state_val.sectors[0].id;
        let hid = state_val.sectors[0].hubs[0].id;
        let state = Arc::new(Mutex::new(state_val));
        
        let shell_obj = ShellApi::new(state.clone(), sid, hid)?;
        let shell = Arc::new(Mutex::new(shell_obj));
        let ipc = Arc::new(IpcHandler::new(state.clone(), shell.clone()));
        let services = Arc::new(crate::services::ServiceManager::new(state.clone()));
        
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
                lock.settings.extend(settings);
            }
            services.logger.log("Persistent settings loaded.", 1);
        }

        services.logger.log("Brain Core Initialized.", 2);
        services.audio.play_earcon("system_ready");
        
        Ok(Self { state, ipc, shell, services })
    }
}
