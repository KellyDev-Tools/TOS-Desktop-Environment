#[cfg(not(target_os = "android"))]
pub mod pty;

#[cfg(not(target_os = "android"))]
pub use pty::PtyShell as ShellApi;

#[cfg(target_os = "android")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "android")]
use crate::TosState;

#[cfg(target_os = "android")]
pub struct ShellApi {
    _state: Arc<Mutex<TosState>>,
    _sector_id: uuid::Uuid,
    _hub_id: uuid::Uuid,
}

#[cfg(target_os = "android")]
impl ShellApi {
    pub fn new(
        state: Arc<Mutex<TosState>>,
        _modules: Arc<crate::brain::module_manager::ModuleManager>,
        _ai: Arc<crate::services::AiService>,
        _heuristic: Arc<crate::services::HeuristicService>,
        sector_id: uuid::Uuid,
        hub_id: uuid::Uuid,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            _state: state,
            _sector_id: sector_id,
            _hub_id: hub_id,
        })
    }
    pub fn write(&mut self, _data: &str) -> anyhow::Result<()> {
        Ok(())
    }
    pub fn resize(&self, _rows: u16, _cols: u16) -> anyhow::Result<()> {
        Ok(())
    }
    pub fn send_signal(&mut self, _signal: &str) -> anyhow::Result<()> {
        Ok(())
    }
    pub fn force_kill(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
    pub fn exec_sandboxed(_command: &str, _cwd: std::path::PathBuf) -> anyhow::Result<(String, crate::modules::sandbox::OverlaySandbox)> {
        Err(anyhow::anyhow!("Sandboxing not supported on Android"))
    }
}
