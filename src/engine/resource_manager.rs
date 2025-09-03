//! Resource management for RustUX

use crate::util::{Result, Error};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Manages game resources like textures, sounds, and data files
pub struct ResourceManager {
    /// Base data directory
    data_dir: PathBuf,
    /// Loaded texture paths for reference
    texture_cache: HashMap<String, PathBuf>,
    /// Loaded sound paths for reference
    sound_cache: HashMap<String, PathBuf>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Result<Self> {
        let data_dir = crate::util::fs::get_data_dir()?;
        
        Ok(Self {
            data_dir,
            texture_cache: HashMap::new(),
            sound_cache: HashMap::new(),
        })
    }

    /// Get the full path to a resource file
    pub fn get_resource_path<P: AsRef<Path>>(&self, relative_path: P) -> PathBuf {
        self.data_dir.join(relative_path)
    }

    /// Load a texture and cache its path
    pub fn load_texture(&mut self, name: &str, relative_path: &str) -> Result<PathBuf> {
        let full_path = self.get_resource_path(relative_path);
        
        if !full_path.exists() {
            return Err(Error::ResourceNotFound(format!(
                "Texture file not found: {}",
                full_path.display()
            )));
        }

        self.texture_cache.insert(name.to_string(), full_path.clone());
        log::debug!("Loaded texture '{}' from {}", name, full_path.display());
        
        Ok(full_path)
    }

    /// Get a cached texture path
    pub fn get_texture_path(&self, name: &str) -> Option<&PathBuf> {
        self.texture_cache.get(name)
    }

    /// Load a sound and cache its path
    pub fn load_sound(&mut self, name: &str, relative_path: &str) -> Result<PathBuf> {
        let full_path = self.get_resource_path(relative_path);
        
        if !full_path.exists() {
            return Err(Error::ResourceNotFound(format!(
                "Sound file not found: {}",
                full_path.display()
            )));
        }

        self.sound_cache.insert(name.to_string(), full_path.clone());
        log::debug!("Loaded sound '{}' from {}", name, full_path.display());
        
        Ok(full_path)
    }

    /// Get a cached sound path
    pub fn get_sound_path(&self, name: &str) -> Option<&PathBuf> {
        self.sound_cache.get(name)
    }

    /// Load a level file
    pub fn load_level_data(&self, level_name: &str) -> Result<String> {
        let level_path = self.get_resource_path(format!("levels/{}.json", level_name));
        
        if !level_path.exists() {
            return Err(Error::ResourceNotFound(format!(
                "Level file not found: {}",
                level_path.display()
            )));
        }

        let content = std::fs::read_to_string(&level_path)?;
        log::debug!("Loaded level data for '{}'", level_name);
        
        Ok(content)
    }

    /// Load configuration data
    pub fn load_config<T>(&self, config_name: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let config_path = self.get_resource_path(format!("config/{}.toml", config_name));
        
        if !config_path.exists() {
            return Err(Error::ResourceNotFound(format!(
                "Config file not found: {}",
                config_path.display()
            )));
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: T = toml::from_str(&content)?;
        
        log::debug!("Loaded config '{}'", config_name);
        Ok(config)
    }

    /// Check if a resource exists
    pub fn resource_exists<P: AsRef<Path>>(&self, relative_path: P) -> bool {
        self.get_resource_path(relative_path).exists()
    }

    /// Get the data directory
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Clear all cached resources
    pub fn clear_cache(&mut self) {
        self.texture_cache.clear();
        self.sound_cache.clear();
        log::debug!("Cleared resource cache");
    }

    /// Get all cached texture names
    pub fn get_cached_textures(&self) -> Vec<&String> {
        self.texture_cache.keys().collect()
    }

    /// Get all cached sound names
    pub fn get_cached_sounds(&self) -> Vec<&String> {
        self.sound_cache.keys().collect()
    }

    /// Preload common resources
    pub fn preload_common_resources(&mut self) -> Result<()> {
            log::info!("Preloading common resources...");
    
            // Load our basic textures
            let basic_textures = [
                ("tux", "textures/tux.bmp"),
                ("platform", "textures/platform.bmp"),
                ("ground", "textures/ground.bmp"),
                ("coin", "textures/coin.bmp"),
            ];
    
            for (name, path) in &basic_textures {
                if self.resource_exists(path) {
                    self.load_texture(name, path)?;
                    log::info!("Loaded texture: {} from {}", name, path);
                } else {
                    log::warn!("Basic texture not found: {}", path);
                }
            }
    
            // Try to load common sounds (optional for now)
            let common_sounds = [
                ("jump", "sounds/jump.wav"),
                ("coin", "sounds/coin.wav"),
                ("hurt", "sounds/hurt.wav"),
                ("music_main", "music/main_theme.ogg"),
            ];
    
            for (name, path) in &common_sounds {
                if self.resource_exists(path) {
                    self.load_sound(name, path)?;
                } else {
                    log::debug!("Optional sound not found: {}", path);
                }
            }
    
            log::info!("Finished preloading resources");
            Ok(())
        }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default ResourceManager")
    }
}

/// Resource loading utilities
pub mod loaders {
    use super::*;

    /// Load and parse a JSON file
    pub fn load_json<T>(path: &Path) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let content = std::fs::read_to_string(path)?;
        let data = serde_json::from_str(&content)?;
        Ok(data)
    }

    /// Load and parse a TOML file
    pub fn load_toml<T>(path: &Path) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let content = std::fs::read_to_string(path)?;
        let data = toml::from_str(&content)?;
        Ok(data)
    }

    /// Save data as JSON
    pub fn save_json<T>(data: &T, path: &Path) -> Result<()>
    where
        T: serde::Serialize,
    {
        let content = serde_json::to_string_pretty(data)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Save data as TOML
    pub fn save_toml<T>(data: &T, path: &Path) -> Result<()>
    where
        T: serde::Serialize,
    {
        let content = toml::to_string_pretty(data)
            .map_err(|e| Error::Unknown(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}