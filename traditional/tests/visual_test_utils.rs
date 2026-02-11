// Test utilities for visual debugging with dev monitor
#![cfg(feature = "dev-monitor")]

use tos_comp::{DesktopEnvironment, UiCommand};
use tos_comp::dev_monitor::{get_monitor, DevMonitorBroadcaster};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::Duration;

/// Test environment wrapper that automatically broadcasts to dev monitor
pub struct VisualTestEnv {
    pub env: DesktopEnvironment,
    pub tx: Sender<UiCommand>,
    pub rx: Receiver<UiCommand>,
    pub test_name: String,
    step_count: usize,
}

impl VisualTestEnv {
    pub fn new(test_name: impl Into<String>) -> Self {
        let (tx, rx) = channel::<UiCommand>();
        let mut env = DesktopEnvironment::new(Some(tx.clone()));
        
        let test_name = test_name.into();
        
        // Announce test start
        if let Some(monitor) = get_monitor() {
            monitor.test_event(&test_name, "started", "Test initialized");
        }
        
        Self {
            env,
            tx,
            rx,
            test_name,
            step_count: 0,
        }
    }

    /// Execute a step and broadcast to monitor
    pub fn step(&mut self, description: impl Into<String>) {
        self.step_count += 1;
        let desc = description.into();
        
        if let Some(monitor) = get_monitor() {
            monitor.test_event(&self.test_name, "step", &format!("Step {}: {}", self.step_count, desc));
        }
        
        // Small delay so browser can render
        std::thread::sleep(Duration::from_millis(300));
    }

    /// Assert something and broadcast to monitor
    pub fn assert(&mut self, condition: bool, message: impl Into<String>) {
        let msg = message.into();
        
        if let Some(monitor) = get_monitor() {
            let status = if condition { "✓ PASS" } else { "✗ FAIL" };
            monitor.test_event(&self.test_name, "assertion", &format!("{}: {}", status, msg));
        }
        
        assert!(condition, "{}", msg);
        std::thread::sleep(Duration::from_millis(200));
    }

    /// Update viewport and broadcast
    pub fn update_viewport(&mut self) {
        let html = self.env.generate_viewport_html();
        let zoom: u8 = match self.env.navigator.current_level {
            tos_comp::navigation::zoom::ZoomLevel::Level1Root => 1,
            tos_comp::navigation::zoom::ZoomLevel::Level2Sector => 2,
            tos_comp::navigation::zoom::ZoomLevel::Level3Focus => 3,
            tos_comp::navigation::zoom::ZoomLevel::Level3aPicker => 3,
            tos_comp::navigation::zoom::ZoomLevel::Level3Split => 3,
            tos_comp::navigation::zoom::ZoomLevel::Level4Detail => 4,
            tos_comp::navigation::zoom::ZoomLevel::Level5Buffer => 5,
        };
        
        if let Some(monitor) = get_monitor() {
            monitor.update_viewport(html.clone(), zoom, self.env.is_red_alert);
        }
        
        let _ = self.tx.send(UiCommand::UpdateViewport {
            html_content: html,
            zoom_level: zoom,
            is_red_alert: self.env.is_red_alert,
        });
        
        std::thread::sleep(Duration::from_millis(100));
    }

    /// Update dashboard and broadcast
    pub fn update_dashboard(&mut self) {
        let html = self.env.dashboard.render_all_html();
        
        if let Some(monitor) = get_monitor() {
            monitor.update_dashboard(html.clone());
        }
        
        let _ = self.tx.send(UiCommand::UpdateDashboard(html));
        std::thread::sleep(Duration::from_millis(100));
    }

    /// Finish test
    pub fn finish(&self) {
        if let Some(monitor) = get_monitor() {
            monitor.test_event(&self.test_name, "completed", &format!("✓ Test completed after {} steps", self.step_count));
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

/// Macro to create a visual test
#[macro_export]
macro_rules! visual_test {
    ($name:ident, $body:expr) => {
        #[test]
        #[ignore] // Only run when explicitly requested with --include-ignored
        fn $name() {
            let mut vt = VisualTestEnv::new(stringify!($name));
            $body(&mut vt);
            vt.finish();
        }
    };
}
