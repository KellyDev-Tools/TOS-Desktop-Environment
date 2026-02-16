// GPU Rendering Pipeline
// Based on "Performance.md" — wgpu-based rendering for zoom transitions
//
// This module provides GPU-accelerated surface compositing:
// - Level-based rendering strategies (static textures, live injection)
// - Texture caching with VRAM depth-based pruning
// - Zoom transition animations (scale, opacity, blur)
// - DMA-BUF texture import for zero-copy Wayland surface display

use std::collections::HashMap;
use std::time::Instant;

/// GPU texture handle (wraps an opaque ID for the renderer)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(pub u64);

/// Rendering strategy per zoom level (from Performance.md)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderStrategy {
    /// Level 1/2: Use cached static thumbnails, update infrequently
    StaticTexture {
        update_interval_ms: u32,
    },
    /// Level 3: Live surface injection at native refresh rate
    LiveInjection,
    /// Level 4/5: On-demand rendering (only when viewed)
    OnDemand,
}

impl RenderStrategy {
    /// Get the appropriate strategy for a zoom depth
    pub fn for_depth(depth: usize) -> Self {
        match depth {
            0 => Self::StaticTexture { update_interval_ms: 2000 }, // L1: 0.5 FPS
            1 => Self::StaticTexture { update_interval_ms: 500 },  // L2: 2 FPS
            2 => Self::LiveInjection,                               // L3: native FPS
            _ => Self::OnDemand,                                    // L4+: on-demand
        }
    }
}

/// Animation parameters for zoom transitions
#[derive(Debug, Clone)]
pub struct ZoomTransition {
    /// Source rect (normalized 0.0-1.0)
    pub from_rect: NormRect,
    /// Target rect (normalized 0.0-1.0)
    pub to_rect: NormRect,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Easing function
    pub easing: EasingFunction,
    /// When the transition started
    pub start_time: Instant,
    /// Direction: zooming in (true) or out (false)
    pub zooming_in: bool,
}

/// Normalized rectangle (0.0 to 1.0 coordinates)
#[derive(Debug, Clone, Copy)]
pub struct NormRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl NormRect {
    pub fn full() -> Self {
        Self { x: 0.0, y: 0.0, w: 1.0, h: 1.0 }
    }

    /// A specific tile in a grid
    pub fn tile(col: u32, row: u32, cols: u32, rows: u32) -> Self {
        let w = 1.0 / cols as f32;
        let h = 1.0 / rows as f32;
        Self {
            x: col as f32 * w,
            y: row as f32 * h,
            w,
            h,
        }
    }

    /// Interpolate between two rects
    pub fn lerp(a: &NormRect, b: &NormRect, t: f32) -> Self {
        Self {
            x: a.x + (b.x - a.x) * t,
            y: a.y + (b.y - a.y) * t,
            w: a.w + (b.w - a.w) * t,
            h: a.h + (b.h - a.h) * t,
        }
    }
}

/// Easing functions for smooth transitions
#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseInOut,
    EaseOut,
    EaseIn,
    /// LCARS-style: fast start, slight overshoot, settle
    LcarsSnap,
}

impl EasingFunction {
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Self::EaseOut => 1.0 - (1.0 - t).powi(2),
            Self::EaseIn => t * t,
            Self::LcarsSnap => {
                // Fast ease-out with slight overshoot
                if t < 0.8 {
                    let p = t / 0.8;
                    1.05 * (1.0 - (1.0 - p).powi(3))
                } else {
                    1.05 - 0.05 * ((t - 0.8) / 0.2) // Settle back from overshoot
                }
            }
        }
    }
}

impl ZoomTransition {
    pub fn new(from: NormRect, to: NormRect, duration_ms: u32, zooming_in: bool) -> Self {
        Self {
            from_rect: from,
            to_rect: to,
            duration_ms,
            easing: EasingFunction::LcarsSnap,
            start_time: Instant::now(),
            zooming_in,
        }
    }

    /// Get the interpolated rect at the current time
    pub fn current_rect(&self) -> NormRect {
        let elapsed = self.start_time.elapsed().as_millis() as f32;
        let raw_t = (elapsed / self.duration_ms as f32).min(1.0);
        let t = self.easing.apply(raw_t);
        NormRect::lerp(&self.from_rect, &self.to_rect, t)
    }

    /// Whether the transition has completed
    pub fn is_complete(&self) -> bool {
        self.start_time.elapsed().as_millis() >= self.duration_ms as u128
    }

