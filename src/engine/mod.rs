//! Core game engine for RustUX

use crate::util::Result;
use crate::math::Vector2;
use crate::config::*;
use crate::sprite::TextureManager;
use crate::audio::AudioManager;
use crate::control::InputManager;
use std::time::{Duration, Instant};

pub mod game_state;
pub mod resource_manager;

pub use game_state::{GameState, GameStateManager, MenuState, PlayingState, StateId};
pub use resource_manager::ResourceManager;

/// Main game engine that manages the game loop and systems
pub struct Engine {
    /// SDL2 context
    sdl_context: sdl2::Sdl,
    /// Video subsystem
    video_subsystem: sdl2::VideoSubsystem,
    /// Audio subsystem
    audio_subsystem: sdl2::AudioSubsystem,
    /// Canvas for rendering
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    /// Event pump for handling input
    event_pump: sdl2::EventPump,
    /// Texture creator for loading textures
    texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    /// Audio manager for sound and music
    audio_manager: AudioManager,
    /// Input manager for handling controls
    input_manager: InputManager,
    /// Resource manager
    resource_manager: ResourceManager,
    /// Game state manager
    state_manager: GameStateManager,
    /// Whether the engine is running
    running: bool,
    /// Target frame time
    target_frame_time: Duration,
    /// Last frame time
    last_frame_time: Instant,
    /// Delta time for current frame
    delta_time: f32,
}

impl Engine {
    /// Create a new engine instance
    pub fn new() -> Result<Self> {
        // Initialize SDL2
        let sdl_context = sdl2::init().map_err(|e| crate::util::Error::Sdl2(e))?;
        let video_subsystem = sdl_context.video().map_err(|e| crate::util::Error::Sdl2(e))?;
        let audio_subsystem = sdl_context.audio().map_err(|e| crate::util::Error::Sdl2(e))?;

        // Create window
        let window = video_subsystem
            .window("RustUX - SuperTux in Rust", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| crate::util::Error::Sdl2(e.to_string()))?;

        // Create canvas
        let canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|e| crate::util::Error::Sdl2(e.to_string()))?;

        // Create event pump
        let event_pump = sdl_context.event_pump().map_err(|e| crate::util::Error::Sdl2(e))?;

        // Create texture creator
        let texture_creator = canvas.texture_creator();

        // Initialize audio manager
        let audio_manager = AudioManager::new(audio_subsystem.clone())
            .map_err(|e| crate::util::Error::Audio(format!("Failed to initialize audio: {}", e)))?;

        // Initialize input manager
        let mut input_manager = InputManager::new()?;
        input_manager.init_with_sdl(&sdl_context)?;

        // Initialize subsystems
        let resource_manager = ResourceManager::new()?;
        let state_manager = GameStateManager::new();

        let target_frame_time = Duration::from_nanos(1_000_000_000 / TARGET_FPS as u64);

