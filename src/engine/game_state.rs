//! Game state management for RustUX

use crate::util::Result;
use crate::assets::AssetDownloader;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;
use std::path::Path;

/// Trait for game states
pub trait GameState {
    /// Update the game state
    fn update(&mut self, delta_time: f32) -> Result<()>;
/// Update the game state with input manager
    fn update_with_input(&mut self, delta_time: f32, input_manager: &crate::control::InputManager) -> Result<()> {
        let _ = input_manager; // Default implementation ignores input manager
        self.update(delta_time)
    }

    /// Render the game state
    fn render(&self, canvas: &mut Canvas<Window>) -> Result<()>;

    /// Handle key down events
    fn handle_key_down(&mut self, keycode: Keycode) -> Result<Option<StateId>> {
        let _ = keycode; // Suppress unused parameter warning
        Ok(None)
    }

    /// Handle key up events
    fn handle_key_up(&mut self, keycode: Keycode) -> Result<()> {
        let _ = keycode; // Suppress unused parameter warning
        Ok(())
    }

    /// Called when entering this state
    fn on_enter(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when leaving this state
    fn on_exit(&mut self) -> Result<()> {
        Ok(())
    }

    /// Get the state's name
    fn name(&self) -> &str;
}

/// Game state identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StateId {
    Menu,
    Playing,
    Paused,
    GameOver,
    Loading,
    Settings,
}

impl StateId {
    pub fn as_str(&self) -> &'static str {
        match self {
            StateId::Menu => "menu",
            StateId::Playing => "playing",
            StateId::Paused => "paused",
            StateId::GameOver => "game_over",
            StateId::Loading => "loading",
            StateId::Settings => "settings",
        }
    }
}

/// Manages game states and transitions
pub struct GameStateManager {
    states: HashMap<StateId, Box<dyn GameState>>,
    current_state: Option<StateId>,
    next_state: Option<StateId>,
}

impl GameStateManager {
    /// Create a new game state manager
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            current_state: None,
            next_state: None,
        }
    }

    /// Add a game state
    pub fn add_state(&mut self, id: StateId, state: Box<dyn GameState>) {
        self.states.insert(id, state);
    }

    /// Set the current state
    pub fn set_state(&mut self, id: StateId) -> Result<()> {
        if !self.states.contains_key(&id) {
            return Err(crate::util::Error::GameLogic(format!(
                "State {:?} not found",
                id
            )));
        }
        self.next_state = Some(id);
        Ok(())
    }

    /// Get the current state
    pub fn current_state(&self) -> Option<&dyn GameState> {
        self.current_state.as_ref().and_then(|id| self.states.get(id))
            .map(|state| state.as_ref())
    }

    /// Get the current state mutably
    pub fn current_state_mut(&mut self) -> Option<&mut Box<dyn GameState>> {
        if let Some(current_id) = &self.current_state {
            self.states.get_mut(current_id)
        } else {
            None
        }
    }

    /// Process state transitions
    pub fn process_transitions(&mut self) -> Result<()> {
        if let Some(next_state) = self.next_state.take() {
            log::info!("Processing state transition to {:?}", next_state);
            // Exit current state
            if let Some(current_id) = &self.current_state {
                log::info!("Exiting current state: {:?}", current_id);if let Some(current_state) = self.states.get_mut(current_id) {
                    current_state.on_exit()?;
                }
            }

            // Enter new state
            log::info!("Entering new state: {:?}", next_state);
            if let Some(new_state) = self.states.get_mut(&next_state) {
                new_state.on_enter()?;
            }

            self.current_state = Some(next_state);
            log::info!("State transition completed. Current state: {:?}", self.current_state);
        }
        Ok(())
    }

    /// Get the current state ID
    pub fn current_state_id(&self) -> Option<&StateId> {
        self.current_state.as_ref()}

    /// Check if a state exists
    pub fn has_state(&self, id: &StateId) -> bool {
        self.states.contains_key(id)
    }

    /// Remove a state
    pub fn remove_state(&mut self, id: &StateId) -> Option<Box<dyn GameState>> {
        self.states.remove(id)
    }
}

