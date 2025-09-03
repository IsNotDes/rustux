//! Input control system for RustUX

use crate::util::{Result, Error};
use crate::math::Vector2;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Game actions that can be triggered by input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameAction {
    // Player movement
    MoveLeft,
    MoveRight,
    Jump,
    Duck,
    Run,
    
    // Game controls
    Pause,
    Menu,
    Confirm,
    Cancel,
    
    // Debug actions
    ToggleDebug,
    Screenshot,
    
    // Menu navigation
    MenuUp,
    MenuDown,
    MenuLeft,
    MenuRight,
    MenuSelect,
    MenuBack,
}

/// Input device types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputDevice {
    Keyboard,
    Mouse,
    Gamepad(u32), // Gamepad ID
}

/// Input binding for mapping inputs to actions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputBinding {
    Key(String), // SDL2 keycode name
    MouseButton(u8),
    GamepadButton(u32, u8), // (gamepad_id, button)
    GamepadAxis(u32, u8, bool), // (gamepad_id, axis, positive_direction)
}

impl InputBinding {
    /// Create a keyboard binding
    pub fn key(keycode: sdl2::keyboard::Keycode) -> Self {
        Self::Key(keycode.name())
    }
    
    /// Create a mouse button binding
    pub fn mouse_button(button: sdl2::mouse::MouseButton) -> Self {
        Self::MouseButton(button as u8)
    }
    
    /// Create a gamepad button binding
    pub fn gamepad_button(gamepad_id: u32, button: u8) -> Self {
        Self::GamepadButton(gamepad_id, button)
    }
    
    /// Create a gamepad axis binding
    pub fn gamepad_axis(gamepad_id: u32, axis: u8, positive: bool) -> Self {
        Self::GamepadAxis(gamepad_id, axis, positive)
    }
}

/// Input state for tracking pressed keys and buttons
#[derive(Debug, Default)]
pub struct InputState {
    /// Currently pressed keys
    pressed_keys: HashSet<sdl2::keyboard::Keycode>,
    /// Keys that were just pressed this frame
    just_pressed_keys: HashSet<sdl2::keyboard::Keycode>,
    /// Keys that were just released this frame
    just_released_keys: HashSet<sdl2::keyboard::Keycode>,
    
    /// Currently pressed mouse buttons
    pressed_mouse_buttons: HashSet<sdl2::mouse::MouseButton>,
    /// Mouse buttons that were just pressed this frame
    just_pressed_mouse_buttons: HashSet<sdl2::mouse::MouseButton>,
    /// Mouse buttons that were just released this frame
    just_released_mouse_buttons: HashSet<sdl2::mouse::MouseButton>,
    
    /// Current mouse position
    mouse_position: Vector2,
    /// Mouse movement delta
    mouse_delta: Vector2,
    /// Mouse wheel delta
    mouse_wheel_delta: Vector2,
    
    /// Gamepad button states
    gamepad_buttons: HashMap<(u32, u8), bool>,
    /// Gamepad axis values
    gamepad_axes: HashMap<(u32, u8), f32>,
}

