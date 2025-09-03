//! Asset management and downloading utilities for RustUX

use crate::util::Result;
use reqwest;
use std::path::{Path, PathBuf};
use std::fs;


/// Base URL for SuperTux repository raw files
const SUPERTUX_BASE_URL: &str = "https://raw.githubusercontent.com/SuperTux/supertux/master/data/images";

/// Asset downloader for SuperTux sprites
pub struct AssetDownloader {
    client: reqwest::Client,
    base_path: PathBuf,
}

impl AssetDownloader {
    /// Create a new asset downloader
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Download a single file from SuperTux repository
    pub async fn download_file(&self, relative_path: &str, local_path: &str) -> Result<()> {
        let url = format!("{}/{}", SUPERTUX_BASE_URL, relative_path);
        let local_file_path = self.base_path.join(local_path);

        // Create parent directories if they don't exist
        if let Some(parent) = local_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        log::info!("Downloading {} to {}", url, local_file_path.display());

        let response = self.client.get(&url).send().await
            .map_err(|e| crate::util::Error::AssetDownload(e.to_string()))?;

        if !response.status().is_success() {
            return Err(crate::util::Error::AssetDownload(
                format!("Failed to download {}: HTTP {}", url, response.status())
            ));
        }

        let bytes = response.bytes().await
            .map_err(|e| crate::util::Error::AssetDownload(e.to_string()))?;

        fs::write(&local_file_path, &bytes)?;
        log::debug!("Successfully downloaded {}", local_file_path.display());

        Ok(())
    }

    /// Download Tux sprites (small variant)
    pub async fn download_tux_small_sprites(&self) -> Result<()> {
        let tux_sprites = vec![
            // Climbing animation
            "climb-0.png", "climb-1.png", "climb-2.png", "climb-3.png",
            "climb-4.png", "climb-5.png", "climb-6.png", "climb-7.png",
            // Growing animation
            "grow-0.png", "grow-1.png", "grow-2.png", "grow-3.png",
            "grow-4.png", "grow-5.png", "grow-6.png", "grow-7.png",
            // Growing climb animation
            "grow_climb-0.png", "grow_climb-1.png", "grow_climb-2.png", "grow_climb-3.png",
            // Game over animation
            "gameover-0.png", "gameover-1.png", "gameover-2.png", "gameover-3.png",
            // Idle and walking
            "idle-0.png", "walk-0.png", "walk-1.png", "walk-2.png", "walk-3.png",
            "walk-4.png", "walk-5.png", "walk-6.png", "walk-7.png",
            // Jumping
            "jump-0.png", "jump-1.png",// Skidding
            "skid-0.png",
            // Ducking
            "duck-0.png",
            // Kicking
            "kick-0.png",
        ];

        for sprite in tux_sprites {
            let remote_path = format!("creatures/tux/small/{}", sprite);
            let local_path = format!("sprites/creatures/tux/small/{}", sprite);
            if let Err(e) = self.download_file(&remote_path, &local_path).await {
                log::warn!("Failed to download {}: {}", sprite, e);
                // Continue with other sprites even if one fails
            }
        }

        Ok(())
    }

    /// Download Tux sprites (big variant)
    pub async fn download_tux_big_sprites(&self) -> Result<()> {
        let tux_sprites = vec![
            // Basic animations for big Tux
            "idle-0.png", "walk-0.png", "walk-1.png", "walk-2.png", "walk-3.png",
            "walk-4.png", "walk-5.png", "walk-6.png", "walk-7.png",
            "jump-0.png", "jump-1.png", "skid-0.png", "duck-0.png", "kick-0.png",
            // Fire Tux variants
            "fire-idle-0.png", "fire-walk-0.png", "fire-walk-1.png", "fire-walk-2.png",
            "fire-walk-3.png", "fire-walk-4.png", "fire-walk-5.png", "fire-walk-6.png",
            "fire-walk-7.png", "fire-jump-0.png", "fire-jump-1.png", "fire-skid-0.png",
            "fire-duck-0.png", "fire-kick-0.png",
        ];

        for sprite in tux_sprites {
            let remote_path = format!("creatures/tux/big/{}", sprite);
            let local_path = format!("sprites/creatures/tux/big/{}", sprite);
            
            if let Err(e) = self.download_file(&remote_path, &local_path).await {
                log::warn!("Failed to download {}: {}", sprite, e);
                // Continue with other sprites even if one fails
            }
        }

        Ok(())
    }

    /// Download common enemy sprites
    pub async fn download_enemy_sprites(&self) -> Result<()> {
        let enemies = vec![
            ("bouncing_snowball", vec!["left-0.png", "left-1.png", "left-2.png", "left-3.png", "left-4.png", "left-5.png"]),
            ("snowball", vec!["left-0.png", "left-1.png", "left-2.png", "squished-left.png"]),
            ("spiky", vec!["left-0.png", "left-1.png", "left-2.png", "sleeping-left.png"]),
            ("mriceblock", vec!["left-0.png", "left-1.png", "left-2.png", "flat-left.png"]),
            ("goomba", vec!["left-0.png", "left-1.png", "squished-left.png"]),
        ];

        for (enemy_name, sprites) in enemies {
            for sprite in sprites {
                let remote_path = format!("creatures/{}/{}", enemy_name, sprite);
                let local_path = format!("sprites/creatures/enemies/{}/{}", enemy_name, sprite);
                
                if let Err(e) = self.download_file(&remote_path, &local_path).await {
                    log::warn!("Failed to download {} {}: {}", enemy_name, sprite, e);
                    // Continue with other sprites even if one fails
                }
            }
        }

        Ok(())
    }