impl Default for GameStateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple menu state for testing
pub struct MenuState {
    title: String,
    downloading: bool,
    download_complete: bool,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            title: "RustUX - SuperTux in Rust".to_string(),
            downloading: false,
            download_complete: false,
        }
    }/// Download assets asynchronously
    async fn download_assets() -> Result<()> {
        log::info!("Starting asset download...");
        
        // Create assets directory if it doesn't exist
        let assets_path = Path::new("assets");
        if !assets_path.exists() {
            std::fs::create_dir_all(assets_path)?;
            log::info!("Created assets directory");
        }

        // Initialize the downloader
        let downloader = AssetDownloader::new(assets_path);
        
        // Download essential sprites
        downloader.download_essential_sprites().await?;
        
        log::info!("Asset download completed successfully!");
        Ok(())
    }
}

impl GameState for MenuState {
    fn update(&mut self, _delta_time: f32) -> Result<()> {
        // Menu logic here
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<()> {
        log::debug!("MenuState render called, downloading: {}, complete: {}", self.downloading, self.download_complete);
        // Simple menu rendering - just clear to a different color for now
        if self.downloading {
            // Show downloading status with orange background
            canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 165, 0));
        } else if self.download_complete {
            // Show ready status with green background
            canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 128, 0));
        } else {
            // Default menu color
            canvas.set_draw_color(sdl2::pixels::Color::RGB(50, 50, 100));
        }
        canvas.clear();
        // TODO: Render actual menu text and options showing download status
        Ok(())
    }

    fn handle_key_down(&mut self, keycode: Keycode) -> Result<Option<StateId>> {
        match keycode {
            Keycode::Return | Keycode::Space => {
                if !self.downloading && !self.download_complete {
                    log::info!("Starting asset download before launching game...");
                    self.downloading = true;
                    
                    // Use tokio runtime to block on async operation
                    let rt = tokio::runtime::Runtime::new()
                        .map_err(|e| crate::util::Error::GameLogic(format!("Failed to create tokio runtime: {}", e)))?;
                    
                    match rt.block_on(Self::download_assets()) {
                        Ok(_) => {
                            self.downloading = false;
                            self.download_complete = true;
                            log::info!("Assets downloaded successfully! Starting game...");
                            return Ok(Some(StateId::Playing));
                        }
                        Err(e) => {
                            self.downloading = false;
                            log::error!("Failed to download assets: {}", e);
                            return Err(e);
                        }
                    }
                } else if self.download_complete {
                    log::info!("Assets already downloaded, starting game...");
                    return Ok(Some(StateId::Playing));
                } else {
                    log::info!("Download already in progress...");
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn name(&self) -> &str {
        "Menu"
    }
}

/// A simple playing state for testing
pub struct PlayingState {
    game_world: Option<crate::supertux::GameWorld>,
    
    initialized: bool,
}

impl PlayingState {
    pub fn new() -> Self {
        Self {
            game_world: None,
            
            initialized: false,
        }
    }
    
    fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        // Check if required texture files exist before initializing
        let tux_exists = std::path::Path::new("assets/sprites/creatures/tux/small/idle-0.png").exists();
        let platform_exists = std::path::Path::new("assets/sprites/tiles/blocks/brick0.png").exists();
        let ground_exists = std::path::Path::new("assets/sprites/tiles/blocks/bigblock.png").exists();
        
        if !tux_exists || !platform_exists || !ground_exists {
            log::warn!("Required texture files not found, skipping game world initialization");
            log::warn!("Tux: {}, Platform: {}, Ground: {}", tux_exists, platform_exists, ground_exists);
            return Ok(());
        }
        
        let mut game_world = crate::supertux::GameWorld::new();
        
        // Create the player at a starting position
        let player_position = crate::math::Vector2::new(250.0, 368.0); // On Platform1 surface (400-32=368)
        game_world.create_player(player_position, "tux".to_string());
        
        // Create some test platforms
        self.create_test_level(&mut game_world)?;
        
        self.game_world = Some(game_world);
        self.initialized = true;
        
        log::info!("Playing state initialized with test level");
        Ok(())
    }
    
    fn create_test_level(&self, game_world: &mut crate::supertux::GameWorld) -> Result<()> {
            use crate::physics::BodyType;
            use crate::collision::CollisionLayer;
            use crate::math::Vector2;
            
            // Create platforms manually to avoid borrow checker issues
            self.create_platform(game_world, "Ground".to_string(), Vector2::new(0.0, 500.0), Vector2::new(800.0, 100.0))?;
            self.create_platform(game_world, "Platform1".to_string(), Vector2::new(200.0, 400.0), Vector2::new(100.0, 20.0))?;
            self.create_platform(game_world, "Platform2".to_string(), Vector2::new(400.0, 350.0), Vector2::new(100.0, 20.0))?;
            
            Ok(())
        }
        
        fn create_platform(&self, game_world: &mut crate::supertux::GameWorld, name: String, position: crate::math::Vector2, size: crate::math::Vector2) -> Result<()> {use crate::object::{Transform, PhysicsComponent, SpriteComponent};
                use crate::physics::{PhysicsBody, BodyType};
                use crate::collision::CollisionLayer;
                use crate::sprite::Sprite;
                
                // Create the object first
                let object_id = game_world.object_manager_mut().create_object(name.clone());
                
                // Create physics body
                let physics_body = PhysicsBody::new(0, position, size, BodyType::Static);
                let body_id = game_world.physics_world_mut().add_body(physics_body);
                
                // Determine texture based on platform name
                let texture_name = if name.contains("Ground") {
                    "ground"
                } else {
                    "platform"
                };
                
                // Add components to the object
                if let Some(object) = game_world.object_manager_mut().get_object_mut(object_id) {
                    object.add_component(Transform::new(position));
                    object.add_component(PhysicsComponent::new(body_id, BodyType::Static, CollisionLayer::World));
                    
                    // Add sprite component
                    let sprite = Sprite::new(texture_name.to_string(), crate::math::Vector2::ZERO);
                    object.add_component(SpriteComponent::new(sprite));
                }
                
                Ok(())}
}

impl GameState for PlayingState {
    fn on_enter(&mut self) -> Result<()> {
        log::info!("Entering Playing state - initializing game world");
        if !self.initialized {
            self.initialize()?;
        }
        Ok(())
    }

    fn update(&mut self, delta_time: f32) -> Result<()> {
        // Default update - does nothing, use update_with_input instead
        Ok(())
    }

    fn update_with_input(&mut self, delta_time: f32, input_manager: &crate::control::InputManager) -> Result<()> {
        // Update game world only if initialized
        if let Some(ref mut game_world) = &mut self.game_world {
            game_world.update(input_manager, delta_time)?;
        }
        
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<()> {
            log::debug!("PlayingState render called, initialized: {}", self.initialized);
            
            // Only render if the state has been properly initialized
            if !self.initialized {
                // Clear screen with a simple color to indicate not ready
                canvas.set_draw_color(sdl2::pixels::Color::RGB(50, 50, 50)); // Dark gray
                canvas.clear();
                return Ok(());
            }
            
            // Clear screen with sky blue
            canvas.set_draw_color(sdl2::pixels::Color::RGB(135, 206, 235));
            canvas.clear();
            
            // Render game world only if it exists and we're initialized
            if let Some(ref game_world) = self.game_world {
                // Create a temporary texture manager for rendering
                let texture_creator = canvas.texture_creator();
                let mut texture_manager = crate::sprite::TextureManager::new(&texture_creator);
                
                // Load textures for this frame (in a real game, this would be cached)
                // Use downloaded SuperTux assets if they exist
                if std::path::Path::new("assets/sprites/creatures/tux/small/idle-0.png").exists() {
                    if let Err(e) = texture_manager.load_texture_from_file("tux", "assets/sprites/creatures/tux/small/idle-0.png") {
                        log::warn!("Failed to load tux texture: {}", e);
                    } else {
                        log::info!("Successfully loaded tux texture");
                    }
                } else {
                    log::warn!("Tux texture file does not exist");
                }
                if std::path::Path::new("assets/sprites/tiles/blocks/brick0.png").exists() {
                    if let Err(e) = texture_manager.load_texture_from_file("platform", "assets/sprites/tiles/blocks/brick0.png") {
                        log::warn!("Failed to load platform texture: {}", e);
                    }
                }
                if std::path::Path::new("assets/sprites/tiles/blocks/bigblock.png").exists() {
                    if let Err(e) = texture_manager.load_texture_from_file("ground", "assets/sprites/tiles/blocks/bigblock.png") {
                        log::warn!("Failed to load ground texture: {}", e);
                    }
                }
                
                game_world.render(canvas, &texture_manager)?;
            }
            
            Ok(())
        }

    fn handle_key_down(&mut self, keycode: Keycode) -> Result<Option<StateId>> {
        // Input processing is now handled by the engine's input manager
        match keycode {
            Keycode::P => {
                log::info!("Pausing game");
                // TODO: Transition to paused state
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_up(&mut self, keycode: Keycode) -> Result<()> {
        // Input processing is now handled by the engine's input manager
        let _ = keycode; // Suppress unused parameter warning
        Ok(())
    }

    fn name(&self) -> &str {
        "Playing"
    }
}
