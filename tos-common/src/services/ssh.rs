use crate::platform::ssh_fallback::SshSession;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct SshService {
    sessions: Mutex<HashMap<Uuid, SshSession>>,
}

impl Default for SshService {
    fn default() -> Self {
        Self::new()
    }
}

impl SshService {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn connect(
        &self,
        host: &str,
        state: Arc<Mutex<crate::TosState>>,
        sector_id: Uuid,
        hub_id: Uuid,
    ) -> anyhow::Result<()> {
        let session = SshSession::connect(host, state, sector_id, hub_id)?;
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(hub_id, session); // Map by Hub ID for easy lookup during input routing
        Ok(())
    }

    pub fn write(&self, hub_id: &Uuid, data: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(hub_id) {
            session.write(data)?;
            Ok(())
        } else {
            anyhow::bail!("No active SSH session for hub {}", hub_id)
        }
    }

    pub fn disconnect(&self, hub_id: &Uuid) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(hub_id);
    }
}
