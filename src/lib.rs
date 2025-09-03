//! RustUX - A SuperTux remake written in Rust
//!
//! This is a complete reimplementation of the classic SuperTux platformer game,
//! built from the ground up in Rust with modern game development practices.

pub mod assets;
pub mod audio;
pub mod badguy;
pub mod collision;
pub mod control;
pub mod engine;
pub mod gui;
pub mod math;
pub mod object;
pub mod physics;
pub mod sprite;
pub mod supertux;
pub mod trigger;
pub mod util;
pub mod video;

// Re-export commonly used types
pub use engine::Engine;
pub use math::{Vector2, Rect};
pub use util::{Result, Error};

/// Game configuration constants
pub mod config {
    /// Screen dimensions
    pub const SCREEN_WIDTH: u32 = 1024;
    pub const SCREEN_HEIGHT: u32 = 768;
    
    /// Tile size in pixels
    pub const TILE_SIZE: u32 = 32;
    
    /// Physics constants
    pub const GRAVITY: f32 = 1000.0;
    pub const TERMINAL_VELOCITY: f32 = 400.0;
    
    /// Game timing
    pub const TARGET_FPS: u32 = 60;
    pub const FIXED_TIMESTEP: f32 = 1.0 / 60.0;
}