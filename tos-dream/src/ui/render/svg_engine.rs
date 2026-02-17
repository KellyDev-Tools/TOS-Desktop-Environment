use crate::{TosState, HierarchyLevel, CommandHubMode};

pub fn render_state_to_svg(state: &TosState) -> String {
    let mut svg = String::from(r##"<svg width="1280" height="720" viewBox="0 0 1280 720" xmlns="http://www.w3.org/2000/svg" style="background: black; font-family: 'Outfit', sans-serif;">"##);
    
    // Define Gradients and Filters
    svg.push_str(r##"<defs>
        <linearGradient id="headerGrad" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" style="stop-color:white;stop-opacity:1" />
            <stop offset="100%" style="stop-color:white;stop-opacity:0.3" />
        </linearGradient>
        <filter id="bgBlur">
            <feGaussianBlur in="SourceGraphic" stdDeviation="10" />
        </filter>
        <filter id="glow">
            <feGaussianBlur stdDeviation="5" result="coloredBlur"/>
            <feMerge>
                <feMergeNode in="coloredBlur"/>
                <feMergeNode in="SourceGraphic"/>
            </feMerge>
        </filter>
    </defs>"##);

    // Background
    svg.push_str(r##"<rect width="1280" height="720" fill="#050505" />"##);
    
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
    svg.push_str(r##"<text x="100" y="80" fill="url(#headerGrad)" font-size="48" font-weight="800">TOS COMMAND CENTER</text>"##);
    
    // Telemetry Bar (Mock)
    svg.push_str(r##"<rect x="950" y="45" width="230" height="50" rx="15" fill="rgba(255,255,255,0.05)" stroke="rgba(255,255,255,0.1)" />"##);
    svg.push_str(r##"<text x="970" y="70" fill="#ff9900" font-size="10">STARDATE</text>"##);
    svg.push_str(r##"<text x="970" y="85" fill="white" font-size="14" font-weight="700">02-33 // 02-1478</text>"##);

    // Sectors
    let mut x = 100;
    let mut y = 140;
    for sector in &state.sectors {
        let color = if sector.color.is_empty() { "#ff9900" } else { &sector.color };
        
        // Card Body (Glassmorphism effect in SVG)
        svg.push_str(&format!(r##"<rect x="{}" y="{}" width="320" height="420" rx="20" fill="rgba(30,30,50,0.6)" stroke="rgba(255,255,255,0.05)" />"##, x, y));
        
        // Header
        svg.push_str(&format!(r##"<path d="M {} {}, {} {} a 20 20 0 0 1 20 -20 h 280 a 20 20 0 0 1 20 20 v 40 h -320 z" fill="{}" />"##, x, y+20, x, y, color));
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="black" font-size="12" font-weight="800">SECTOR</text>"##, x+20, y+35));
        
        // Icon
        svg.push_str(&format!(r##"<circle cx="{}" cy="{}" r="30" fill="rgba(0,0,0,0.2)" />"##, x+50, y+100));
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="{}" font-size="40" text-anchor="middle" filter="url(#glow)">⌨️</text>"##, x+50, y+115, color));
        
        // Name
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="white" font-size="24" font-weight="800">{}</text>"##, x+20, y+170, sector.name.to_uppercase()));
        
        // Stats
        svg.push_str(&format!(r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="rgba(255,255,255,0.1)" />"##, x+20, y+240, x+300, y+240));
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="rgba(255,255,255,0.4)" font-size="10">USERS</text>"##, x+20, y+260));
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="{}" font-size="18" font-weight="700">{}</text>"##, x+20, y+280, color, sector.participants.len()));
        
        // Action Buttons
        svg.push_str(&format!(r##"<rect x="{}" y="{}" width="80" height="30" rx="5" fill="{}" />"##, x+20, y+370, color));
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="black" font-size="12" font-weight="800" text-anchor="middle">ENTER</text>"##, x+60, y+390));
        
        svg.push_str(&format!(r##"<rect x="{}" y="{}" width="80" height="30" rx="5" fill="rgba(255,255,255,0.1)" stroke="rgba(255,255,255,0.2)" />"##, x+110, y+370));
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="white" font-size="12" font-weight="800" text-anchor="middle">SHARE</text>"##, x+150, y+390));
        
        x += 350;
        if x > 1100 {
            x = 100;
            y += 450;
        }
    }

    // Remote Card Placeholder
    svg.push_str(&format!(r##"<rect x="{}" y="{}" width="320" height="420" rx="20" fill="rgba(20,40,30,0.3)" stroke="rgba(153,204,153,0.3)" stroke-dasharray="5,5" />"##, x, y));
    svg.push_str(&format!(r##"<text x="{}" y="{}" fill="#99cc99" font-size="24" font-weight="800" text-anchor="middle">LINK NODE</text>"##, x+160, y+200));
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