    /// Download all essential SuperTux sprites
    pub async fn download_essential_sprites(&self) -> Result<()> {
        log::info!("Starting download of essential SuperTux sprites...");

        // Download Tux sprites
        self.download_tux_small_sprites().await?;
        self.download_tux_big_sprites().await?;

        // Download enemy sprites
        self.download_enemy_sprites().await?;
// Download tile assets (needed for platforms and ground)
        self.download_tile_assets().await?;

        log::info!("Finished downloading essential SuperTux sprites");
        Ok(())
    }

    /// Download tile assets
    pub async fn download_tile_assets(&self) -> Result<()> {
        log::info!("Downloading tile assets...");

        // Basic blocks
        let blocks = vec![
            "bigblock.png", "block10.png", "block11.png", "block5.png",
            "brick0.png", "brick1.png", "brick2.png", "brick3.png",
            "brick_piece1.png", "brick_piece2.png", "brick_piece3.png",
            "brick_piece4.png", "brick_piece5.png", "brick_piece6.png",
            "brick_piece7.png", "block_wood.png", "block_overlays.png"
        ];

        for block in blocks {
            let remote_path = format!("tiles/blocks/{}", block);
            let local_path = format!("sprites/tiles/blocks/{}", block);
            if let Err(e) = self.download_file(&remote_path, &local_path).await {
                log::warn!("Failed to download block {}: {}", block, e);
            }
        }

        // Snow tiles
        let snow_tiles = vec![
            "snow1.png", "snow2.png", "snow3.png", "snow4.png", "snow5.png",
            "snow6.png", "snow7.png", "snow8.png", "snow9.png", "snow10.png",
            "snow11.png", "snow12.png", "snow13.png", "snow14.png", "snow15.png",
            "snow16.png", "snow17.png", "snow18.png", "snow19.png", "snow20.png"
        ];

        for tile in snow_tiles {
            let remote_path = format!("tiles/snow/{}", tile);
            let local_path = format!("sprites/tiles/snow/{}", tile);
            if let Err(e) = self.download_file(&remote_path, &local_path).await {
                log::warn!("Failed to download snow tile {}: {}", tile, e);
            }
        }

        // Forest tiles
        let forest_tiles = vec![
            "foresttiles-1.png", "foresttiles-2.png", "foresttiles-3.png",
            "foresttiles-4.png", "foresttiles-5.png", "foresttiles-6.png",
            "foresttiles-7.png", "foresttiles-8.png", "foresttiles-9.png",
            "foresttiles-10.png", "foresttiles-11.png", "foresttiles-12.png"
        ];

        for tile in forest_tiles {
            let remote_path = format!("tiles/forest/{}", tile);
            let local_path = format!("sprites/tiles/forest/{}", tile);
            if let Err(e) = self.download_file(&remote_path, &local_path).await {
                log::warn!("Failed to download forest tile {}: {}", tile, e);
            }
        }

        // Castle tiles
        let castle_tiles = vec![
            "castle_wall_1.png", "castle_wall_2.png", "castle_wall_3.png",
            "castle_wall_4.png", "grey_brick.png", "grey_brick_dark.png"
        ];

        for tile in castle_tiles {
            let remote_path = format!("tiles/castle/{}", tile);
            let local_path = format!("sprites/tiles/castle/{}", tile);
            if let Err(e) = self.download_file(&remote_path, &local_path).await {
                log::warn!("Failed to download castle tile {}: {}", tile, e);
            }
        }

        log::info!("Finished downloading tile assets");
        Ok(())
    }

    /// Download background assets
    pub async fn download_background_assets(&self) -> Result<()> {
        log::info!("Downloading background assets...");

        let backgrounds = vec![
            "arctis.png", "arctis2.jpg", "cloud.png", "cloudsdark.png",
            "forest1.jpg", "forest2.png", "ghostwood.png", "snow_para_1.png",
            "snow_para_2.png", "snow_para_3.png", "ocean.png"
        ];

        for bg in backgrounds {
            let remote_path = format!("background/{}", bg);
            let local_path = format!("sprites/backgrounds/{}", bg);
            if let Err(e) = self.download_file(&remote_path, &local_path).await {
                log::warn!("Failed to download background {}: {}", bg, e);
            }
        }

        log::info!("Finished downloading background assets");
        Ok(())
    }

