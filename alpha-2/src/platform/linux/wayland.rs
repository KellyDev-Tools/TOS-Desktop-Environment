use std::collections::HashMap;

/// §15.2: Wayland Shell Management Layer (wlr-layer-shell implementation)
pub struct WaylandShell {
    layers: HashMap<u32, LayerSurface>,
    next_handle: u32,
}

pub struct LayerSurface {
    pub title: String,
    pub layer: u32,
    pub width: u32,
    pub height: u32,
}

impl WaylandShell {
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
            next_handle: 100, // Namespace separate from generic surfaces
        }
    }

    /// Registers a surface as a Layer Shell surface (e.g., Background, Top, Overlay)
    pub fn create_layer_surface(&mut self, title: &str, layer: u32, width: u32, height: u32) -> u32 {
        let handle = self.next_handle;
        self.next_handle += 1;
        
        tracing::info!("Wayland: Creating Layer Surface '{}' (Layer: {}, {}x{})", title, layer, width, height);
        
        self.layers.insert(handle, LayerSurface {
            title: title.to_string(),
            layer,
            width,
            height,
        });
        
        handle
    }
}
