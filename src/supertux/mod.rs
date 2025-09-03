//! Main SuperTux game logic for RustUX

use crate::object::{GameObject, GameObjectManager, Component, Transform, SpriteComponent, Health, ObjectId};
use crate::physics::{PhysicsWorld, PhysicsBody, BodyType};
use crate::collision::CollisionLayer;
use crate::control::{InputManager, GameAction};
use crate::sprite::{Sprite, Animation, animations};
use crate::math::{Vector2, Rect};
use crate::util::Result;
use std::any::Any;

/// Player state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Idle,
    Walking,
    Running,
    Jumping,
    Falling,
    Ducking,
    Climbing,
    Dead,
}

/// Player controller component
#[derive(Debug, Clone)]
pub struct PlayerController {
    pub state: PlayerState,
    pub move_speed: f32,
    pub run_speed: f32,
    pub jump_velocity: f32,
    pub can_jump: bool,
    pub on_ground: bool,
    pub facing_right: bool,
    pub invulnerable: bool,
    pub invulnerability_time: f32,
    pub lives: i32,
    pub score: i32,
    pub coins: i32,
}

impl PlayerController {
    pub fn new() -> Self {
        Self {
            state: PlayerState::Idle,
            move_speed: 150.0,
            run_speed: 250.0,
            jump_velocity: -400.0,
            can_jump: true,
            on_ground: false,
            facing_right: true,
            invulnerable: false,
            invulnerability_time: 0.0,
            lives: 3,
            score: 0,
            coins: 0,
        }
    }

    pub fn jump(&mut self) {
        if self.can_jump && self.on_ground {
            self.state = PlayerState::Jumping;
            self.can_jump = false;
        }
    }

    pub fn start_moving_left(&mut self) {
        self.facing_right = false;
        if self.on_ground {
            self.state = PlayerState::Walking;
        }
    }

    pub fn start_moving_right(&mut self) {
        self.facing_right = true;
        if self.on_ground {
            self.state = PlayerState::Walking;
        }
    }

    pub fn stop_horizontal_movement(&mut self) {
        if self.on_ground && matches!(self.state, PlayerState::Walking | PlayerState::Running) {
            self.state = PlayerState::Idle;
        }
    }

    pub fn start_ducking(&mut self) {
        if self.on_ground {
            self.state = PlayerState::Ducking;
        }
    }

    pub fn stop_ducking(&mut self) {
        if matches!(self.state, PlayerState::Ducking) {
            self.state = PlayerState::Idle;
        }
    }

    pub fn take_damage(&mut self) {
        if !self.invulnerable {
            self.lives -= 1;
            self.invulnerable = true;
            self.invulnerability_time = 2.0; // 2 seconds of invulnerability
            
            if self.lives <= 0 {
                self.state = PlayerState::Dead;
            }
        }
    }

    pub fn add_score(&mut self, points: i32) {
        self.score += points;
    }

    pub fn add_coin(&mut self) {
        self.coins += 1;
        self.add_score(200); // Coins are worth 200 points
        
        // Extra life every 100 coins
        if self.coins % 100 == 0 {
            self.lives += 1;
        }
    }

    pub fn is_alive(&self) -> bool {
        !matches!(self.state, PlayerState::Dead)
    }
}

impl Component for PlayerController {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn clone_component(&self) -> Box<dyn Component> { Box::new(self.clone()) }
}

/// Player input system
pub struct PlayerInputSystem;