impl InputState {
    /// Create a new input state
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Clear frame-specific input data
    pub fn clear_frame_data(&mut self) {
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.just_pressed_mouse_buttons.clear();
        self.just_released_mouse_buttons.clear();
        self.mouse_delta = Vector2::ZERO;
        self.mouse_wheel_delta = Vector2::ZERO;
    }
    
    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: sdl2::keyboard::Keycode) -> bool {
        self.pressed_keys.contains(&key)
    }
    
    /// Check if a key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: sdl2::keyboard::Keycode) -> bool {
        self.just_pressed_keys.contains(&key)
    }
    
    /// Check if a key was just released this frame
    pub fn is_key_just_released(&self, key: sdl2::keyboard::Keycode) -> bool {
        self.just_released_keys.contains(&key)
    }
    
    /// Check if a mouse button is currently pressed
    pub fn is_mouse_button_pressed(&self, button: sdl2::mouse::MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }
    
    /// Check if a mouse button was just pressed this frame
    pub fn is_mouse_button_just_pressed(&self, button: sdl2::mouse::MouseButton) -> bool {
        self.just_pressed_mouse_buttons.contains(&button)
    }
    
    /// Check if a mouse button was just released this frame
    pub fn is_mouse_button_just_released(&self, button: sdl2::mouse::MouseButton) -> bool {
        self.just_released_mouse_buttons.contains(&button)
    }
    
    /// Get the current mouse position
    pub fn mouse_position(&self) -> Vector2 {
        self.mouse_position
    }
    
    /// Get the mouse movement delta
    pub fn mouse_delta(&self) -> Vector2 {
        self.mouse_delta
    }
    
    /// Get the mouse wheel delta
    pub fn mouse_wheel_delta(&self) -> Vector2 {
        self.mouse_wheel_delta
    }
    
    /// Get gamepad axis value
    pub fn gamepad_axis(&self, gamepad_id: u32, axis: u8) -> f32 {
        self.gamepad_axes.get(&(gamepad_id, axis)).copied().unwrap_or(0.0)
    }
    
    /// Check if gamepad button is pressed
    pub fn is_gamepad_button_pressed(&self, gamepad_id: u32, button: u8) -> bool {
        self.gamepad_buttons.get(&(gamepad_id, button)).copied().unwrap_or(false)
    }
}

/// Input configuration for key bindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Action to input binding mappings
    pub bindings: HashMap<GameAction, Vec<InputBinding>>,
    /// Mouse sensitivity
    pub mouse_sensitivity: f32,
    /// Gamepad deadzone
    pub gamepad_deadzone: f32,
}

impl Default for InputConfig {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        // Default keyboard bindings
        bindings.insert(GameAction::MoveLeft, vec![
            InputBinding::key(sdl2::keyboard::Keycode::Left),
            InputBinding::key(sdl2::keyboard::Keycode::A),]);
        bindings.insert(GameAction::MoveRight, vec![
            InputBinding::key(sdl2::keyboard::Keycode::Right),
            InputBinding::key(sdl2::keyboard::Keycode::D),
        ]);
        bindings.insert(GameAction::Jump, vec![
            InputBinding::key(sdl2::keyboard::Keycode::Space),
            InputBinding::key(sdl2::keyboard::Keycode::Up),
            InputBinding::key(sdl2::keyboard::Keycode::W),
        ]);
        bindings.insert(GameAction::Duck, vec![
            InputBinding::key(sdl2::keyboard::Keycode::Down),
            InputBinding::key(sdl2::keyboard::Keycode::S),
        ]);
        bindings.insert(GameAction::Run, vec![
            InputBinding::key(sdl2::keyboard::Keycode::LShift),
            InputBinding::key(sdl2::keyboard::Keycode::RShift),
        ]);
        // Game controls
        bindings.insert(GameAction::Pause, vec![
            InputBinding::key(sdl2::keyboard::Keycode::P),
            InputBinding::key(sdl2::keyboard::Keycode::Escape),
        ]);
        bindings.insert(GameAction::Menu, vec![
            InputBinding::key(sdl2::keyboard::Keycode::Escape),
        ]);
        bindings.insert(GameAction::Confirm, vec![
            InputBinding::key(sdl2::keyboard::Keycode::Return),
            InputBinding::key(sdl2::keyboard::Keycode::Space),
        ]);
        bindings.insert(GameAction::Cancel, vec![
            InputBinding::key(sdl2::keyboard::Keycode::Escape),
        ]);
        
        // Debug actions
        bindings.insert(GameAction::ToggleDebug, vec![
            InputBinding::key(sdl2::keyboard::Keycode::F3),
        ]);
        bindings.insert(GameAction::Screenshot, vec![
            InputBinding::key(sdl2::keyboard::Keycode::F12),
        ]);
        
