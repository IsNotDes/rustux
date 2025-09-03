//! Enemy/badguy system for RustUX

use crate::object::{GameObjectManager, Component, Transform, SpriteComponent, Health, ObjectId};
use crate::physics::{PhysicsWorld, BodyType};
use crate::collision::CollisionLayer;
use crate::sprite::Sprite;
use crate::math::Vector2;
use crate::util::Result;
use std::any::Any;

/// Badguy AI state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadguyState {
    Idle,
    Walking,
    Chasing,
    Attacking,
    Stunned,
    Dead,
}

/// Badguy AI component
#[derive(Debug, Clone)]
pub struct BadguyAI {
    pub state: BadguyState,
    pub move_speed: f32,
    pub detection_range: f32,
    pub attack_range: f32,
    pub direction: f32, // -1.0 for left, 1.0 for right
    pub state_timer: f32,
    pub patrol_distance: f32,
    pub start_position: Vector2,
}

impl BadguyAI {
    pub fn new(move_speed: f32) -> Self {
        Self {
            state: BadguyState::Walking,
            move_speed,
            detection_range: 150.0,
            attack_range: 32.0,
            direction: 1.0,
            state_timer: 0.0,
            patrol_distance: 100.0,
            start_position: Vector2::ZERO,
        }
    }

    pub fn with_patrol_distance(mut self, distance: f32) -> Self {
        self.patrol_distance = distance;
        self
    }

    pub fn with_detection_range(mut self, range: f32) -> Self {
        self.detection_range = range;
        self
    }
}

impl Component for BadguyAI {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn clone_component(&self) -> Box<dyn Component> { Box::new(self.clone()) }
}

/// Badguy type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadguyType {
    Goomba,    // Simple walking enemy
    Spiky,     // Spiky enemy that hurts to touch
    Jumpy,     // Jumping enemy
    Flying,    // Flying enemy
}

/// Badguy component that defines the type and behavior
#[derive(Debug, Clone)]
pub struct Badguy {
    pub badguy_type: BadguyType,
    pub damage: i32,
    pub points: i32, // Points awarded when defeated
    pub can_be_stomped: bool,
    pub can_be_kicked: bool,
}

impl Badguy {
    pub fn new(badguy_type: BadguyType) -> Self {
        let (damage, points, can_be_stomped, can_be_kicked) = match badguy_type {
            BadguyType::Goomba => (1, 100, true, false),
            BadguyType::Spiky => (1, 200, false, true),
            BadguyType::Jumpy => (1, 150, true, false),
            BadguyType::Flying => (1, 250, false, false),
        };

        Self {
            badguy_type,
            damage,
            points,
            can_be_stomped,
            can_be_kicked,
        }
    }
}

impl Component for Badguy {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn clone_component(&self) -> Box<dyn Component> { Box::new(self.clone()) }
}

/// Badguy system for updating AI and behavior
pub struct BadguySystem {
    player_id: Option<ObjectId>,
}

impl BadguySystem {
    pub fn new() -> Self {
        Self {
            player_id: None,
        }
    }

    pub fn set_player_id(&mut self, player_id: ObjectId) {
        self.player_id = Some(player_id);
    }

    pub fn update(
        &mut self,
        object_manager: &mut GameObjectManager,
        physics_world: &mut PhysicsWorld,
        delta_time: f32,
    ) -> Result<()> {
        // Get player position for AI calculations
        let player_position = if let Some(player_id) = self.player_id {
            object_manager.get_object(player_id)
                .and_then(|obj| obj.get_component::<Transform>())
                .map(|t| t.position)
        } else {
            None
        };

        // Collect badguy object IDs to avoid borrowing issues
        let badguy_ids: Vec<ObjectId> = object_manager
            .get_object_ids()
            .into_iter()
            .filter(|&id| {
                object_manager.get_object(id)
                    .map(|obj| obj.has_component::<Badguy>())
                    .unwrap_or(false)
            })
            .collect();

        // Update each badguy
        for badguy_id in badguy_ids {
            self.update_badguy(badguy_id, object_manager, physics_world, player_position, delta_time)?;
        }

        Ok(())
    }