impl PlayerInputSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn update(
        &self,
        player_id: ObjectId,
        object_manager: &mut GameObjectManager,
        physics_world: &mut PhysicsWorld,
        input_manager: &InputManager,
        delta_time: f32,
    ) -> Result<()> {
        let player_obj = match object_manager.get_object_mut(player_id) {
            Some(obj) => obj,
            None => return Ok(()),
        };

        if !player_obj.active {
            return Ok(());
        }

        // Get physics body ID first to avoid borrowing conflicts
        let body_id = {
            let physics_comp = match player_obj.get_component::<crate::object::PhysicsComponent>() {
                Some(comp) => comp.body_id,
                None => return Ok(()),
            };
            physics_comp
        };

        let controller = match player_obj.get_component_mut::<PlayerController>() {
            Some(c) => c,
            None => return Ok(()),
        };

        // Update invulnerability
        if controller.invulnerable {
            controller.invulnerability_time -= delta_time;
            if controller.invulnerability_time <= 0.0 {
                controller.invulnerable = false;
            }
        }

        // Don't process input if dead
        if !controller.is_alive() {
            return Ok(());
        }

        // Handle input
        let mut horizontal_input = 0.0;
        let mut is_running = false;

        if input_manager.is_action_pressed(GameAction::MoveLeft) {
            horizontal_input = -1.0;
            controller.start_moving_left();
            log::debug!("Player moving left");
        }
        if input_manager.is_action_pressed(GameAction::MoveRight) {
            horizontal_input = 1.0;
            controller.start_moving_right();
        }
        if horizontal_input == 0.0 {
            controller.stop_horizontal_movement();
        }

        if input_manager.is_action_pressed(GameAction::Run) {
            is_running = true;
            if horizontal_input != 0.0 && controller.on_ground {
                controller.state = PlayerState::Running;
            }
        }

        if input_manager.is_action_just_pressed(GameAction::Jump) {
            controller.jump();
            log::debug!("Player jumping");
        }

        if input_manager.is_action_pressed(GameAction::Duck) {
            controller.start_ducking();
        } else {
            controller.stop_ducking();
        }

        // Apply movement to physics body
        if let Some(body) = physics_world.get_body(body_id) {
            let current_velocity = body.velocity;
            let speed = if is_running { controller.run_speed } else { controller.move_speed };
            
            let mut new_velocity = current_velocity;
            
            // Horizontal movement
            if !matches!(controller.state, PlayerState::Ducking) {
                new_velocity.x = horizontal_input * speed;
            } else {
                new_velocity.x = 0.0; // Can't move while ducking
            }
            
            // Jumping
            if matches!(controller.state, PlayerState::Jumping) && controller.can_jump {
                new_velocity.y = controller.jump_velocity;
                controller.can_jump = false;
            }
            
            physics_world.set_body_velocity(body_id, new_velocity);
            log::debug!("Player velocity set to: ({:.2}, {:.2})", new_velocity.x, new_velocity.y);
        }

        // Update state based on physics
        if let Some(body) = physics_world.get_body(body_id) {
            controller.on_ground = body.on_ground;
            
            // Update state based on velocity and ground status
            if !controller.on_ground {
                if body.velocity.y < 0.0 {
                    controller.state = PlayerState::Jumping;
                } else {
                    controller.state = PlayerState::Falling;
                }
            } else {
                // Reset jump ability when on ground
                controller.can_jump = true;
                
                // Update ground state
                if horizontal_input == 0.0 && !matches!(controller.state, PlayerState::Ducking) {
                    controller.state = PlayerState::Idle;
                }
            }
        }

        Ok(())
    }
}

/// Player animation system
pub struct PlayerAnimationSystem;