        // Menu navigation
        bindings.insert(GameAction::MenuUp, vec![
            InputBinding::key(sdl2::keyboard::Keycode::Up),
            InputBinding::key(sdl2::keyboard::Keycode::W),
        ]);
        bindings.insert(GameAction::MenuDown, vec![
            InputBinding::key(sdl2::keyboard::Keycode::Down),
            InputBinding::key(sdl2::keyboard::Keycode::S),
        ]);
        bindings.insert(GameAction::MenuLeft, vec![
            InputBinding::key(sdl2::keyboard::Keycode::Left),
            InputBinding::key(sdl2::keyboard::Keycode::A),
        ]);
        bindings.insert(GameAction::MenuRight, vec![
            InputBinding::key(sdl2::keyboard::Keycode::Right),
            InputBinding::key(sdl2::keyboard::Keycode::D),
        ]);
        bindings.insert(GameAction::MenuSelect, vec![
            InputBinding::key(sdl2::keyboard::Keycode::Return),
            InputBinding::key(sdl2::keyboard::Keycode::Space),
        ]);
        bindings.insert(GameAction::MenuBack, vec![
            InputBinding::key(sdl2::keyboard::Keycode::Escape),
        ]);
        
        Self {
            bindings,
            mouse_sensitivity: 1.0,
            gamepad_deadzone: 0.1,
        }
    }
}

/// Input manager for handling keyboard, mouse, and gamepad input
pub struct InputManager {
    /// Current input state
    state: InputState,
    /// Input configuration
    config: InputConfig,
}

