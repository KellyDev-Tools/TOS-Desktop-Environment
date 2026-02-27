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
}

impl Brain {
    pub fn new() -> anyhow::Result<Self> {
        let state_val = TosState::default();
        let sid = state_val.sectors[0].id;
        let hid = state_val.sectors[0].hubs[0].id;
        let state = Arc::new(Mutex::new(state_val));
        
        let shell = Arc::new(Mutex::new(ShellApi::new(state.clone(), sid, hid)?));
        let ipc = Arc::new(IpcHandler::new(state.clone(), shell.clone()));
        
        Ok(Self { state, ipc, shell })
    }
}
