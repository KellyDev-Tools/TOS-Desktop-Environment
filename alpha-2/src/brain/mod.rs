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
    pub ipc: IpcHandler,
    pub shell: Arc<Mutex<ShellApi>>,
}

impl Brain {
    pub fn new() -> anyhow::Result<Self> {
        let state = Arc::new(Mutex::new(TosState::default()));
        let shell = Arc::new(Mutex::new(ShellApi::new(state.clone())?));
        let ipc = IpcHandler::new(state.clone(), shell.clone());
        
        Ok(Self { state, ipc, shell })
    }
}
