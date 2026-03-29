// Main test runner for brain crate tests.
// This file aggregates all test modules and provides a unified test entry point.

// Test modules for each feature area
mod brain_core;
mod shell_integration;
mod ai_integration;
mod sandbox;
mod collaboration;
mod communication;
mod messaging;
mod navigation;
mod user;
mod user_app;
mod user_hub;
mod user_terminal;

// Re-export all tests through a unified test module
// Each module can be run independently or together

#[cfg(test)]
pub mod tests {
    // Brain core tests
    pub use brain_core::*;

    // Shell integration tests
    pub use shell_integration::*;

    // AI integration tests
    pub use ai_integration::*;

    // Sandbox tests
    pub use sandbox::*;

    // Collaboration tests
    pub use collaboration::*;

    // Communication tests
    pub use communication::*;

    // Messaging tests
    pub use messaging::*;

    // Navigation tests
    pub use navigation::*;

    // User tests
    pub use user::*;

    // User app tests
    pub use user_app::*;

    // User hub tests
    pub use user_hub::*;

    // User terminal tests
    pub use user_terminal::*;
}

// Main test function that runs all tests
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\x1B[1;35m[TOS BRAIN TESTS]\x1B[0m");
    println!("Running all brain crate tests...\n");

    // Run all test modules
    brain_core::main()?;
    shell_integration::main()?;
    ai_integration::main()?;
    sandbox::main()?;
    collaboration::main()?;
    communication::main()?;
    messaging::main()?;
    navigation::main()?;
    user::main()?;
    user_app::main()?;
    user_hub::main()?;
    user_terminal::main()?;

    println!("\n\x1B[1;32m=== ALL TESTS PASSED ===\x1B[0m");

    Ok(())
}
