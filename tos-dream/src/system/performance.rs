//! Phase 15: Performance Monitoring & Tactical Alerts
//!
//! Implements FPS monitoring with sustained threshold detection and
//! non-intrusive performance alerts with corrective action suggestions.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Configuration for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Target FPS (default: 60.0)
    pub target_fps: f32,
    /// Alert threshold FPS (default: 30.0)
    pub alert_threshold_fps: f32,
    /// Duration of sustained low FPS before alerting (default: 2 seconds)
    pub alert_duration: Duration,
    /// Whether to show corrective action suggestions
    pub show_suggestions: bool,
    /// Maximum history samples to keep
    pub max_history_samples: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            target_fps: 60.0,
            alert_threshold_fps: 30.0,
            alert_duration: Duration::from_secs(2),
            show_suggestions: true,
            max_history_samples: 300, // 5 seconds at 60 FPS
        }
    }
}

/// A single FPS sample with timestamp
#[derive(Debug, Clone, Copy)]
pub struct FpsSample {
    pub fps: f32,
    pub timestamp: Instant,
}

/// Performance monitoring state
#[derive(Debug)]
pub struct PerformanceMonitor {
    config: PerformanceConfig,
    history: VecDeque<FpsSample>,
    alert_start: Option<Instant>,
    alert_active: bool,
    last_alert_time: Option<Instant>,
    total_frames: u64,
    last_frame_time: Instant,
}

/// Suggested corrective actions when performance drops
#[derive(Debug, Clone)]
pub enum CorrectiveAction {
    CloseUnusedApplications,
    ReduceSplitViewports,
    DisableVisualEffects,
    LowerTextureQuality,
    CloseBackgroundSectors,
    EnablePerformanceMode,
    CheckForRunawayProcesses,
}

impl CorrectiveAction {
    pub fn description(&self) -> &'static str {
        match self {
            Self::CloseUnusedApplications => "Close unused applications",
            Self::ReduceSplitViewports => "Reduce number of split viewports",
            Self::DisableVisualEffects => "Disable visual effects and animations",
            Self::LowerTextureQuality => "Lower texture quality in background levels",
            Self::CloseBackgroundSectors => "Close inactive background sectors",
            Self::EnablePerformanceMode => "Enable performance mode (reduced features)",
            Self::CheckForRunawayProcesses => "Check for runaway processes",
        }
    }
    
    pub fn priority(&self) -> u8 {
        match self {
            Self::CloseUnusedApplications => 1,
            Self::ReduceSplitViewports => 2,
            Self::DisableVisualEffects => 3,
            Self::LowerTextureQuality => 4,
            Self::CloseBackgroundSectors => 5,
            Self::EnablePerformanceMode => 6,
            Self::CheckForRunawayProcesses => 7,
        }
    }
}

/// Current performance status
#[derive(Debug, Clone)]
pub struct PerformanceStatus {
    pub current_fps: f32,
    pub average_fps: f32,
    pub min_fps: f32,
    pub max_fps: f32,
    pub alert_active: bool,
    pub alert_duration: Option<Duration>,
    pub suggestions: Vec<CorrectiveAction>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor with default configuration
    pub fn new() -> Self {
        Self::with_config(PerformanceConfig::default())
    }
    
    /// Create a new performance monitor with custom configuration
    pub fn with_config(config: PerformanceConfig) -> Self {
        Self {
            config,
            history: VecDeque::with_capacity(300),
            alert_start: None,
            alert_active: false,
            last_alert_time: None,
            total_frames: 0,
            last_frame_time: Instant::now(),
        }
    }
    
    /// Record a frame and update FPS calculations
    pub fn record_frame(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        
        // Calculate instantaneous FPS
        let fps = if delta.as_secs_f32() > 0.0 {
            1.0 / delta.as_secs_f32()
        } else {
            self.config.target_fps
        };
        
        // Clamp to reasonable values
        let fps = fps.min(999.0).max(1.0);
        
        // Add to history
        self.history.push_back(FpsSample { fps, timestamp: now });
        if self.history.len() > self.config.max_history_samples {
            self.history.pop_front();
        }
        
        self.total_frames += 1;
        
        // Check alert conditions
        self.update_alert_state(now, fps);
        
        fps
    }
    