    /// Download object assets (coins, powerups, etc.)
    pub async fn download_object_assets(&self) -> Result<()> {
        log::info!("Downloading object assets...");

        // Coins
        let coins = vec![
            "coin-0.png", "coin-1.png", "coin-2.png", "coin-3.png",
            "coin-4.png", "coin-5.png", "coin-6.png", "coin-7.png"
        ];

        for coin in coins {
            let remote_path = format!("objects/coin/{}", coin);
            let local_path = format!("sprites/objects/coin/{}", coin);
            if let Err(e) = self.download_file(&remote_path, &local_path).await {
                log::warn!("Failed to download coin {}: {}", coin, e);
            }
        }

        // Powerups
        let powerups = vec![
            "egg.png", "fireflower.png", "iceflower.png", "star.png",
            "1up.png", "potions.png", "herring.png"
        ];

        for powerup in powerups {
            let remote_path = format!("objects/powerup/{}", powerup);
            let local_path = format!("sprites/objects/powerup/{}", powerup);
            if let Err(e) = self.download_file(&remote_path, &local_path).await {
                log::warn!("Failed to download powerup {}: {}", powerup, e);
            }
        }

        // Bonus blocks
        let bonus_blocks = vec![
            "full-0.png", "full-1.png", "full-2.png", "full-3.png",
            "full-4.png", "empty.png"
        ];

        for block in bonus_blocks {
            let remote_path = format!("objects/bonus_block/{}", block);
            let local_path = format!("sprites/objects/bonus_block/{}", block);
            if let Err(e) = self.download_file(&remote_path, &local_path).await {
                log::warn!("Failed to download bonus block {}: {}", block, e);
            }
        }

        log::info!("Finished downloading object assets");
        Ok(())
    }

    /// Download particle effects
    pub async fn download_particle_assets(&self) -> Result<()> {
        log::info!("Downloading particle assets...");

        let particles = vec![
            "smoke.png", "sparkle.png", "star.png", "snowflake-1.png",
            "snowflake-2.png", "snowflake-3.png", "rainsplash-1.png",
            "rainsplash-2.png", "rainsplash-3.png", "rainsplash-4.png"
        ];

        for particle in particles {
            let remote_path = format!("particles/{}", particle);
            let local_path = format!("sprites/particles/{}", particle);
            if let Err(e) = self.download_file(&remote_path, &local_path).await {
                log::warn!("Failed to download particle {}: {}", particle, e);
            }
        }

        log::info!("Finished downloading particle assets");
        Ok(())
    }

    /// Download ALL SuperTux assets (comprehensive)
    pub async fn download_all_assets(&self) -> Result<()> {
        log::info!("Starting comprehensive download of ALL SuperTux assets...");

        // Download all categories
        self.download_essential_sprites().await?;
        self.download_tile_assets().await?;
        self.download_background_assets().await?;
        self.download_object_assets().await?;
        self.download_particle_assets().await?;

        log::info!("Finished downloading ALL SuperTux assets!");
        Ok(())
    }
    }

/// Sprite definition for SuperTux assets
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpriteDefinition {
    pub name: String,
    pub texture_path: String,
    pub animations: std::collections::HashMap<String, AnimationDefinition>,
}

/// Animation definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnimationDefinition {
    pub frames: Vec<FrameDefinition>,
    pub loops: bool,
    pub frame_duration: f32,
}

/// Frame definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FrameDefinition {
    pub file_name: String,
    pub duration: Option<f32>, // Override default duration if needed
}

impl SpriteDefinition {
    /// Create Tux small sprite definition
    pub fn tux_small() -> Self {
        let mut animations = std::collections::HashMap::new();

        // Idle animation
        animations.insert("idle".to_string(), AnimationDefinition {
            frames: vec![FrameDefinition { file_name: "idle-0.png".to_string(), duration: None }],
            loops: true,
            frame_duration: 1.0,
        });

        // Walk animation
        animations.insert("walk".to_string(), AnimationDefinition {
            frames: (0..8).map(|i| FrameDefinition {
                file_name: format!("walk-{}.png", i),
                duration: None,
            }).collect(),
            loops: true,
            frame_duration: 0.1,
        });

        // Jump animation
        animations.insert("jump".to_string(), AnimationDefinition {
            frames: vec![
                FrameDefinition { file_name: "jump-0.png".to_string(), duration: None },
                FrameDefinition { file_name: "jump-1.png".to_string(), duration: None },
            ],
            loops: false,
            frame_duration: 0.2,
        });

        // Climb animation
        animations.insert("climb".to_string(), AnimationDefinition {
            frames: (0..8).map(|i| FrameDefinition {
                file_name: format!("climb-{}.png", i),
                duration: None,
            }).collect(),
            loops: true,
            frame_duration: 0.15,
        });

        // Grow animation
        animations.insert("grow".to_string(), AnimationDefinition {
            frames: (0..8).map(|i| FrameDefinition {
                file_name: format!("grow-{}.png", i),
                duration: None,
            }).collect(),
            loops: false,
            frame_duration: 0.1,
        });

        Self {
            name: "tux_small".to_string(),
            texture_path: "sprites/creatures/tux/small".to_string(),
            animations,
        }
    }

    /// Save sprite definition to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load sprite definition from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let definition = serde_json::from_str(&content)?;
        Ok(definition)
    }
}