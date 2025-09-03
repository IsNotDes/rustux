//! RustUX - A SuperTux remake written in Rust

use rustux::engine::{Engine, GameStateManager, MenuState, PlayingState, StateId};
use rustux::util::Result;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    log::info!("Starting RustUX - SuperTux remake in Rust");

    // Create and configure the game engine
    let mut engine = Engine::new()?;

    // Set up game states
    let mut state_manager = GameStateManager::new();
    state_manager.add_state(StateId::Menu, Box::new(MenuState::new()));
    state_manager.add_state(StateId::Playing, Box::new(PlayingState::new()));
    
    // Start with the menu state
    log::info!("Setting initial state to Menu");
    state_manager.set_state(StateId::Menu)?;
    
    // Replace the engine's state manager with our configured one
    *engine.state_manager_mut() = state_manager;

    // Skip preloading - we'll download assets when needed
    // engine.resource_manager_mut().preload_common_resources()?;

    // Start the main game loop
    engine.run()?;

    log::info!("RustUX shutdown complete");
    Ok(())
}