    /// Update the alert state based on current FPS
    fn update_alert_state(&mut self, now: Instant, current_fps: f32) {
        if current_fps < self.config.alert_threshold_fps {
            // FPS is below threshold
            if self.alert_start.is_none() {
                // Start tracking low FPS duration
                self.alert_start = Some(now);
            } else if let Some(start) = self.alert_start {
                let duration = now.duration_since(start);
                if duration >= self.config.alert_duration && !self.alert_active {
                    // Sustained low FPS - activate alert
                    self.alert_active = true;
                    self.last_alert_time = Some(now);
                }
            }
        } else {
            // FPS is above threshold - clear alert
            self.alert_start = None;
            self.alert_active = false;
        }
    }
    
    /// Get current performance status
    pub fn get_status(&self) -> PerformanceStatus {
        let current_fps = self.history.back()
            .map(|s| s.fps)
            .unwrap_or(self.config.target_fps);
        
        let (avg, min, max) = if self.history.is_empty() {
            (current_fps, current_fps, current_fps)
        } else {
            let sum: f32 = self.history.iter().map(|s| s.fps).sum();
            let avg = sum / self.history.len() as f32;
            let min = self.history.iter().map(|s| s.fps).fold(f32::INFINITY, f32::min);
            let max = self.history.iter().map(|s| s.fps).fold(0.0f32, f32::max);
            (avg, min, max)
        };
        
        let alert_duration = self.alert_start.map(|start| {
            Instant::now().duration_since(start)
        });
        
        let suggestions = if self.alert_active && self.config.show_suggestions {
            self.generate_suggestions()
        } else {
            Vec::new()
        };
        
        PerformanceStatus {
            current_fps,
            average_fps: avg,
            min_fps: min,
            max_fps: max,
            alert_active: self.alert_active,
            alert_duration,
            suggestions,
        }
    }
    
    /// Generate corrective action suggestions based on current state
    fn generate_suggestions(&self) -> Vec<CorrectiveAction> {
        let mut suggestions = Vec::new();
        
        // Basic suggestions always shown when alert is active
        suggestions.push(CorrectiveAction::CloseUnusedApplications);
        suggestions.push(CorrectiveAction::ReduceSplitViewports);
        suggestions.push(CorrectiveAction::DisableVisualEffects);
        
        // Additional suggestions based on history analysis
        if self.history.len() >= 60 {
            let recent_avg: f32 = self.history.iter()
                .rev()
                .take(60)
                .map(|s| s.fps)
                .sum::<f32>() / 60.0;
            
            if recent_avg < 20.0 {
                suggestions.push(CorrectiveAction::EnablePerformanceMode);
                suggestions.push(CorrectiveAction::CheckForRunawayProcesses);
            }
            
            if recent_avg < 15.0 {
                suggestions.push(CorrectiveAction::CloseBackgroundSectors);
                suggestions.push(CorrectiveAction::LowerTextureQuality);
            }
        }
        
        // Sort by priority
        suggestions.sort_by_key(|a| a.priority());
        suggestions.dedup_by(|a, b| a.priority() == b.priority());
        
        suggestions
    }
    
    /// Check if an alert is currently active
    pub fn is_alert_active(&self) -> bool {
        self.alert_active
    }
    
    /// Get the time since the last alert (if any)
    pub fn time_since_last_alert(&self) -> Option<Duration> {
        self.last_alert_time.map(|t| Instant::now().duration_since(t))
    }
    
    /// Reset the alert state (dismiss current alert)
    pub fn dismiss_alert(&mut self) {
        self.alert_active = false;
        self.alert_start = None;
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: PerformanceConfig) {
        self.config = config;
        // Resize history if needed
        if self.history.capacity() < self.config.max_history_samples {
            self.history.reserve(self.config.max_history_samples - self.history.capacity());
        }
    }
    
    /// Get current configuration
    pub fn config(&self) -> &PerformanceConfig {
        &self.config
    }
    
    /// Get total frames recorded
    pub fn total_frames(&self) -> u64 {
        self.total_frames
    }
    
    /// Calculate 1% and 0.1% low FPS (percentile lows)
    pub fn percentile_lows(&self) -> Option<(f32, f32)> {
        if self.history.len() < 100 {
            return None;
        }
        
        let mut fps_values: Vec<f32> = self.history.iter().map(|s| s.fps).collect();
        fps_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let one_percent_idx = (fps_values.len() as f32 * 0.01) as usize;
        let point_one_percent_idx = (fps_values.len() as f32 * 0.001) as usize;
        
        let one_percent_low = fps_values.get(one_percent_idx).copied();
        let point_one_percent_low = fps_values.get(point_one_percent_idx.max(1)).copied();
        
        one_percent_low.zip(point_one_percent_low)
    }
    
