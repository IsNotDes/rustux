//! Sprite system for RustUX

use crate::util::Result;
use crate::math::{Vector2, Rect};
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::Rect as SdlRect;
use std::collections::HashMap;
use std::path::Path;

/// Sprite animation frame
#[derive(Debug, Clone)]
pub struct AnimationFrame {
    /// Source rectangle in the texture
    pub source_rect: Rect,
    /// Duration of this frame in seconds
    pub duration: f32,
}

/// Sprite animation
#[derive(Debug, Clone)]
pub struct Animation {
    /// Animation frames
    pub frames: Vec<AnimationFrame>,
    /// Whether the animation loops
    pub loops: bool,
    /// Current frame index
    current_frame: usize,
    /// Time accumulated for current frame
    frame_time: f32,
}

impl Animation {
    /// Create a new animation
    pub fn new(frames: Vec<AnimationFrame>, loops: bool) -> Self {
        Self {
            frames,
            loops,
            current_frame: 0,
            frame_time: 0.0,
        }
    }

    /// Create a simple animation from a sprite sheet
    pub fn from_sprite_sheet(
        frame_width: f32,
        frame_height: f32,
        frame_count: usize,
        frame_duration: f32,
        loops: bool,
    ) -> Self {
        let mut frames = Vec::new();
        for i in 0..frame_count {
            frames.push(AnimationFrame {
                source_rect: Rect::new(
                    i as f32 * frame_width,
                    0.0,
                    frame_width,
                    frame_height,
                ),
                duration: frame_duration,
            });
        }
        Self::new(frames, loops)
    }

    /// Update the animation
    pub fn update(&mut self, delta_time: f32) {
        if self.frames.is_empty() {
            return;
        }

        self.frame_time += delta_time;
        let current_frame_duration = self.frames[self.current_frame].duration;

        if self.frame_time >= current_frame_duration {
            self.frame_time -= current_frame_duration;
            self.current_frame += 1;

            if self.current_frame >= self.frames.len() {
                if self.loops {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.frames.len() - 1;
                }
            }
        }
    }

    /// Get the current frame
    pub fn current_frame(&self) -> Option<&AnimationFrame> {
        self.frames.get(self.current_frame)
    }

    /// Reset the animation to the first frame
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.frame_time = 0.0;
    }

    /// Check if the animation is finished (for non-looping animations)
    pub fn is_finished(&self) -> bool {
        !self.loops && self.current_frame >= self.frames.len() - 1
    }
}

/// Sprite for rendering textures with animation support
#[derive(Debug, Clone)]
pub struct Sprite {
    /// Texture name for lookup in texture manager
    pub texture_name: String,
    /// Position in world coordinates
    pub position: Vector2,
    /// Size of the sprite
    pub size: Vector2,
    /// Current animation
    pub animation: Option<Animation>,
    /// Static source rectangle (used when no animation)
    pub source_rect: Option<Rect>,
    /// Whether the sprite is visible
    pub visible: bool,
    /// Sprite rotation in degrees
    pub rotation: f64,
    /// Sprite scale
    pub scale: Vector2,
    /// Flip horizontally
    pub flip_horizontal: bool,
    /// Flip vertically
    pub flip_vertical: bool,
}

impl Sprite {
    /// Create a new sprite
    pub fn new(texture_name: String, position: Vector2) -> Self {
        Self {
            texture_name,
            position,
            size: Vector2::new(32.0, 32.0),
            animation: None,
            source_rect: None,
            visible: true,
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0),
            flip_horizontal: false,
            flip_vertical: false,
        }
    }

    /// Create a sprite with a specific size
    pub fn with_size(texture_name: String, position: Vector2, size: Vector2) -> Self {
        let mut sprite = Self::new(texture_name, position);
        sprite.size = size;
        sprite
    }

    /// Create a sprite with size based on texture dimensions
    pub fn with_texture_size(texture_name: String, position: Vector2, texture_manager: &TextureManager) -> Self {
        let size = if let Some((width, height)) = texture_manager.get_texture_dimensions(&texture_name) {
            Vector2::new(width as f32, height as f32)
        } else {
            Vector2::new(32.0, 32.0) // Fallback to default size
        };
        
        let mut sprite = Self::new(texture_name, position);
        sprite.size = size;
        sprite
    }

    /// Set the sprite's animation
    pub fn set_animation(&mut self, animation: Animation) {
        self.animation = Some(animation);
    }

    /// Set a static source rectangle
    pub fn set_source_rect(&mut self, rect: Rect) {
        self.source_rect = Some(rect);
        self.animation = None;
    }

    /// Update the sprite (mainly for animations)
    pub fn update(&mut self, delta_time: f32) {
        if let Some(ref mut animation) = self.animation {
            animation.update(delta_time);
        }
    }

    /// Get the current source rectangle
    pub fn get_source_rect(&self) -> Option<Rect> {
        if let Some(ref animation) = self.animation {
            animation.current_frame().map(|frame| frame.source_rect)
        } else {
            self.source_rect
        }
    }

    /// Get the destination rectangle for rendering
    pub fn get_dest_rect(&self) -> Rect {
        let scaled_size = Vector2::new(
            self.size.x * self.scale.x,
            self.size.y * self.scale.y,
        );
        Rect::new(
            self.position.x,
            self.position.y,
            scaled_size.x,
            scaled_size.y,
        )
    }
}

