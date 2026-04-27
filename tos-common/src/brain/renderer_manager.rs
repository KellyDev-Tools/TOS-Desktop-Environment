use crate::platform::{HeadlessRenderer, RemoteRenderer, Renderer};
use std::env;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RendererMode {
    Headless,
    Remote,
    Quest,
}

pub struct RendererManager;

impl RendererManager {
    /// Detect the appropriate renderer mode for the current environment.
    pub fn detect() -> RendererMode {
        // 1. Explicit overrides
        if env::var("TOS_HEADLESS").is_ok() {
            return RendererMode::Headless;
        }
        if env::var("TOS_XR").is_ok() {
            return RendererMode::Quest;
        }
        if env::var("TOS_REMOTE").is_ok() {
            return RendererMode::Remote;
        }

        // 2. Default to headless for the daemon.
        RendererMode::Headless
    }

    /// Initialize renderer for the detected mode.
    pub fn init(mode: RendererMode) -> anyhow::Result<Box<dyn Renderer + Send>> {
        match mode {
            RendererMode::Headless => Ok(Box::new(HeadlessRenderer::new())),
            RendererMode::Remote => Ok(Box::new(RemoteRenderer)),
            RendererMode::Quest => Ok(Box::new(crate::platform::quest::QuestRenderer::new())),
        }
    }
}
