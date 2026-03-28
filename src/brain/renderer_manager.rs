//! Renderer mode detection and initialization.
//! 
//! Per Architecture §15.2–§15.5, the Brain must support three rendering modes:
//! - LocalWayland: Direct Wayland compositor available
//! - Headless: No GPU/compositor; buffers in CPU RAM
//! - Remote: No local render; stream to remote Face via WebRTC

use std::env;
use crate::platform::{Renderer, HeadlessRenderer, RemoteRenderer};
#[cfg(target_os = "linux")]
use crate::platform::linux::wayland::WaylandShell;
#[cfg(target_os = "linux")]
use crate::platform::linux::LinuxRenderer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RendererMode {
    LocalWayland,
    Headless,
    Remote,
}

pub struct RendererManager;

impl RendererManager {
    /// Detect the appropriate renderer mode for the current environment.
    pub fn detect() -> RendererMode {
        // Priority order: explicit flag > environment detection > default
        if env::var("TOS_HEADLESS").is_ok() || env::args().any(|arg| arg == "--headless") {
            return RendererMode::Headless;
        }

        #[cfg(target_os = "linux")]
        {
            if WaylandShell::can_connect() {
                return RendererMode::LocalWayland;
            }
        }

        // Default to remote/streaming fallback
        RendererMode::Remote
    }

    /// Initialize renderer for the detected mode.
    pub fn init(mode: RendererMode) -> anyhow::Result<Box<dyn Renderer + Send>> {
        match mode {
            RendererMode::LocalWayland => {
                #[cfg(target_os = "linux")]
                {
                    Ok(Box::new(LinuxRenderer::new()))
                }
                #[cfg(not(target_os = "linux"))]
                {
                    anyhow::bail!("LocalWayland mode only supported on Linux")
                }
            }
            RendererMode::Headless => {
                Ok(Box::new(HeadlessRenderer::new()))
            }
            RendererMode::Remote => {
                Ok(Box::new(RemoteRenderer))
            }
        }
    }
}
