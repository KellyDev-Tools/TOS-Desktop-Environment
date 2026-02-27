use tos_alpha2::brain::Brain;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize Brain (Logic, State, Shell)
    let brain = Brain::new()?;
    
    println!("TOS Alpha-2 Brain Initialized.");
    
    // Example: Submitting a command manually for early binary flow testing
    brain.ipc.handle_request("prompt_submit:ls -la");
    
    // Keep alive to see output
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Check state to see if output arrived
    {
        let state = brain.state.lock().unwrap();
        if let Some(sector) = state.sectors.get(0) {
            println!("Output received: {} lines", sector.hubs[0].terminal_output.len());
        }
    }
    
    Ok(())
}
