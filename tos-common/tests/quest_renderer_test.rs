use tos_common::platform::{Renderer, SurfaceConfig};
use tos_common::platform::quest::{QuestRenderer, CockpitLayer};

#[test]
fn test_quest_renderer_layer_resolution() {
    let mut renderer = QuestRenderer::new();
    
    // 1. Large surface should be MainCylinder
    let h1 = renderer.create_surface(SurfaceConfig {
        width: 3840,
        height: 1080,
        depth: 0,
    });
    assert_eq!(renderer.surfaces.get(&h1).unwrap().layer_type, CockpitLayer::MainCylinder);
    
    // 2. Surface with depth should be FloatingQuad
    let h2 = renderer.create_surface(SurfaceConfig {
        width: 400,
        height: 400,
        depth: 1,
    });
    assert_eq!(renderer.surfaces.get(&h2).unwrap().layer_type, CockpitLayer::FloatingQuad);
    
    // 3. Small surface without depth should be AppViewport
    let h3 = renderer.create_surface(SurfaceConfig {
        width: 800,
        height: 600,
        depth: 0,
    });
    assert_eq!(renderer.surfaces.get(&h3).unwrap().layer_type, CockpitLayer::AppViewport);
}

#[test]
fn test_quest_renderer_composition() {
    let mut renderer = QuestRenderer::new();
    renderer.create_surface(SurfaceConfig { width: 4000, height: 1000, depth: 0 }); // MainCylinder
    renderer.create_surface(SurfaceConfig { width: 500, height: 500, depth: 1 });  // FloatingQuad
    renderer.create_surface(SurfaceConfig { width: 800, height: 600, depth: 0 });  // AppViewport
    
    // This should not panic and should exercise the composition logic
    renderer.composite();
    
    assert_eq!(renderer.surfaces.len(), 3);
}
