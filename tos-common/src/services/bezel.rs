use crate::modules::BezelModule;
use crate::state::{TosState, BezelComponentState};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct BezelService {
    active_components: Mutex<HashMap<String, Box<dyn BezelModule>>>,
    module_manager: Mutex<Option<Arc<crate::brain::module_manager::ModuleManager>>>,
}

impl BezelService {
    pub fn new() -> Self {
        Self {
            active_components: Mutex::new(HashMap::new()),
            module_manager: Mutex::new(None),
        }
    }

    pub fn set_module_manager(&self, mm: Arc<crate::brain::module_manager::ModuleManager>) {
        *self.module_manager.lock().unwrap() = Some(mm);
    }

    pub fn activate_component(&self, id: &str) -> anyhow::Result<()> {
        let mm = self.module_manager.lock().unwrap();
        let mm = mm.as_ref().ok_or_else(|| anyhow::anyhow!("ModuleManager not set"))?;
        
        let component = mm.load_bezel(id)?;
        let mut lock = self.active_components.lock().unwrap();
        lock.insert(id.to_string(), component);
        Ok(())
    }

    pub fn deactivate_component(&self, id: &str) {
        let mut lock = self.active_components.lock().unwrap();
        lock.remove(id);
    }

    pub fn update_state(&self, state: &mut TosState) {
        let mut lock = self.active_components.lock().unwrap();
        let mut component_states = Vec::new();
        
        for (id, component) in lock.iter_mut() {
            let (html, data) = component.update(state);
            
            // Resolve slot from manifest if possible
            let slot = "right".to_string(); // Default to right lateral bezel
            
            component_states.push(BezelComponentState {
                id: id.clone(),
                name: component.name().to_string(),
                html,
                data,
                slot,
            });
        }
        
        state.active_bezel_components = component_states;
    }

    pub fn handle_click(&self, component_id: &str, element_id: &str, x: f32, y: f32) {
        let mut lock = self.active_components.lock().unwrap();
        if let Some(component) = lock.get_mut(component_id) {
            component.handle_click(element_id, x, y);
        }
    }
}

impl Default for BezelService {
    fn default() -> Self {
        Self::new()
    }
}