        Ok(Self {
            sdl_context,
            video_subsystem,
            audio_subsystem,
            canvas,
            event_pump,
            texture_creator,
            audio_manager,
            input_manager,
            resource_manager,
            state_manager,
            running: false,
            target_frame_time,
            last_frame_time: Instant::now(),
            delta_time: 0.0,
        })
    }

    /// Start the main game loop
    pub fn run(&mut self) -> Result<()> {
        log::info!("Starting RustUX engine");
        self.running = true;
        self.last_frame_time = Instant::now();

        while self.running {
            let frame_start = Instant::now();
            // Calculate delta time
            self.delta_time = self.last_frame_time.elapsed().as_secs_f32();
            self.last_frame_time = frame_start;

            // Handle events
            self.handle_events()?;

            // Update game logic
            self.update()?;

            // Render frame
            self.render()?;

            // Frame rate limiting
            let frame_time = frame_start.elapsed();
            if frame_time < self.target_frame_time {
                std::thread::sleep(self.target_frame_time - frame_time);
            }
        }

        log::info!("RustUX engine stopped");
        Ok(())
    }

    /// Handle SDL2 events
    fn handle_events(&mut self) -> Result<()> {
        for event in self.event_pump.poll_iter() {
            // Process event with input manager
            self.input_manager.process_event(&event);
            
            match event {
                sdl2::event::Event::Quit { .. } => {
                    self.running = false;
                }
                sdl2::event::Event::KeyDown { keycode: Some(keycode), .. } => {
                                    if keycode == sdl2::keyboard::Keycode::Escape {
                                        self.running = false;
                                    }
                                    // Handle state transitions based on current state and input
                                    if let Some(current_state_id) = self.state_manager.current_state_id() {
                                        match (current_state_id, keycode) {
                                            (StateId::Playing, sdl2::keyboard::Keycode::P) => {
                                                log::info!("Pausing game - transitioning to Menu state");
                                                self.state_manager.set_state(StateId::Menu)?;
                                            }
                                            _ => {}
                                        }
                                    }
                                    
                                    // Forward to current game state
                                    if let Some(state) = self.state_manager.current_state_mut() {
                                        if let Some(next_state) = state.handle_key_down(keycode)? {
                                            log::info!("State transition requested: {:?}", next_state);
                                            self.state_manager.set_state(next_state)?;
                                        }
                                    }
                                }
                sdl2::event::Event::KeyUp { keycode: Some(keycode), .. } => {
                    // Forward to current game state
                    if let Some(state) = self.state_manager.current_state_mut() {
                        state.handle_key_up(keycode)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Update game logic
    fn update(&mut self) -> Result<()> {
        // Update input manager
        self.input_manager.update();
        
        // Update current game state
        if let Some(state) = self.state_manager.current_state_mut() {
            state.update_with_input(self.delta_time, &self.input_manager)?;
        }

        // Handle state transitions
        self.state_manager.process_transitions()?;

        Ok(())
    }

    /// Render the current frame
    fn render(&mut self) -> Result<()> {
        // Clear screen
        self.canvas.set_draw_color(sdl2::pixels::Color::RGB(135, 206, 235)); // Sky blue
        self.canvas.clear();

        // Render current game state
        if let Some(state) = self.state_manager.current_state() {
            state.render(&mut self.canvas)?;
        }

        // Present frame
        self.canvas.present();
        Ok(())
    }

    /// Stop the engine
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Get the current delta time
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    /// Get the resource manager
    pub fn resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    /// Get the resource manager mutably
    pub fn resource_manager_mut(&mut self) -> &mut ResourceManager {
        &mut self.resource_manager
    }

    /// Get the state manager
    pub fn state_manager(&self) -> &GameStateManager {
        &self.state_manager
    }

    /// Get the state manager mutably
    pub fn state_manager_mut(&mut self) -> &mut GameStateManager {
        &mut self.state_manager
    }

    /// Get the texture creator
    pub fn texture_creator(&self) -> &sdl2::render::TextureCreator<sdl2::video::WindowContext> {
        &self.texture_creator
    }

    /// Get the audio manager
    pub fn audio_manager(&self) -> &AudioManager {
        &self.audio_manager
    }

    /// Get the audio manager mutably
    pub fn audio_manager_mut(&mut self) -> &mut AudioManager {
        &mut self.audio_manager
    }

    /// Get the input manager
    pub fn input_manager(&self) -> &InputManager {
        &self.input_manager
    }

    /// Get the input manager mutably
    pub fn input_manager_mut(&mut self) -> &mut InputManager {
        &mut self.input_manager
    }

    /// Create a texture manager for the current frame
    pub fn create_texture_manager(&self) -> TextureManager {
        TextureManager::new(&self.texture_creator)
    }

    /// Get the canvas size
    pub fn canvas_size(&self) -> Vector2 {
        let (w, h) = self.canvas.output_size().unwrap_or((SCREEN_WIDTH, SCREEN_HEIGHT));
        Vector2::new(w as f32, h as f32)
    }

    /// Get the canvas for rendering
    pub fn canvas(&mut self) -> &mut sdl2::render::Canvas<sdl2::video::Window> {
        &mut self.canvas
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        log::info!("Shutting down RustUX engine");
    }
}