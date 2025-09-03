//! Test program for SuperTux sprite loading
//!
//! This program tests the sprite loading functionality by loading
//! SuperTux sprites and displaying basic information about them.

use rustux::assets::SpriteDefinition;
use rustux::sprite::{SuperTuxSpriteFactory, TextureManager};
use rustux::math::Vector2;
use rustux::util::Result;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    println!("RustUX Sprite Loading Test");
    println!("=========================");

    // Test loading sprite definition
    println!("Loading Tux sprite definition...");
    let definition_path = "assets/sprites/definitions/tux_small.json";
    
    match SpriteDefinition::load_from_file(definition_path) {
        Ok(definition) => {
            println!("✓ Successfully loaded sprite definition: {}", definition.name);
            println!("  Texture path: {}", definition.texture_path);
            println!("  Animations:");
            for (name, animation) in &definition.animations {
                println!("    - {}: {} frames, loops: {}, duration: {}s",name, 
                    animation.frames.len(), 
                    animation.loops, 
                    animation.frame_duration
                );
            }

            // Test creating sprites
            println!("\nTesting sprite creation...");
            
            let animations_to_test = vec!["idle", "walk", "jump", "climb"];
            for animation_name in animations_to_test {
                match SuperTuxSpriteFactory::create_sprite_from_definition(
                    &definition, 
                    Vector2::new(100.0, 100.0), 
                    animation_name
                ) {
                    Ok(sprite) => {
                        println!("✓ Created {} sprite successfully", animation_name);
                        println!("  Position: ({}, {})", sprite.position.x, sprite.position.y);
                        println!("  Size: ({}, {})", sprite.size.x, sprite.size.y);
                        println!("  Texture: {}", sprite.texture_name);
                    }
                    Err(e) => {
                        println!("✗ Failed to create {} sprite: {}", animation_name, e);
                    }
                }
            }

            // Test convenience function
            println!("\nTesting convenience function...");
            match SuperTuxSpriteFactory::create_tux_sprite(Vector2::new(200.0, 200.0), "walk") {
                Ok(sprite) => {
                    println!("✓ Created Tux sprite using convenience function");
                    println!("  Position: ({}, {})", sprite.position.x, sprite.position.y);
                }
                Err(e) => {
                    println!("✗ Failed to create Tux sprite: {}", e);
                }
            }}
        Err(e) => {
            println!("✗ Failed to load sprite definition: {}", e);
            println!("Make sure you have run 'cargo run --bin download_assets' first!");return Err(e);
        }
    }

    println!("\nSprite loading test completed!");
    println!("\nNote: This test only verifies sprite definition loading and sprite creation.");
    println!("To test actual texture loading and rendering, you need an SDL2 context.");

    Ok(())
}