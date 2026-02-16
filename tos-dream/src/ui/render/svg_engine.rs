use crate::{TosState, Viewport, HierarchyLevel, CommandHubMode};

pub fn render_state_to_svg(state: &TosState) -> String {
    let mut svg = String::from(r##"<svg width="1280" height="720" viewBox="0 0 1280 720" xmlns="http://www.w3.org/2000/svg" style="background: black; font-family: sans-serif;">"##);
    
    // Background
    svg.push_str(r##"<rect width="1280" height="720" fill="black" />"##);
    
    match state.current_level {
        HierarchyLevel::GlobalOverview => {
            render_global_overview_svg(&mut svg, state);
        }
        HierarchyLevel::CommandHub => {
            render_command_hub_svg(&mut svg, state);
        }
        HierarchyLevel::ApplicationFocus => {
            render_app_focus_svg(&mut svg, state);
        }
        HierarchyLevel::SplitView => {
            svg.push_str(r##"<text x="640" y="360" fill="white" text-anchor="middle" font-size="24">SPLIT VIEW (GRID)</text>"##);
        }
        _ => {
            svg.push_str(r##"<text x="640" y="360" fill="white" text-anchor="middle" font-size="24">TOS SYSTEM STATE</text>"##);
        }
    }
    
    svg.push_str("</svg>");
    svg
}

fn render_global_overview_svg(svg: &mut String, state: &TosState) {
    // Title
    svg.push_str(r##"<text x="40" y="60" fill="#9999cc" font-size="36" font-weight="bold">TOS COMMAND CENTER</text>"##);
    
    // Sectors
    let mut x = 40;
    let mut y = 120;
    for sector in &state.sectors {
        let color = if sector.color.is_empty() { "#ff9900" } else { &sector.color };
        
        // Card Border
        svg.push_str(&format!(r##"<rect x="{}" y="{}" width="250" height="350" rx="15" fill="#111" stroke="{}" stroke-width="2" />"##, x, y, color));
        
        // Header
        svg.push_str(&format!(r##"<path d="M {} {}, {} {} a 15 15 0 0 1 15 -15 h 220 a 15 15 0 0 1 15 15 v 30 h -250 z" fill="{}" />"##, x, y+15, x, y, color));
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="black" font-size="14" font-weight="bold">SECTOR</text>"##, x+10, y+25));
        
        // Content
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="white" font-size="18" font-weight="bold">{}</text>"##, x+10, y+180, sector.name.to_uppercase()));
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="#aaa" font-size="12">{}</text>"##, x+10, y+210, sector.host));
        
        x += 280;
        if x > 1000 {
            x = 40;
            y += 380;
        }
    }
}

fn render_command_hub_svg(svg: &mut String, state: &TosState) {
    let viewport = &state.viewports[0];
    let sector = &state.sectors[viewport.sector_index];
    let hub = &sector.hubs[viewport.hub_index];
    
    // Side Bar (LCARS Elbow)
    svg.push_str(r##"<path d="M 0 100 h 150 a 30 30 0 0 1 30 30 v 400 a 30 30 0 0 1 -30 30 h -150 v -100 h 100 a 10 10 0 0 0 10 -10 v -260 a 10 10 0 0 0 -10 -10 h -100 z" fill="#9999cc" />"##);
    
    // Mode Indicators
    let (c1, c2, c3) = match hub.mode {
        CommandHubMode::Command => ("#ff9900", "#444", "#444"),
        CommandHubMode::Directory => ("#444", "#9999cc", "#444"),
        CommandHubMode::Activity => ("#444", "#444", "#cc99cc"),
    };
    
    svg.push_str(&format!(r##"<rect x="10" y="140" width="120" height="40" rx="20" fill="{}" />"##, c1));
    svg.push_str(r##"<text x="70" y="165" fill="black" text-anchor="middle" font-size="14">COMMAND</text>"##);
    
    svg.push_str(&format!(r##"<rect x="10" y="190" width="120" height="40" rx="20" fill="{}" />"##, c2));
    svg.push_str(r##"<text x="70" y="215" fill="black" text-anchor="middle" font-size="14">DIRECTORY</text>"##);
    
    svg.push_str(&format!(r##"<rect x="10" y="240" width="120" height="40" rx="20" fill="{}" />"##, c3));
    svg.push_str(r##"<text x="70" y="265" fill="black" text-anchor="middle" font-size="14">ACTIVITY</text>"##);
    
    // Main Content Area
    svg.push_str(r##"<rect x="200" y="100" width="1000" height="500" rx="10" fill="#080808" stroke="#333" stroke-width="1" />"##);
    svg.push_str(&format!(r##"<text x="220" y="140" fill="#9999cc" font-size="24" font-weight="bold">SECTOR: {}</text>"##, sector.name.to_uppercase()));
    
    // Prompt
    svg.push_str(r##"<rect x="200" y="620" width="1000" height="60" rx="30" fill="#111" stroke="#ff9900" stroke-width="2" />"##);
    svg.push_str(&format!(r##"<text x="240" y="658" fill="#ff9900" font-family="monospace" font-size="20">> {}_</text>"##, hub.prompt));
}

fn render_app_focus_svg(svg: &mut String, state: &TosState) {
    let viewport = &state.viewports[0];
    
    // App Surface
    svg.push_str(r##"<rect x="40" y="80" width="1200" height="600" fill="#050505" stroke="#222" />"##);
    svg.push_str(r##"<text x="640" y="380" fill="#333" text-anchor="middle" font-size="48">APPLICATION SURFACE</text>"##);
    
    // Bezel
    let bezel_h = if viewport.bezel_expanded { 200 } else { 40 };
    svg.push_str(&format!(r##"<rect x="0" y="0" width="1280" height="{}" fill="#111" opacity="0.9" />"##, bezel_h));
    svg.push_str(r##"<rect x="0" y="0" width="1280" height="4" fill="#9999cc" />"##);
    
    svg.push_str(r##"<text x="40" y="28" fill="#9999cc" font-size="18" font-weight="bold">TOS // SYSTEM BEZEL</text>"##);
    
    if viewport.bezel_expanded {
        // Buttons
        svg.push_str(r##"<rect x="40" y="60" width="120" height="40" rx="5" fill="#333" stroke="#9999cc" />"##);
        svg.push_str(r##"<text x="100" y="85" fill="#9999cc" text-anchor="middle" font-size="14">ZOOM OUT</text>"##);
        
        svg.push_str(r##"<rect x="180" y="60" width="120" height="40" rx="5" fill="#333" stroke="#66cc66" />"##);
        svg.push_str(r##"<text x="240" y="85" fill="#66cc66" text-anchor="middle" font-size="14">SPLIT VIEW</text>"##);
    }
}
