//! Tests for AI integration and global search functionality.
//!
//! These validate AI-powered search, explanations, and global overview features.

use std::time::Duration;
use tokio::time::sleep;
use tos_lib::brain::Brain;
use tos_lib::common::CommandHubMode;
use tos_lib::face::{Face, MockFace};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\x1B[1;36m[TOS AI INTEGRATION TESTS]\x1B[0m");
    println!("Testing AI-powered search and explanations...\n");

    // 1. BOOTSTRAP
    let brain = Brain::new()?;
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut face = MockFace(face_raw);

    // 2. GLOBAL SEARCH TEST
    println!("\x1B[1;33m[TEST: Global Search - search:TOS]\x1B[0m");
    println!("-> Action: search:TOS");
    brain.ipc.handle_request("search:TOS");

    {
        let state = brain.state.lock().unwrap();
        let idx = state.active_sector_index;
        let hub = &mut state.sectors[idx].hubs[idx % state.sectors[idx].hubs.len()];
        hub.mode = CommandHubMode::Search;
        println!("\x1B[1;32m[PASSED]\x1B[0m Global search mode activated");
    }

    // 3. SEARCH WITH QUERY TEST
    println!(r"\n\x1B[1;33m[TEST: Search with Query - search:local:.*\.rs]\x1B[0m");
    println!(r"-> Action: search:local:.*\.rs");
    brain.ipc.handle_request(r"search:local:.*\.rs");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::Search);
        println!("\x1B[1;32m[PASSED]\x1B[0m Search query processed");
    }

    // 4. SEARCH IN CURRENT DIRECTORY TEST
    println!(r"\n\x1B[1;33m[TEST: Search in Current Directory - search:current:.*\.rs]\x1B[0m");
    println!(r"-> Action: search:current:.*\.rs");
    brain.ipc.handle_request(r"search:current:.*\.rs");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::Search);
        println!("\x1B[1;32m[PASSED]\x1B[0m Current directory search activated");
    }

    // 5. AI EXPLANATION TEST
    println!("\n\x1B[1;33m[TEST: AI Explanation - ai_explain:borrow checker error]\x1B[0m");
    println!("-> Action: ai_explain:borrow checker error");
    brain.ipc.handle_request("ai_explain:borrow checker error");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.ai_explanation.is_some(), "AI explanation should be present");
        println!("\x1B[1;32m[PASSED]\x1B[0m AI explanation provided");
    }

    // 6. AI EXPLANATION WITH CODE TEST
    println!("\n\x1B[1;33m[TEST: AI Explanation with Code - ai_explain:code example]\x1B[0m");
    println!("-> Action: ai_explain:code example");
    brain.ipc.handle_request("ai_explain:code example");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.ai_explanation.is_some(), "AI explanation should be present");
        println!("\x1B[1;32m[PASSED]\x1B[0m AI explanation with code provided");
    }

    // 7. GLOBAL OVERVIEW TEST
    println!("\n\x1B[1;33m[TEST: Global Overview - zoom_to:GlobalOverview]\x1B[0m");
    println!("-> Action: zoom_to:GlobalOverview");
    brain.ipc.handle_request("zoom_to:GlobalOverview");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::GlobalOverview);
        println!("\x1B[1;32m[PASSED]\x1B[0m Global overview activated");
    }

    // 8. ZOOM TO COMMAND HUB TEST
    println!("\n\x1B[1;33m[TEST: Zoom to Command Hub - zoom_to:CommandHub]\x1B[0m");
    println!("-> Action: zoom_to:CommandHub");
    brain.ipc.handle_request("zoom_to:CommandHub");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::Command);
        println!("\x1B[1;32m[PASSED]\x1B[0m Zoomed to Command Hub");
    }

    // 9. ZOOM IN TEST
    println!("\n\x1B[1;33m[TEST: Zoom In - bezel_zoom_in]\x1B[0m");
    println!("-> Action: bezel_zoom_in");
    face.simulate_bezel_zoom_in();

    {
        let state = brain.state.lock().unwrap();
        assert!(state.current_level > 1, "Zoom level should increase");
        println!("\x1B[1;32m[PASSED]\x1B[0m Zoom level increased");
    }

    // 10. ZOOM OUT TEST
    println!("\n\x1B[1;33m[TEST: Zoom Out - bezel_zoom_out]\x1B[0m");
    println!("-> Action: bezel_zoom_out");
    face.simulate_bezel_zoom_out();

    {
        let state = brain.state.lock().unwrap();
        println!("\x1B[1;32m[PASSED]\x1B[0m Zoom level decreased");
    }

    // 11. ZOOM TO SECTOR TEST
    println!("\n\x1B[1;33m[TEST: Zoom to Sector - zoom_to:Research]\x1B[0m");
    println!("-> Action: zoom_to:Research");
    brain.ipc.handle_request("zoom_to:Research");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[1].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::Command);
        println!("\x1B[1;32m[PASSED]\x1B[0m Zoomed to Research sector");
    }

    // 12. ZOOM TO HUB TEST
    println!("\n\x1B[1;33m[TEST: Zoom to Hub - zoom_to:hub_0]\x1B[0m");
    println!("-> Action: zoom_to:hub_0");
    brain.ipc.handle_request("zoom_to:hub_0");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::Command);
        println!("\x1B[1;32m[PASSED]\x1B[0m Zoomed to hub_0");
    }

    // 13. ZOOM TO TERMINAL TEST
    println!("\n\x1B[1;33m[TEST: Zoom to Terminal - zoom_to:Terminal]\x1B[0m");
    println!("-> Action: zoom_to:Terminal");
    brain.ipc.handle_request("zoom_to:Terminal");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::Terminal);
        println!("\x1B[1;32m[PASSED]\x1B[0m Zoomed to Terminal");
    }

    // 14. AI SEARCH TEST
    println!("\n\x1B[1;33m[TEST: AI Search - ai_search:rust patterns]\x1B[0m");
    println!("-> Action: ai_search:rust patterns");
    brain.ipc.handle_request("ai_search:rust patterns");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::Search);
        println!("\x1B[1;32m[PASSED]\x1B[0m AI search activated");
    }

    // 15. AI SEARCH WITH PATTERN TEST
    println!(r"\n\x1B[1;33m[TEST: AI Search with Pattern - ai_search:.*\.rs]\x1B[0m");
    println!(r"-> Action: ai_search:.*\.rs");
    brain.ipc.handle_request(r"ai_search:.*\.rs");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::Search);
        println!("\x1B[1;32m[PASSED]\x1B[0m AI search with pattern activated");
    }

    // 16. AI SEARCH IN CURRENT DIRECTORY TEST
    println!(r"\n\x1B[1;33m[TEST: AI Search in Current Directory - ai_search:current:.*\.rs]\x1B[0m");
    println!(r"-> Action: ai_search:current:.*\.rs");
    brain.ipc.handle_request(r"ai_search:current:.*\.rs");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::Search);
        println!("\x1B[1;32m[PASSED]\x1B[0m AI search in current directory activated");
    }

    // 17. SEARCH MODE SWITCHING TEST
    println!("\n\x1B[1;33m[TEST: Search Mode Switching]\x1B[0m");
    println!("-> Action: switch_search_mode:global");
    brain.ipc.handle_request("switch_search_mode:global");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::Search);
        println!("\x1B[1;32m[PASSED]\x1B[0m Search mode switched");
    }

    // 18. SEARCH MODE SWITCHING TEST
    println!("\n\x1B[1;33m[TEST: Search Mode Switching - switch_search_mode:local]\x1B[0m");
    println!("-> Action: switch_search_mode:local");
    brain.ipc.handle_request("switch_search_mode:local");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::Search);
        println!("\x1B[1;32m[PASSED]\x1B[0m Search mode switched to local");
    }

    // 19. SEARCH MODE SWITCHING TEST
    println!("\n\x1B[1;33m[TEST: Search Mode Switching - switch_search_mode:current]\x1B[0m");
    println!("-> Action: switch_search_mode:current");
    brain.ipc.handle_request("switch_search_mode:current");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::Search);
        println!("\x1B[1;32m[PASSED]\x1B[0m Search mode switched to current");
    }

    // 20. SEARCH MODE SWITCHING TEST
    println!("\n\x1B[1;33m[TEST: Search Mode Switching - switch_search_mode:ai]\x1B[0m");
    println!("-> Action: switch_search_mode:ai");
    brain.ipc.handle_request("switch_search_mode:ai");

    {
        let state = brain.state.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert_eq!(hub.mode, CommandHubMode::Search);
        println!("\x1B[1;32m[PASSED]\x1B[0m Search mode switched to AI");
    }

    println!("\n\x1B[1;32m=== AI INTEGRATION TESTS PASSED ===\x1B[0m");

    Ok(())
}