    /// Render performance overlay HTML
    pub fn render_overlay(&self) -> String {
        let status = self.get_status();
        
        let alert_class = if status.alert_active {
            "performance-alert-active"
        } else {
            "performance-alert-inactive"
        };
        
        let suggestions_html = if status.suggestions.is_empty() {
            String::new()
        } else {
            let items: String = status.suggestions.iter()
                .map(|s| format!("<li>{}</li>", s.description()))
                .collect();
            format!(
                r#"<div class="suggestions">
                    <h4>Suggested Actions:</h4>
                    <ul>{}</ul>
                </div>"#,
                items
            )
        };
        
        format!(
            r#"<div class="performance-overlay {}">
                <div class="fps-display">
                    <span class="fps-value">{:.1}</span>
                    <span class="fps-label">FPS</span>
                </div>
                <div class="fps-stats">
                    <div>Target: {:.0}</div>
                    <div>Avg: {:.1}</div>
                    <div>Min: {:.1}</div>
                    <div>Max: {:.1}</div>
                </div>
                {}
            </div>"#,
            alert_class,
            status.current_fps,
            self.config.target_fps,
            status.average_fps,
            status.min_fps,
            status.max_fps,
            suggestions_html
        )
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    
    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert!(!monitor.is_alert_active());
        assert_eq!(monitor.total_frames(), 0);
    }
    
    #[test]
    fn test_fps_recording() {
        let mut monitor = PerformanceMonitor::new();
        
        // Record some frames
        for _ in 0..10 {
            monitor.record_frame();
            sleep(Duration::from_millis(16)); // ~60 FPS
        }
        
        assert_eq!(monitor.total_frames(), 10);
        assert!(monitor.get_status().current_fps > 0.0);
    }
    
    #[test]
    fn test_alert_threshold() {
        let config = PerformanceConfig {
            target_fps: 60.0,
            alert_threshold_fps: 50.0,
            alert_duration: Duration::from_millis(100),
            show_suggestions: true,
            max_history_samples: 100,
        };
        
        let mut monitor = PerformanceMonitor::with_config(config);
        
        // Record frames at low FPS
        for _ in 0..20 {
            monitor.record_frame();
            sleep(Duration::from_millis(50)); // ~20 FPS
        }
        
        // Should have triggered alert
        assert!(monitor.is_alert_active() || monitor.get_status().alert_duration.is_some());
    }
    
    #[test]
    fn test_suggestions_generation() {
        let mut monitor = PerformanceMonitor::new();
        
        // Force alert state by manipulating history
        for i in 0..100 {
            monitor.history.push_back(FpsSample {
                fps: 10.0, // Very low FPS
                timestamp: Instant::now() - Duration::from_millis(i as u64 * 16),
            });
        }
        
        monitor.alert_active = true;
        
        let status = monitor.get_status();
        assert!(!status.suggestions.is_empty());
        
        // Should include high-priority suggestions
        let descriptions: Vec<_> = status.suggestions.iter()
            .map(|s| s.description())
            .collect();
        assert!(descriptions.contains(&"Close unused applications"));
    }
    
    #[test]
    fn test_dismiss_alert() {
        let mut monitor = PerformanceMonitor::new();
        monitor.alert_active = true;
        monitor.alert_start = Some(Instant::now());
        
        monitor.dismiss_alert();
        
        assert!(!monitor.is_alert_active());
        assert!(monitor.alert_start.is_none());
    }
    
    #[test]
    fn test_percentile_lows() {
        let mut monitor = PerformanceMonitor::new();
        
        // Add varied FPS samples
        for i in 0..200 {
            let fps = if i % 10 == 0 { 10.0 } else { 60.0 };
            monitor.history.push_back(FpsSample {
                fps,
                timestamp: Instant::now(),
            });
        }
        
        let (one_percent, point_one_percent) = monitor.percentile_lows().unwrap();
        assert!(one_percent <= 60.0);
        assert!(point_one_percent <= one_percent);
    }
    
    #[test]
    fn test_render_overlay() {
        let monitor = PerformanceMonitor::new();
        let html = monitor.render_overlay();
        
        assert!(html.contains("performance-overlay"));
        assert!(html.contains("FPS"));
    }
}