impl PlayerAnimationSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn update(
        &self,
        player_id: ObjectId,
        object_manager: &mut GameObjectManager,) -> Result<()> {
        let player_obj = match object_manager.get_object_mut(player_id) {
            Some(obj) => obj,
            None => return Ok(()),
        };

        if !player_obj.active {
            return Ok(());
        }

        // Get facing direction first to avoid borrowing conflicts
        let facing_right = if let Some(controller) = player_obj.get_component::<PlayerController>() {
            controller.facing_right
        } else {
            return Ok(());
        };

        let state = if let Some(controller) = player_obj.get_component::<PlayerController>() {
            controller.state
        } else {
            return Ok(());
        };

        let sprite_comp = match player_obj.get_component_mut::<SpriteComponent>() {
            Some(s) => s,
            None => return Ok(()),
        };

        // Update sprite flip based on facing direction
        sprite_comp.sprite.flip_horizontal = !facing_right;

        // Set animation based on state
        match state {
            PlayerState::Idle => {
                // Set idle animation
                let idle_rect = Rect::new(0.0, 0.0, 32.0, 32.0);
                sprite_comp.sprite.set_animation(animations::idle(idle_rect));
            }
            PlayerState::Walking => {
                // Set walking animation
                sprite_comp.sprite.set_animation(animations::walk(32.0, 32.0, 4));
            }
            PlayerState::Running => {
                // Set running animation (faster walking)
                let mut run_anim = animations::walk(32.0, 32.0, 4);
                // Make it faster by reducing frame duration
                for frame in &mut run_anim.frames {
                    frame.duration *= 0.7;
                }
                sprite_comp.sprite.set_animation(run_anim);
            }
            PlayerState::Jumping | PlayerState::Falling => {
                // Set jump animation
                let jump_rect = Rect::new(128.0, 0.0, 32.0, 32.0);
                sprite_comp.sprite.set_animation(animations::jump(jump_rect));
            }
            PlayerState::Ducking => {
                // Set ducking animation
                let duck_rect = Rect::new(160.0, 0.0, 32.0, 24.0);
                sprite_comp.sprite.set_animation(animations::idle(duck_rect));
            }
            PlayerState::Dead => {
                // Set death animation
                let death_rect = Rect::new(192.0, 0.0, 32.0, 32.0);
                sprite_comp.sprite.set_animation(animations::idle(death_rect));
            }
            _ => {}
        }

        Ok(())
    }
}

/// Game world containing all game objects and systems
pub struct GameWorld {
    object_manager: GameObjectManager,
    physics_world: PhysicsWorld,
    player_id: Option<ObjectId>,
    player_input_system: PlayerInputSystem,
    player_animation_system: PlayerAnimationSystem,
    camera_position: Vector2,
    world_bounds: Rect,
}

impl GameWorld {
    pub fn new() -> Self {
        Self {
            object_manager: GameObjectManager::new(),
            physics_world: PhysicsWorld::new(),
            player_id: None,
            player_input_system: PlayerInputSystem::new(),
            player_animation_system: PlayerAnimationSystem::new(),
            camera_position: Vector2::ZERO,
            world_bounds: Rect::new(0.0, 0.0, 2048.0, 768.0), // Default world size
        }
    }

    /// Create the player character
    pub fn create_player(&mut self, position: Vector2, texture_name: String) -> ObjectId {
        // Create physics body for player
        let player_body = PhysicsBody::new(0, position, Vector2::new(32.0, 32.0), BodyType::Dynamic);
        let body_id = self.physics_world.add_body(player_body);

        // Create player object
        let player_id = self.object_manager.create_object("Tux".to_string());
        if let Some(player_obj) = self.object_manager.get_object_mut(player_id) {
            // Add components
            player_obj.add_component(Transform::new(position));
            let mut sprite = Sprite::new("tux".to_string(), Vector2::ZERO);
            // Set a source rectangle for the idle frame
            sprite.set_source_rect(crate::math::Rect::new(0.0, 0.0, 32.0, 32.0));
            log::info!("Created Tux sprite with source rect: {:?}", sprite.get_source_rect());
            player_obj.add_component(SpriteComponent::new(sprite));
            
            player_obj.add_component(crate::object::PhysicsComponent::new(
                body_id,
                BodyType::Dynamic,
                CollisionLayer::Player,
            ));
            
            player_obj.add_component(PlayerController::new());
            player_obj.add_component(Health::new(1)); // Tux has 1 HP (power-ups can increase this)
            
            player_obj.tag = "player".to_string();}

        self.player_id = Some(player_id);
        player_id
    }