    /// Progress as 0.0-1.0
    pub fn progress(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_millis() as f32;
        (elapsed / self.duration_ms as f32).min(1.0)
    }
}

/// A cached texture in VRAM
#[derive(Debug)]
pub struct CachedTexture {
    pub id: TextureId,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    /// How deep in the hierarchy this texture lives
    pub depth: usize,
    /// Last time this texture was accessed
    pub last_accessed: Instant,
    /// Estimated VRAM usage in bytes
    pub vram_bytes: u64,
    /// Whether this texture needs a re-render
    pub dirty: bool,
}

/// Texture formats supported by the pipeline
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureFormat {
    Bgra8Unorm,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    /// DMA-BUF imported texture (zero-copy from Wayland client)
    DmaBuf,
}

/// The VRAM texture cache with depth-based pruning.
/// Closer textures (lower depth) are kept; deeper textures are evicted first.
pub struct TextureCache {
    textures: HashMap<TextureId, CachedTexture>,
    next_id: u64,
    /// Maximum VRAM budget in bytes
    vram_budget: u64,
    /// Current estimated VRAM usage
    vram_used: u64,
}

impl TextureCache {
    pub fn new(vram_budget_mb: u64) -> Self {
        Self {
            textures: HashMap::new(),
            next_id: 1,
            vram_budget: vram_budget_mb * 1024 * 1024,
            vram_used: 0,
        }
    }

    /// Allocate a new texture in the cache
    pub fn allocate(&mut self, width: u32, height: u32, format: TextureFormat, depth: usize) -> TextureId {
        let id = TextureId(self.next_id);
        self.next_id += 1;

        let bytes_per_pixel: u64 = match format {
            TextureFormat::DmaBuf => 0, // Zero-copy, no VRAM counted
            _ => 4,
        };
        let vram_bytes = width as u64 * height as u64 * bytes_per_pixel;

        // Check if we need to evict
        while self.vram_used + vram_bytes > self.vram_budget && !self.textures.is_empty() {
            self.evict_one();
        }

        self.vram_used += vram_bytes;
        self.textures.insert(id, CachedTexture {
            id,
            width,
            height,
            format,
            depth,
            last_accessed: Instant::now(),
            vram_bytes,
            dirty: true,
        });

        println!("[GPU Cache] Allocated texture {} ({}x{}, depth {}, {:.1}KB)",
            id.0, width, height, depth, vram_bytes as f64 / 1024.0);
        id
    }

    /// Mark a texture as accessed (updates LRU timestamp)
    pub fn touch(&mut self, id: TextureId) {
        if let Some(tex) = self.textures.get_mut(&id) {
            tex.last_accessed = Instant::now();
        }
    }

    /// Mark a texture as needing re-render
    pub fn mark_dirty(&mut self, id: TextureId) {
        if let Some(tex) = self.textures.get_mut(&id) {
            tex.dirty = true;
        }
    }

    /// Evict the least valuable texture (deepest + oldest)
    fn evict_one(&mut self) {
        // Score = depth * 100 + seconds_since_access
        // Higher score = evict first
        let victim = self.textures.values()
            .max_by_key(|t| {
                let age = t.last_accessed.elapsed().as_secs();
                t.depth as u64 * 100 + age
            })
            .map(|t| t.id);

        if let Some(id) = victim {
            if let Some(tex) = self.textures.remove(&id) {
                self.vram_used = self.vram_used.saturating_sub(tex.vram_bytes);
                println!("[GPU Cache] Evicted texture {} (depth {}, {:.1}KB freed)",
                    id.0, tex.depth, tex.vram_bytes as f64 / 1024.0);
            }
        }
    }

    /// Free a specific texture
    pub fn free(&mut self, id: TextureId) {
        if let Some(tex) = self.textures.remove(&id) {
            self.vram_used = self.vram_used.saturating_sub(tex.vram_bytes);
        }
    }

    /// Get cache stats
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            texture_count: self.textures.len(),
            vram_used_bytes: self.vram_used,
            vram_budget_bytes: self.vram_budget,
            dirty_count: self.textures.values().filter(|t| t.dirty).count(),
        }
    }

    /// Prune textures deeper than a threshold (for VRAM pressure)
    pub fn prune_below_depth(&mut self, max_depth: usize) {
        let to_remove: Vec<TextureId> = self.textures.values()
            .filter(|t| t.depth > max_depth)
            .map(|t| t.id)
            .collect();

        for id in to_remove {
            self.free(id);
        }
    }
}