/// Texture manager for loading and caching textures
pub struct TextureManager<'a> {
    textures: HashMap<String, Texture<'a>>,
    texture_creator: &'a TextureCreator<WindowContext>,
}

impl<'a> TextureManager<'a> {
    /// Create a new texture manager
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        Self {
            textures: HashMap::new(),
            texture_creator,
        }
    }

    /// Load a texture from file
    pub fn load_texture<P: AsRef<Path>>(&mut self, name: &str, path: P) -> Result<()> {
        let surface = sdl2::surface::Surface::load_bmp(path)?;
        let texture = self.texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| crate::util::Error::SpriteLoading(e.to_string()))?;
        
        self.textures.insert(name.to_string(), texture);
        log::debug!("Loaded texture: {}", name);
        Ok(())
    }

    /// Load a texture from image data
    pub fn load_texture_from_bytes(&mut self, name: &str, data: &[u8]) -> Result<()> {
        // Load image using the image crate
        let img = image::load_from_memory(data)
            .map_err(|e| crate::util::Error::SpriteLoading(e.to_string()))?;
        
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();
        
        // Create SDL surface from image data
        let mut rgba_data = rgba_img.into_raw();
        let surface = sdl2::surface::Surface::from_data(
            &mut rgba_data,
            width,
            height,
            width * 4,
            sdl2::pixels::PixelFormatEnum::RGBA32,
        ).map_err(|e| crate::util::Error::SpriteLoading(e.to_string()))?;
        
        let texture = self.texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| crate::util::Error::SpriteLoading(e.to_string()))?;
        
        self.textures.insert(name.to_string(), texture);
        log::debug!("Loaded texture from bytes: {}", name);
        Ok(())
    }

    /// Get a texture by name
    pub fn get_texture(&self, name: &str) -> Option<&Texture> {
        self.textures.get(name)
    }

    /// Check if a texture is loaded
    pub fn has_texture(&self, name: &str) -> bool {
        self.textures.contains_key(name)
    }

    /// Remove a texture
    pub fn remove_texture(&mut self, name: &str) -> bool {
        self.textures.remove(name).is_some()
    }

    /// Clear all textures
    pub fn clear(&mut self) {
        self.textures.clear();
    }

    /// Get the number of loaded textures
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Load textures from a SuperTux sprite definition
    pub fn load_from_sprite_definition(&mut self, definition: &crate::assets::SpriteDefinition) -> Result<()> {
        use std::path::Path;
        
        let base_path = Path::new("assets").join(&definition.texture_path);
        
        for (_animation_name, animation_def) in &definition.animations {
            for frame in &animation_def.frames {
                let texture_path = base_path.join(&frame.file_name);
                let texture_name = format!("{}_{}", definition.name, frame.file_name.trim_end_matches(".png"));
                
                if let Err(e) = self.load_texture_from_file(&texture_name, &texture_path) {
                    log::warn!("Failed to load texture {}: {}", texture_name, e);
                }
            }
        }
        
        log::info!("Loaded textures for sprite definition: {}", definition.name);
        Ok(())
    }

    /// Load texture from file (supports PNG, JPG, etc.)
    pub fn load_texture_from_file<P: AsRef<std::path::Path>>(&mut self, name: &str, path: P) -> Result<()> {
        let img = image::open(&path)
            .map_err(|e| crate::util::Error::SpriteLoading(format!("Failed to load image {}: {}", path.as_ref().display(), e)))?;
        
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();
        
        // Create SDL surface from image data
        let mut rgba_data = rgba_img.into_raw();
        let surface = sdl2::surface::Surface::from_data(
            &mut rgba_data,
            width,
            height,
            width * 4,
            sdl2::pixels::PixelFormatEnum::RGBA32,
        ).map_err(|e| crate::util::Error::SpriteLoading(e.to_string()))?;
        
        let texture = self.texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| crate::util::Error::SpriteLoading(e.to_string()))?;
        
        self.textures.insert(name.to_string(), texture);
        log::debug!("Loaded texture from file: {} -> {}", path.as_ref().display(), name);
        Ok(())
    }

    /// Get texture dimensions
    pub fn get_texture_dimensions(&self, name: &str) -> Option<(u32, u32)> {
        self.textures.get(name).map(|texture| {
            let query = texture.query();
            (query.width, query.height)
        })
    }
}