impl InputManager {
    /// Create a new input manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: InputState::new(),
            config: InputConfig::default(),
        })
    }
    
    /// Initialize with SDL2 context (placeholder for future gamepad support)
    pub fn init_with_sdl(&mut self, _sdl_context: &sdl2::Sdl) -> Result<()> {
        // Future gamepad initialization will go here
        Ok(())
    }
    
    /// Process an SDL2 event
    pub fn process_event(&mut self, event: &sdl2::event::Event) {
        match event {
            sdl2::event::Event::KeyDown { keycode: Some(keycode), repeat: false, .. } => {
                self.state.pressed_keys.insert(*keycode);
                self.state.just_pressed_keys.insert(*keycode);
            }
            sdl2::event::Event::KeyUp { keycode: Some(keycode), .. } => {
                self.state.pressed_keys.remove(keycode);
                self.state.just_released_keys.insert(*keycode);
            }
            sdl2::event::Event::MouseButtonDown { mouse_btn, .. } => {
                self.state.pressed_mouse_buttons.insert(*mouse_btn);
                self.state.just_pressed_mouse_buttons.insert(*mouse_btn);
            }
            sdl2::event::Event::MouseButtonUp { mouse_btn, .. } => {
                self.state.pressed_mouse_buttons.remove(mouse_btn);
                self.state.just_released_mouse_buttons.insert(*mouse_btn);
            }
            sdl2::event::Event::MouseMotion { x, y, xrel, yrel, .. } => {
                self.state.mouse_position = Vector2::new(*x as f32, *y as f32);
                self.state.mouse_delta = Vector2::new(*xrel as f32, *yrel as f32) * self.config.mouse_sensitivity;
            }
            sdl2::event::Event::MouseWheel { x, y, .. } => {
                self.state.mouse_wheel_delta = Vector2::new(*x as f32, *y as f32);
            }
            _ => {}
        }
    }
    
    /// Update input state (call once per frame)
    pub fn update(&mut self) {
        self.state.clear_frame_data();
    }
    
    /// Check if an action is currently active
    pub fn is_action_pressed(&self, action: GameAction) -> bool {
        if let Some(bindings) = self.config.bindings.get(&action) {
            for binding in bindings {
                if self.is_binding_pressed(binding) {
                    return true;
                }
            }
        }
        false
    }
    
    /// Check if an action was just activated this frame
    pub fn is_action_just_pressed(&self, action: GameAction) -> bool {
        if let Some(bindings) = self.config.bindings.get(&action) {
            for binding in bindings {
                if self.is_binding_just_pressed(binding) {
                    return true;
                }
            }
        }
        false
    }
    
    /// Check if an action was just deactivated this frame
    pub fn is_action_just_released(&self, action: GameAction) -> bool {
        if let Some(bindings) = self.config.bindings.get(&action) {
            for binding in bindings {
                if self.is_binding_just_released(binding) {
                    return true;
                }
            }
        }
        false
    }
    
    /// Get the strength of an action (for analog inputs)
    pub fn get_action_strength(&self, action: GameAction) -> f32 {
        if let Some(bindings) = self.config.bindings.get(&action) {
            let mut max_strength = 0.0;
            for binding in bindings {
                let strength = self.get_binding_strength(binding);
                if strength > max_strength {
                    max_strength = strength;
                }
            }
            max_strength
        } else {
            0.0
        }
    }
    
    /// Check if a specific binding is pressed
    fn is_binding_pressed(&self, binding: &InputBinding) -> bool {
        match binding {
            InputBinding::Key(key_name) => {
                if let Some(keycode) = sdl2::keyboard::Keycode::from_name(key_name) {
                    self.state.is_key_pressed(keycode)
                } else {
                    false
                }
            }
            InputBinding::MouseButton(button) => {
                let mouse_button = match *button {
                    1 => sdl2::mouse::MouseButton::Left,
                    2 => sdl2::mouse::MouseButton::Middle,
                    3 => sdl2::mouse::MouseButton::Right,
                    4 => sdl2::mouse::MouseButton::X1,
                    5 => sdl2::mouse::MouseButton::X2,
                    _ => return false,
                };
                self.state.is_mouse_button_pressed(mouse_button)
            }
            InputBinding::GamepadButton(gamepad_id, button) => {
                self.state.is_gamepad_button_pressed(*gamepad_id, *button)
            }
            InputBinding::GamepadAxis(gamepad_id, axis, positive) => {
                let value = self.state.gamepad_axis(*gamepad_id, *axis);
                if *positive {
                    value > self.config.gamepad_deadzone
                } else {
                    value < -self.config.gamepad_deadzone
                }
            }
        }
    }
    
    /// Check if a specific binding was just pressed
    fn is_binding_just_pressed(&self, binding: &InputBinding) -> bool {
        match binding {
            InputBinding::Key(key_name) => {
                if let Some(keycode) = sdl2::keyboard::Keycode::from_name(key_name) {
                    self.state.is_key_just_pressed(keycode)
                } else {
                    false
                }
            }
            InputBinding::MouseButton(button) => {
                let mouse_button = match *button {
                    1 => sdl2::mouse::MouseButton::Left,
                    2 => sdl2::mouse::MouseButton::Middle,
                    3 => sdl2::mouse::MouseButton::Right,
                    4 => sdl2::mouse::MouseButton::X1,
                    5 => sdl2::mouse::MouseButton::X2,
                    _ => return false,
                };
                self.state.is_mouse_button_just_pressed(mouse_button)
            }
            InputBinding::GamepadButton(_gamepad_id, _button) => {
                // TODO: Implement gamepad just pressed detection
                false
            }
            InputBinding::GamepadAxis(_gamepad_id, _axis, _positive) => {
                // TODO: Implement gamepad axis just pressed detection
                false
            }
        }
    }
    
    /// Check if a specific binding was just released
    fn is_binding_just_released(&self, binding: &InputBinding) -> bool {
        match binding {
            InputBinding::Key(key_name) => {
                if let Some(keycode) = sdl2::keyboard::Keycode::from_name(key_name) {
                    self.state.is_key_just_released(keycode)
                } else {
                    false
                }
            }
            InputBinding::MouseButton(button) => {
                let mouse_button = match *button {
                    1 => sdl2::mouse::MouseButton::Left,
                    2 => sdl2::mouse::MouseButton::Middle,
                    3 => sdl2::mouse::MouseButton::Right,
                    4 => sdl2::mouse::MouseButton::X1,
                    5 => sdl2::mouse::MouseButton::X2,
                    _ => return false,
                };
                self.state.is_mouse_button_just_released(mouse_button)
            }
            InputBinding::GamepadButton(_gamepad_id, _button) => {
                // TODO: Implement gamepad just released detection
                false
            }
            InputBinding::GamepadAxis(_gamepad_id, _axis, _positive) => {
                // TODO: Implement gamepad axis just released detection
                false
            }
        }
    }
    
    /// Get the strength of a specific binding
    fn get_binding_strength(&self, binding: &InputBinding) -> f32 {
        match binding {
            InputBinding::Key(_) | InputBinding::MouseButton(_) | InputBinding::GamepadButton(_, _) => {
                if self.is_binding_pressed(binding) { 1.0 } else { 0.0 }
            }
            InputBinding::GamepadAxis(gamepad_id, axis, positive) => {
                let value = self.state.gamepad_axis(*gamepad_id, *axis);
                if *positive {
                    value.max(0.0)
                } else {
                    (-value).max(0.0)
                }
            }
        }
    }
    
    /// Get the input state
    pub fn state(&self) -> &InputState {
        &self.state
    }
    
    /// Get the input configuration
    pub fn config(&self) -> &InputConfig {
        &self.config
    }
    
    /// Get the input configuration mutably
    pub fn config_mut(&mut self) -> &mut InputConfig {
        &mut self.config
    }
    
    /// Load input configuration from file
    pub fn load_config<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        self.config = toml::from_str(&content)
            .map_err(|e| Error::Unknown(format!("Failed to parse input config: {}", e)))?;
        Ok(())
    }
    
    /// Save input configuration to file
    pub fn save_config<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| Error::Unknown(format!("Failed to serialize input config: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Add a binding for an action
    pub fn add_binding(&mut self, action: GameAction, binding: InputBinding) {
        self.config.bindings.entry(action).or_insert_with(Vec::new).push(binding);
    }
    
    /// Remove a binding for an action
    pub fn remove_binding(&mut self, action: GameAction, binding: &InputBinding) {
        if let Some(bindings) = self.config.bindings.get_mut(&action) {
            bindings.retain(|b| b != binding);
        }
    }
    
    /// Clear all bindings for an action
    pub fn clear_bindings(&mut self, action: GameAction) {
        self.config.bindings.remove(&action);
    }
    
    /// Get movement vector from input (for2D movement)
    pub fn get_movement_vector(&self) -> Vector2 {
        let mut movement = Vector2::ZERO;
        
        if self.is_action_pressed(GameAction::MoveLeft) {
            movement.x -= 1.0;
        }
        if self.is_action_pressed(GameAction::MoveRight) {
            movement.x += 1.0;
        }
        // Normalize diagonal movement
        if movement.length() > 1.0 {
            movement = movement.normalize();
        }
        
        movement
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new().expect("Failed to create InputManager")
    }
}

/// Input utilities
pub mod utils {
    use super::*;
    
    /// Convert SDL2 keycode to string
    pub fn keycode_to_string(keycode: sdl2::keyboard::Keycode) -> String {
        keycode.name()
    }
    
    /// Convert string to SDL2 keycode
    pub fn string_to_keycode(name: &str) -> Option<sdl2::keyboard::Keycode> {
        sdl2::keyboard::Keycode::from_name(name)
    }
    
    /// Get human-readable name for a game action
    pub fn action_name(action: GameAction) -> &'static str {
        match action {
            GameAction::MoveLeft => "Move Left",
            GameAction::MoveRight => "Move Right",
            GameAction::Jump => "Jump",
            GameAction::Duck => "Duck",
            GameAction::Run => "Run",
            GameAction::Pause => "Pause",
            GameAction::Menu => "Menu",
            GameAction::Confirm => "Confirm",
            GameAction::Cancel => "Cancel",
            GameAction::ToggleDebug => "Toggle Debug",
            GameAction::Screenshot => "Screenshot",
            GameAction::MenuUp => "Menu Up",
            GameAction::MenuDown => "Menu Down",
            GameAction::MenuLeft => "Menu Left",
            GameAction::MenuRight => "Menu Right",
            GameAction::MenuSelect => "Menu Select",
            GameAction::MenuBack => "Menu Back",
        }
    }
    
    /// Get human-readable description for an input binding
    pub fn binding_description(binding: &InputBinding) -> String {
        match binding {
            InputBinding::Key(key_name) => format!("Key: {}", key_name),
            InputBinding::MouseButton(button) => format!("Mouse Button {}", button),
            InputBinding::GamepadButton(gamepad_id, button) => {
                format!("Gamepad {} Button {}", gamepad_id, button)
            }
            InputBinding::GamepadAxis(gamepad_id, axis, positive) => {
                format!("Gamepad {} Axis {} {}", gamepad_id, axis, if *positive { "+" } else { "-" })
            }
        }
    }
}