    /// Update the game world
    pub fn update(&mut self, input_manager: &InputManager, delta_time: f32) -> Result<()> {
        // Update physics
        self.physics_world.update(delta_time);

        // Update player input
        if let Some(player_id) = self.player_id {
            self.player_input_system.update(
                player_id,
                &mut self.object_manager,
                &mut self.physics_world,
                input_manager,
                delta_time,
            )?;

            self.player_animation_system.update(player_id, &mut self.object_manager)?;
        }

        // Sync object positions from physics
        self.object_manager.sync_from_physics(&self.physics_world)?;

        // Update all game objects
        self.object_manager.update(delta_time)?;

        // Update camera to follow player
        self.update_camera();

        Ok(())
    }

    /// Update camera to follow the player
    fn update_camera(&mut self) {
        if let Some(player_id) = self.player_id {
            if let Some(player_obj) = self.object_manager.get_object(player_id) {
                let player_pos = player_obj.position();
                // Center camera on player with some offset
                self.camera_position.x = player_pos.x -400.0; // Half screen width
                self.camera_position.y = player_pos.y - 300.0; // Slightly above center
                
                // Clamp camera to world bounds
                self.camera_position.x = self.camera_position.x.max(0.0)
                    .min(self.world_bounds.width -800.0); // Screen width
                self.camera_position.y = self.camera_position.y.max(0.0)
                    .min(self.world_bounds.height - 600.0); // Screen height
            }
        }
    }

    /// Render the game world
    pub fn render(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        texture_manager: &crate::sprite::TextureManager,
    ) -> Result<()> {
        // TODO: Apply camera transform to rendering
        self.object_manager.render(canvas, texture_manager)
    }

    /// Get the player object
    pub fn get_player(&self) -> Option<&GameObject> {
        self.player_id.and_then(|id| self.object_manager.get_object(id))
    }

    /// Get the player object (mutable)
    pub fn get_player_mut(&mut self) -> Option<&mut GameObject> {
        self.player_id.and_then(|id| self.object_manager.get_object_mut(id))
    }

    /// Get the player ID
    pub fn get_player_id(&self) -> Option<ObjectId> {
        self.player_id
    }

    /// Get the object manager
    pub fn object_manager(&self) -> &GameObjectManager {
        &self.object_manager
    }

    /// Get the object manager (mutable)
    pub fn object_manager_mut(&mut self) -> &mut GameObjectManager {
        &mut self.object_manager
    }

    /// Get the physics world
    pub fn physics_world(&self) -> &PhysicsWorld {
        &self.physics_world
    }

    /// Get the physics world (mutable)
    pub fn physics_world_mut(&mut self) -> &mut PhysicsWorld {
        &mut self.physics_world
    }

    /// Get the camera position
    pub fn camera_position(&self) -> Vector2 {
        self.camera_position
    }

    /// Set the world bounds
    pub fn set_world_bounds(&mut self, bounds: Rect) {
        self.world_bounds = bounds;
    }

    /// Get the world bounds
    pub fn world_bounds(&self) -> Rect {
        self.world_bounds
    }

    /// Check if the player is alive
    pub fn is_player_alive(&self) -> bool {
        self.get_player()
            .and_then(|p| p.get_component::<PlayerController>())
            .map(|c| c.is_alive())
            .unwrap_or(false)
    }

    /// Get player score
    pub fn get_player_score(&self) -> i32 {
        self.get_player()
            .and_then(|p| p.get_component::<PlayerController>())
            .map(|c| c.score)
            .unwrap_or(0)
    }

    /// Get player lives
    pub fn get_player_lives(&self) -> i32 {
        self.get_player()
            .and_then(|p| p.get_component::<PlayerController>())
            .map(|c| c.lives)
            .unwrap_or(0)
    }

    /// Get player coins
    pub fn get_player_coins(&self) -> i32 {
        self.get_player()
            .and_then(|p| p.get_component::<PlayerController>())
            .map(|c| c.coins)
            .unwrap_or(0)
    }
}

impl Default for GameWorld {
    fn default() -> Self {
        Self::new()
    }
}