/// Sprite renderer for drawing sprites to the canvas
pub struct SpriteRenderer;

impl SpriteRenderer {
    /// Render a sprite to the canvas
    pub fn render_sprite(
        canvas: &mut Canvas<Window>,
        texture_manager: &TextureManager,
        sprite: &Sprite,
    ) -> Result<()> {
        if !sprite.visible {
            return Ok(());
        }

        let texture = texture_manager.get_texture(&sprite.texture_name)
            .ok_or_else(|| crate::util::Error::SpriteLoading(
                format!("Texture not found: {}", sprite.texture_name)
            ))?;

        let dest_rect = sprite.get_dest_rect();
        let sdl_dest = SdlRect::new(
            dest_rect.x as i32,
            dest_rect.y as i32,
            dest_rect.width as u32,
            dest_rect.height as u32,
        );

        let sdl_src = sprite.get_source_rect().map(|rect| SdlRect::new(
            rect.x as i32,
            rect.y as i32,
            rect.width as u32,
            rect.height as u32,
        ));

        canvas.copy_ex(
            texture,
            sdl_src,
            Some(sdl_dest),
            sprite.rotation,
            None,
            sprite.flip_horizontal,
            sprite.flip_vertical,
        ).map_err(|e| crate::util::Error::Video(e))?;

        Ok(())
    }

    /// Render multiple sprites
    pub fn render_sprites(
        canvas: &mut Canvas<Window>,
        texture_manager: &TextureManager,
        sprites: &[&Sprite],
    ) -> Result<()> {
        for sprite in sprites {
            Self::render_sprite(canvas, texture_manager, sprite)?;
        }
        Ok(())
    }
}

/// SuperTux sprite factory for creating sprites from definitions
pub struct SuperTuxSpriteFactory;

impl SuperTuxSpriteFactory {
    /// Create a sprite from a SuperTux sprite definition
    pub fn create_sprite_from_definition(
        definition: &crate::assets::SpriteDefinition,position: Vector2,
        animation_name: &str,
    ) -> Result<Sprite> {
        let animation_def = definition.animations.get(animation_name)
            .ok_or_else(|| crate::util::Error::SpriteLoading(
                format!("Animation '{}' not found in sprite definition '{}'", animation_name, definition.name)
            ))?;

        // Create animation frames
        let mut frames = Vec::new();
        for frame_def in &animation_def.frames {
            let _texture_name = format!("{}_{}", definition.name, frame_def.file_name.trim_end_matches(".png"));
            frames.push(AnimationFrame {
                source_rect: Rect::new(0.0, 0.0,32.0, 32.0), // Default size, should be determined from texture
                duration: frame_def.duration.unwrap_or(animation_def.frame_duration),
            });
        }

        let animation = Animation::new(frames, animation_def.loops);
        let mut sprite = Sprite::new(
            format!("{}_{}", definition.name, animation_def.frames[0].file_name.trim_end_matches(".png")),
            position,
        );
        sprite.set_animation(animation);

        Ok(sprite)
    }

    /// Load a SuperTux sprite definition from file
    pub fn load_sprite_definition<P: AsRef<std::path::Path>>(path: P) -> Result<crate::assets::SpriteDefinition> {
        crate::assets::SpriteDefinition::load_from_file(path)
    }

    /// Create Tux sprite with specified animation
    pub fn create_tux_sprite(position: Vector2, animation: &str) -> Result<Sprite> {
        let definition_path = std::path::Path::new("assets/sprites/definitions/tux_small.json");
        let definition = Self::load_sprite_definition(definition_path)?;
        Self::create_sprite_from_definition(&definition, position, animation)
    }
}

/// Helper functions for creating common sprite animations
pub mod animations {
    use super::*;

    /// Create a simple idle animation (single frame)
    pub fn idle(source_rect: Rect) -> Animation {
        Animation::new(
            vec![AnimationFrame {
                source_rect,
                duration: 1.0,
            }],
            true,
        )
    }

    /// Create a walking animation
    pub fn walk(frame_width: f32, frame_height: f32, frame_count: usize) -> Animation {
        Animation::from_sprite_sheet(frame_width, frame_height, frame_count, 0.1, true)
    }

    /// Create a jumping animation
    pub fn jump(source_rect: Rect) -> Animation {
        Animation::new(
            vec![AnimationFrame {
                source_rect,
                duration: 0.5,
            }],
            false,
        )
    }
}