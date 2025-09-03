//! Asset downloader utility for RustUX
//!
//! This utility downloads SuperTux sprites from the official repository
//! and organizes them in the local assets directory.

use rustux::assets::{AssetDownloader, SpriteDefinition};
use rustux::util::Result;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    println!("RustUX Asset Downloader");
    println!("======================");
    println!("Downloading SuperTux sprites from official repository...");

    // Create assets directory if it doesn't exist
    let assets_path = Path::new("assets");
    if !assets_path.exists() {
        std::fs::create_dir_all(assets_path)?;
        println!("Created assets directory");
    }

    // Initialize the downloader
    let downloader = AssetDownloader::new(assets_path);

    // Check if user wants comprehensive download
    let args: Vec<String> = std::env::args().collect();
    let comprehensive = args.len() > 1 && args[1] == "--all";

    if comprehensive {
        println!("Downloading ALL SuperTux assets (comprehensive mode)...");
        match downloader.download_all_assets().await {
            Ok(_) => {
                println!("✓ Successfully downloaded ALL SuperTux assets");
            }
            Err(e) => {
                eprintln!("✗ Error downloading assets: {}", e);
                return Err(e);
            }
        }
    } else {
        println!("Downloading essential sprites (use --all for comprehensive download)...");
        match downloader.download_essential_sprites().await {
            Ok(_) => {
                println!("✓ Successfully downloaded essential sprites");
            }
            Err(e) => {
                eprintln!("✗ Error downloading sprites: {}", e);
                return Err(e);
            }
        }
    }

    // Create sprite definitions
    println!("Creating sprite definitions...");
    
    let tux_small_def = SpriteDefinition::tux_small();
    let def_path = assets_path.join("sprites/definitions/tux_small.json");
    
    // Create definitions directory
    if let Some(parent) = def_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    tux_small_def.save_to_file(&def_path)?;
    println!("✓ Created sprite definition: {}", def_path.display());

    println!();
    println!("Asset download completed successfully!");
    println!("Sprites are now available in the 'assets' directory.");
    println!();
    println!("Directory structure:");
    println!("assets/");
    println!("├── sprites/");
    println!("│   ├── creatures/");
    println!("│   │   ├── tux/");
    println!("│   │   │   ├── small/");
    println!("│   │   │   └── big/");
    println!("│   │   └── enemies/");
    println!("│   └── definitions/");
    println!("│       └── tux_small.json");

    Ok(())
}