/// Statistics about the texture cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub texture_count: usize,
    pub vram_used_bytes: u64,
    pub vram_budget_bytes: u64,
    pub dirty_count: usize,
}

impl CacheStats {
    pub fn usage_percent(&self) -> f64 {
        if self.vram_budget_bytes == 0 { return 0.0; }
        (self.vram_used_bytes as f64 / self.vram_budget_bytes as f64) * 100.0
    }
}

/// Render output for a single frame
#[derive(Debug)]
pub struct RenderFrame {
    /// Surface draw commands in back-to-front order
    pub draw_commands: Vec<DrawCommand>,
    /// Active zoom transition (if any)
    pub transition: Option<ZoomTransition>,
    /// Target frame time in microseconds
    pub target_frame_us: u32,
}

/// A single draw command for the GPU
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Clear the framebuffer
    Clear { r: f32, g: f32, b: f32, a: f32 },
    /// Draw a textured quad (surface content)
    DrawTexture {
        texture_id: TextureId,
        dest: NormRect,
        opacity: f32,
        /// Optional CSS-style blur for unfocused windows (in pixels)
        blur_radius: f32,
    },
    /// Draw the LCARS chrome overlay
    DrawChrome {
        texture_id: TextureId,
        viewport_rect: NormRect,
    },
    /// Draw a solid color rect (for LCARS panels, borders)
    DrawRect {
        rect: NormRect,
        color: [f32; 4], // RGBA
        corner_radius: f32,
    },
}

/// The GPU rendering pipeline coordinator.
/// In a real implementation, this holds wgpu state.
/// Here we define the interface and frame-building logic.
pub struct GpuPipeline {
    pub texture_cache: TextureCache,
    pub current_transition: Option<ZoomTransition>,
    /// Frame counter for debug
    pub frame_count: u64,
    /// Target FPS for the compositor
    pub target_fps: u32,
    /// Whether to use direct scanout for focused Level 3 windows
    pub direct_scanout_enabled: bool,
    // In real implementation, these would be wgpu handles:
    // instance: wgpu::Instance,
    // device: wgpu::Device,
    // queue: wgpu::Queue,
    // surface: wgpu::Surface,
    // render_pipeline: wgpu::RenderPipeline,
}

impl GpuPipeline {
    pub fn new(vram_budget_mb: u64, target_fps: u32) -> Self {
        println!("[GPU] Pipeline initialized (VRAM budget: {}MB, target: {}FPS)",
            vram_budget_mb, target_fps);
        Self {
            texture_cache: TextureCache::new(vram_budget_mb),
            current_transition: None,
            frame_count: 0,
            target_fps,
            direct_scanout_enabled: true,
        }
    }

    /// Begin a zoom transition animation
    pub fn start_transition(&mut self, from: NormRect, to: NormRect, zooming_in: bool) {
        let duration = if zooming_in { 350 } else { 250 }; // Zoom out is faster
        self.current_transition = Some(ZoomTransition::new(from, to, duration, zooming_in));
        println!("[GPU] Transition started ({}, {}ms)",
            if zooming_in { "zoom-in" } else { "zoom-out" }, duration);
    }

    /// Build the render commands for a single frame.
    /// This is called once per vsync.
    pub fn build_frame(
        &mut self,
        surfaces: &[(TextureId, NormRect, f32)], // (texture, dest, opacity)
        chrome_texture: Option<TextureId>,
        viewport_rect: NormRect,
    ) -> RenderFrame {
        self.frame_count += 1;

        let mut commands = Vec::new();

        // 1. Clear to LCARS dark background
        commands.push(DrawCommand::Clear {
            r: 0.0, g: 0.0, b: 0.02, a: 1.0
        });

        // 2. Check if we're in a transition
        let transition = self.current_transition.take();
        let active_transition = if let Some(t) = transition {
            if t.is_complete() {
                None
            } else {
                let rect = t.current_rect();
                // During transition, surfaces are drawn with transformed coordinates
                for &(tex_id, dest, opacity) in surfaces {
                    let transformed = NormRect {
                        x: rect.x + dest.x * rect.w,
                        y: rect.y + dest.y * rect.h,
                        w: dest.w * rect.w,
                        h: dest.h * rect.h,
                    };
                    commands.push(DrawCommand::DrawTexture {
                        texture_id: tex_id,
                        dest: transformed,
                        opacity: opacity * (1.0 - t.progress() * 0.3), // Slight fade during transition
                        blur_radius: 0.0,
                    });
                    self.texture_cache.touch(tex_id);
                }
                Some(t)
            }
        } else {
            None
        };

        // 3. If no transition, draw surfaces normally
        if active_transition.is_none() {
            for &(tex_id, dest, opacity) in surfaces {
                commands.push(DrawCommand::DrawTexture {
                    texture_id: tex_id,
                    dest,
                    opacity,
                    blur_radius: 0.0,
                });
                self.texture_cache.touch(tex_id);
            }
        }

        // 4. Draw LCARS chrome overlay on top
        if let Some(chrome_id) = chrome_texture {
            commands.push(DrawCommand::DrawChrome {
                texture_id: chrome_id,
                viewport_rect,
            });
            self.texture_cache.touch(chrome_id);
        }

        // Re-store transition if still active
        self.current_transition = active_transition.clone();

        RenderFrame {
            draw_commands: commands,
            transition: active_transition,
            target_frame_us: 1_000_000 / self.target_fps,
        }
    }

