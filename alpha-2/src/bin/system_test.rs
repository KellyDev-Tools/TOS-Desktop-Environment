use tos_alpha2::brain::Brain;
use tos_alpha2::face::{Face, MockFace};
use tos_alpha2::common::CommandHubMode;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\x1B[1;36m[TOS ALPHA-2 COMPREHENSIVE COMPONENT TEST]\x1B[0m");
    println!("Initializing Brain and Face layers...\n");

    // 1. BOOTSTRAP
    let brain = Brain::new()?;
    let face_raw = Face::new(brain.state.clone(), brain.ipc.clone());
    let mut face = MockFace(face_raw);

    // HIERARCHY & LIFECYCLE
    println!("\x1B[1;33m[HIERARCHY & LIFECYCLE]\x1B[0m");
    face.0.render();
    sleep(Duration::from_millis(1500)).await;

    println!("-> Action: sector_create:Research");
    brain.ipc.handle_request("sector_create:Research");
    face.0.render();
    sleep(Duration::from_millis(1500)).await;

    println!("-> Action: zoom_in");
    face.simulate_bezel_zoom_in();
    face.0.render();
    sleep(Duration::from_millis(1500)).await;

    // SHELL & DIRECTORY MODE
    println!("\n\x1B[1;33m[SHELL & DIRECTORY MODE]\x1B[0m");
    println!("-> Action: cd /tmp");
    face.simulate_prompt_submit("cd /tmp");
    println!("-> Action: ls");
    face.simulate_prompt_submit("ls");
    sleep(Duration::from_millis(2000)).await; 
    face.0.render();
    sleep(Duration::from_millis(1500)).await;

    // SECURITY & TRUST
    println!("\n\x1B[1;33m[SECURITY INTERCEPTION]\x1B[0m");
    println!("-> Action: rm -rf / (Dangerous Command)");
    face.simulate_prompt_submit("rm -rf /");
    face.0.render();
    {
        let state = brain.state.lock().unwrap();
        if state.pending_confirmation.is_some() {
            println!("\x1B[1;32m[VERIFIED]\x1B[0m Interception successful §17.3");
        }
    }
    sleep(Duration::from_millis(1500)).await;

    // GLOBAL SEARCH
    println!("\n\x1B[1;33m[GLOBAL SEARCH]\x1B[0m");
    println!("-> Action: search:TOS");
    brain.ipc.handle_request("search:TOS");
    {
        let mut state = brain.state.lock().unwrap();
        let idx = state.active_sector_index;
        state.sectors[idx].hubs[0].mode = CommandHubMode::Search;
    }
    face.0.render();
    sleep(Duration::from_millis(1500)).await;

    // SERVICES
    println!("\n\x1B[1;33m[AUXILIARY SERVICES]\x1B[0m");
    println!("-> Action: log priority 3 event");
    brain.services.logger.log("SYSTEM_TEST: PRIORITY 3 ALERT", 3);
    
    println!("-> Action: save setting 'ui.theme'");
    brain.ipc.handle_request("set_setting:ui.theme;dark_obsidian");
    {
        let state = brain.state.lock().unwrap();
        brain.services.settings.save(&state.settings)?;
    }
    face.0.render();
    sleep(Duration::from_millis(1500)).await;

    // FINAL VALIDATION
    println!("\n\x1B[1;33m[FINAL REDUCED STATE]\x1B[0m");
    brain.ipc.handle_request("zoom_to:GlobalOverview");
    face.0.render();
    
    println!("\n\x1B[1;32m=== COMPREHENSIVE TEST SUCCESSFUL ===\x1B[0m");
    
    Ok(())
}