    fn update_badguy(
        &self,
        badguy_id: ObjectId,
        object_manager: &mut GameObjectManager,
        physics_world: &mut PhysicsWorld,
        player_position: Option<Vector2>,
        delta_time: f32,
    ) -> Result<()> {
        // Get transform and body_id first
        let (transform, body_id) = {
            let object = match object_manager.get_object(badguy_id) {
                Some(obj) => obj,
                None => return Ok(()),
            };

            if !object.active {
                return Ok(());
            }

            let transform = match object.get_component::<Transform>() {
                Some(t) => t.position,
                None => return Ok(()),
            };

            let body_id = match object.get_component::<crate::object::PhysicsComponent>() {
                Some(comp) => comp.body_id,
                None => return Ok(()),
            };

            (transform, body_id)
        };

        // Now get mutable access to update AI
        let object = match object_manager.get_object_mut(badguy_id) {
            Some(obj) => obj,
            None => return Ok(()),
        };

        let ai_component = match object.get_component_mut::<BadguyAI>() {
            Some(ai) => ai,
            None => return Ok(()),
        };

        // Initialize start position if not set
        if ai_component.start_position == Vector2::ZERO {
            ai_component.start_position = transform;
        }

        // Update state timer
        ai_component.state_timer += delta_time;

        // Calculate distance to player
        let distance_to_player = if let Some(player_pos) = player_position {
            (transform - player_pos).length()
        } else {
            f32::INFINITY
        };

        // Update AI state
        match ai_component.state {
            BadguyState::Idle => {
                if ai_component.state_timer >1.0 {
                    ai_component.state = BadguyState::Walking;
                    ai_component.state_timer = 0.0;
                }
            }
            BadguyState::Walking => {
                // Check if we should chase the player
                if distance_to_player < ai_component.detection_range {
                    ai_component.state = BadguyState::Chasing;
                    ai_component.state_timer = 0.0;
                } else {
                    // Patrol behavior
                    let distance_from_start = (transform - ai_component.start_position).length();
                    if distance_from_start > ai_component.patrol_distance {
                        ai_component.direction *= -1.0; // Turn around
                    }
                }
            }
            BadguyState::Chasing => {
                // Chase the player
                if let Some(player_pos) = player_position {
                    if distance_to_player > ai_component.detection_range * 1.5 {
                        // Lost the player, go back to walking
                        ai_component.state = BadguyState::Walking;
                        ai_component.state_timer = 0.0;
                    } else if distance_to_player < ai_component.attack_range {
                        // Close enough to attack
                        ai_component.state = BadguyState::Attacking;
                        ai_component.state_timer = 0.0;
                    } else {
                        // Move towards player
                        ai_component.direction = if player_pos.x > transform.x { 1.0 } else { -1.0 };
                }
                }
            }
            BadguyState::Attacking => {
                if ai_component.state_timer > 0.5 {
                    ai_component.state = BadguyState::Chasing;
                    ai_component.state_timer = 0.0;
                }
            }
            BadguyState::Stunned => {
                if ai_component.state_timer > 2.0 {
                    ai_component.state = BadguyState::Walking;
                    ai_component.state_timer = 0.0;
                }
            }
            BadguyState::Dead => {
                // Dead badguys don't move
                return Ok(());
            }
        }

        // Apply movement based on state
        match ai_component.state {
            BadguyState::Walking | BadguyState::Chasing => {
                let velocity = Vector2::new(
                    ai_component.direction * ai_component.move_speed,
                    0.0, // Don't override gravity
                );
                
                // Only set horizontal velocity, preserve vertical
                if let Some(body) = physics_world.get_body(body_id) {
                    let current_velocity = body.velocity;
                    physics_world.set_body_velocity(
                        body_id,
                        Vector2::new(velocity.x, current_velocity.y)
                    );
                }
            }
            BadguyState::Stunned | BadguyState::Dead => {
                // Stop movement
                if let Some(body) = physics_world.get_body(body_id) {
                    let current_velocity = body.velocity;
                    physics_world.set_body_velocity(
                        body_id,
                        Vector2::new(0.0, current_velocity.y)
                    );
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle collision between badguy and player
    pub fn handle_player_collision(
        &self,
        badguy_id: ObjectId,
        player_id: ObjectId,
        object_manager: &mut GameObjectManager,
    ) -> Result<()> {
        // Get badguy data first
        let (can_be_stomped, damage, badguy_pos) = {
            let badguy_obj = match object_manager.get_object(badguy_id) {
                Some(obj) => obj,
                None => return Ok(()),
            };

            let can_be_stomped = badguy_obj.get_component::<Badguy>()
                .map(|b| b.can_be_stomped)
                .unwrap_or(false);

            let damage = badguy_obj.get_component::<Badguy>()
                .map(|b| b.damage)
                .unwrap_or(1);

            let badguy_pos = badguy_obj.position();
            (can_be_stomped, damage, badguy_pos)
        };
        
        // Get player position
        let player_pos = object_manager.get_object(player_id)
            .map(|obj| obj.position())
            .unwrap_or(Vector2::ZERO);

        let is_stomping = player_pos.y < badguy_pos.y - 16.0; // Player is above badguy

        if is_stomping && can_be_stomped {
            // Stomp the badguy
            if let Some(badguy_obj) = object_manager.get_object_mut(badguy_id) {
                if let Some(ai) = badguy_obj.get_component_mut::<BadguyAI>() {
                    ai.state = BadguyState::Stunned;
                    ai.state_timer = 0.0;
                }
            }
            
            // Award points to player (this would be handled by a score system)
            log::info!("Badguy stomped!");
        } else {
            // Damage the player
            if let Some(player_obj) = object_manager.get_object_mut(player_id) {
                if let Some(health) = player_obj.get_component_mut::<Health>() {
                    health.take_damage(damage);
                    log::info!("Player took {} damage from badguy", damage);
                }
            }
        }

        Ok(())
    }
}

/// Factory functions for creating different types of badguys
pub mod factory {
    use super::*;
    use crate::object::factory;

    /// Create a Goomba enemy
    pub fn create_goomba(
        object_manager: &mut GameObjectManager,
        physics_world: &mut PhysicsWorld,
        position: Vector2,
        texture_name: String,
    ) -> ObjectId {
        let id = factory::create_physics_object(
            object_manager,
            physics_world,
            "Goomba".to_string(),
            position,
            Vector2::new(32.0, 32.0),
            BodyType::Dynamic,
            CollisionLayer::Enemy,
        );

        if let Some(object) = object_manager.get_object_mut(id) {
            // Add sprite
            let sprite = Sprite::with_size(texture_name, Vector2::ZERO, Vector2::new(32.0, 32.0));
            object.add_component(SpriteComponent::new(sprite));
            
            // Add badguy components
            object.add_component(Badguy::new(BadguyType::Goomba));
            object.add_component(BadguyAI::new(50.0)); // Slow movement
            object.add_component(Health::new(1));
            
            object.tag = "badguy".to_string();
        }

        id
    }

    /// Create a Spiky enemy
    pub fn create_spiky(
        object_manager: &mut GameObjectManager,
        physics_world: &mut PhysicsWorld,
        position: Vector2,
        texture_name: String,
    ) -> ObjectId {
        let id = factory::create_physics_object(
            object_manager,
            physics_world,
            "Spiky".to_string(),
            position,
            Vector2::new(32.0, 32.0),
            BodyType::Dynamic,
            CollisionLayer::Enemy,
        );

        if let Some(object) = object_manager.get_object_mut(id) {
            // Add sprite
            let sprite = Sprite::with_size(texture_name, Vector2::ZERO, Vector2::new(32.0, 32.0));
            object.add_component(SpriteComponent::new(sprite));
            
            // Add badguy components
            object.add_component(Badguy::new(BadguyType::Spiky));
            object.add_component(BadguyAI::new(75.0)); // Medium movement
            object.add_component(Health::new(2));
            
            object.tag = "badguy".to_string();
        }

        id
    }

    /// Create a Flying enemy
    pub fn create_flying(
        object_manager: &mut GameObjectManager,
        physics_world: &mut PhysicsWorld,
        position: Vector2,
        texture_name: String,
    ) -> ObjectId {
        let id = factory::create_physics_object(
            object_manager,
            physics_world,
            "Flying".to_string(),
            position,
            Vector2::new(32.0, 32.0),
            BodyType::Kinematic, // Flying enemies don't use gravity
            CollisionLayer::Enemy,
        );

        if let Some(object) = object_manager.get_object_mut(id) {
            // Add sprite
            let sprite = Sprite::with_size(texture_name, Vector2::ZERO, Vector2::new(32.0, 32.0));
            object.add_component(SpriteComponent::new(sprite));
            
            // Add badguy components
            object.add_component(Badguy::new(BadguyType::Flying));
            let mut ai = BadguyAI::new(100.0); // Fast movement
            ai.patrol_distance = 200.0; // Larger patrol area
            object.add_component(ai);
            object.add_component(Health::new(1));
            
            object.tag = "badguy".to_string();
        }

        id
    }
}