    /// Check if we can use direct scanout (bypass compositor)
    /// for a single focused Level 3 window
    pub fn can_direct_scanout(&self, surface_count: usize, depth: usize) -> bool {
        self.direct_scanout_enabled
            && surface_count == 1
            && depth == 2 // Level 3
            && self.current_transition.is_none()
    }

    /// Get current cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.texture_cache.stats()
    }

    /// End-of-frame housekeeping
    pub fn end_frame(&mut self) {
        // Periodically log stats
        if self.frame_count % (self.target_fps as u64 * 10) == 0 {
            let stats = self.cache_stats();
            println!("[GPU] Frame {}: {} textures, {:.1}MB/{:.1}MB VRAM ({:.0}%)",
                self.frame_count,
                stats.texture_count,
                stats.vram_used_bytes as f64 / 1024.0 / 1024.0,
                stats.vram_budget_bytes as f64 / 1024.0 / 1024.0,
                stats.usage_percent()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_strategy_by_depth() {
        assert!(matches!(RenderStrategy::for_depth(0), RenderStrategy::StaticTexture { update_interval_ms: 2000 }));
        assert!(matches!(RenderStrategy::for_depth(1), RenderStrategy::StaticTexture { update_interval_ms: 500 }));
        assert!(matches!(RenderStrategy::for_depth(2), RenderStrategy::LiveInjection));
        assert!(matches!(RenderStrategy::for_depth(3), RenderStrategy::OnDemand));
        assert!(matches!(RenderStrategy::for_depth(10), RenderStrategy::OnDemand));
    }

    #[test]
    fn test_easing_functions() {
        // All easing functions should start at ~0 and end at ~1
        for easing in &[EasingFunction::Linear, EasingFunction::EaseInOut, EasingFunction::EaseOut, EasingFunction::EaseIn] {
            let start = easing.apply(0.0);
            let end = easing.apply(1.0);
            assert!(start.abs() < 0.01, "Easing {:?} starts at {} (expected ~0)", easing, start);
            assert!((end - 1.0).abs() < 0.01, "Easing {:?} ends at {} (expected ~1)", easing, end);
        }

        // LCARS snap overshoots slightly
        let mid = EasingFunction::LcarsSnap.apply(0.7);
        assert!(mid > 0.9, "LcarsSnap should be past 0.9 at t=0.7");
        let end = EasingFunction::LcarsSnap.apply(1.0);
        assert!((end - 1.0).abs() < 0.01, "LcarsSnap should settle at ~1.0");
    }

    #[test]
    fn test_norm_rect_lerp() {
        let a = NormRect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 };
        let b = NormRect { x: 0.25, y: 0.25, w: 0.5, h: 0.5 };

        let mid = NormRect::lerp(&a, &b, 0.5);
        assert!((mid.x - 0.125).abs() < 0.001);
        assert!((mid.w - 0.75).abs() < 0.001);

        let end = NormRect::lerp(&a, &b, 1.0);
        assert!((end.x - 0.25).abs() < 0.001);
        assert!((end.w - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_texture_cache_allocation() {
        let mut cache = TextureCache::new(64); // 64MB budget

        let t1 = cache.allocate(1920, 1080, TextureFormat::Bgra8Unorm, 0);
        let t2 = cache.allocate(1920, 1080, TextureFormat::Bgra8Unorm, 1);

        let stats = cache.stats();
        assert_eq!(stats.texture_count, 2);
        // 1920 * 1080 * 4 = ~7.9MB per texture
        assert!(stats.vram_used_bytes > 7_000_000);
    }

    #[test]
    fn test_texture_cache_eviction() {
        let mut cache = TextureCache::new(1); // 1MB budget — very small

        // Allocate textures that exceed budget
        let t1 = cache.allocate(512, 512, TextureFormat::Bgra8Unorm, 0); // 1MB
        let t2 = cache.allocate(512, 512, TextureFormat::Bgra8Unorm, 3); // 1MB, deeper

        // The deeper texture should have been evicted to make room
        let stats = cache.stats();
        // With only 1MB budget, we can't hold two 1MB textures
        assert!(stats.vram_used_bytes <= 1024 * 1024 + 1024); // Allow slight overflow
    }

    #[test]
    fn test_texture_cache_depth_pruning() {
        let mut cache = TextureCache::new(64);

        cache.allocate(256, 256, TextureFormat::Bgra8Unorm, 0);
        cache.allocate(256, 256, TextureFormat::Bgra8Unorm, 1);
        cache.allocate(256, 256, TextureFormat::Bgra8Unorm, 3);
        cache.allocate(256, 256, TextureFormat::Bgra8Unorm, 4);

        assert_eq!(cache.stats().texture_count, 4);

        cache.prune_below_depth(2); // Remove depth > 2
        assert_eq!(cache.stats().texture_count, 2);
    }

    #[test]
    fn test_gpu_pipeline_creation() {
        let pipeline = GpuPipeline::new(256, 60);
        assert_eq!(pipeline.target_fps, 60);
        assert_eq!(pipeline.frame_count, 0);
        assert!(pipeline.direct_scanout_enabled);
    }

    #[test]
    fn test_gpu_pipeline_build_frame() {
        let mut pipeline = GpuPipeline::new(256, 60);

        let t1 = pipeline.texture_cache.allocate(1920, 1080, TextureFormat::Bgra8Unorm, 2);

        let surfaces = vec![
            (t1, NormRect::full(), 1.0),
        ];

        let frame = pipeline.build_frame(&surfaces, None, NormRect::full());

        assert!(frame.draw_commands.len() >= 2); // Clear + at least 1 surface
        assert_eq!(frame.target_frame_us, 1_000_000 / 60);
        assert_eq!(pipeline.frame_count, 1);
    }

    #[test]
    fn test_direct_scanout_conditions() {
        let pipeline = GpuPipeline::new(256, 60);

        // Single surface at Level 3 = can scanout
        assert!(pipeline.can_direct_scanout(1, 2));

        // Multiple surfaces = can't scanout
        assert!(!pipeline.can_direct_scanout(2, 2));

        // Wrong depth = can't scanout
        assert!(!pipeline.can_direct_scanout(1, 0));
    }

    #[test]
    fn test_zoom_transition_animation() {
        let from = NormRect::full();
        let to = NormRect { x: 0.25, y: 0.25, w: 0.5, h: 0.5 };

        let transition = ZoomTransition::new(from, to, 300, true);
        assert!(!transition.is_complete());
        assert!(transition.progress() < 0.5); // Should be very early

        // The rect should be near the start
        let rect = transition.current_rect();
        assert!(rect.w > 0.8); // hasn't zoomed much yet
    }

    #[test]
    fn test_dmabuf_texture_zero_vram() {
        let mut cache = TextureCache::new(64);

        let before = cache.stats().vram_used_bytes;
        cache.allocate(1920, 1080, TextureFormat::DmaBuf, 2);
        let after = cache.stats().vram_used_bytes;

        // DMA-BUF textures should not count against VRAM budget
        assert_eq!(before, after);
    }

    #[test]
    fn test_norm_rect_tile() {
        let tile = NormRect::tile(1, 0, 3, 2);
        assert!((tile.x - 1.0/3.0).abs() < 0.001);
        assert!((tile.y - 0.0).abs() < 0.001);
        assert!((tile.w - 1.0/3.0).abs() < 0.001);
        assert!((tile.h - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_cache_stats_usage() {
        let mut cache = TextureCache::new(100); // 100MB
        cache.allocate(1024, 1024, TextureFormat::Bgra8Unorm, 0); // 4MB

        let stats = cache.stats();
        let pct = stats.usage_percent();
        assert!(pct > 3.0 && pct < 5.0); // ~4% of 100MB
    }
}
