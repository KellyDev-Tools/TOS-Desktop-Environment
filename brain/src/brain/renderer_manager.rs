use std::env;
use crate::platform::{Renderer, HeadlessRenderer, RemoteRenderer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RendererMode {
    Headless,
    Remote,
}

pub struct RendererManager;

impl RendererManager {
    /// Detect the appropriate renderer mode for the current environment.
    pub fn detect() -> RendererMode {
        // Default to headless for the daemon; remote if needed for streaming.
        if env::var("TOS_REMOTE").is_ok() {
            return RendererMode::Remote;
        }
        RendererMode::Headless
    }

    /// Initialize renderer for the detected mode.
    pub fn init(mode: RendererMode) -> anyhow::Result<Box<dyn Renderer + Send>> {
        match mode {
            RendererMode::Headless => {
                Ok(Box::new(HeadlessRenderer::new()))
            }
            RendererMode::Remote => {
                Ok(Box::new(RemoteRenderer))
            }
        }
    }
}
