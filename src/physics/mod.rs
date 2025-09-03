//! Physics system for RustUX

use crate::math::{Vector2, Rect};
use crate::collision::{CollisionSystem, CollisionLayer, CollisionType, CollisionResult};
use crate::config::{GRAVITY, TERMINAL_VELOCITY, FIXED_TIMESTEP};
use crate::util::Result;
use std::collections::HashMap;

/// Physics body type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyType {
    /// Static body (doesn't move, infinite mass)
    Static,
    /// Kinematic body (moves but not affected by forces)
    Kinematic,
    /// Dynamic body (affected by forces and gravity)
    Dynamic,
}

/// Physics material properties
#[derive(Debug, Clone)]
pub struct PhysicsMaterial {
    /// Friction coefficient (0.0 = no friction, 1.0 = high friction)
    pub friction: f32,
    /// Restitution/bounciness (0.0 = no bounce, 1.0 = perfect bounce)
    pub restitution: f32,
    /// Density (affects mass calculation)
    pub density: f32,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
        }
    }
}

/// Physics body component
#[derive(Debug, Clone)]
pub struct PhysicsBody {
    /// Unique identifier
    pub id: u32,
    /// Body type
    pub body_type: BodyType,
    /// Current position
    pub position: Vector2,
    /// Current velocity
    pub velocity: Vector2,
    /// Acceleration (forces applied this frame)
    pub acceleration: Vector2,
    /// Size of the body
    pub size: Vector2,
    /// Mass of the body
    pub mass: f32,
    /// Physics material
    pub material: PhysicsMaterial,
    /// Collision layer
    pub collision_layer: CollisionLayer,
    /// Whether the body is affected by gravity
    pub use_gravity: bool,
    /// Whether the body is on the ground
    pub on_ground: bool,
    /// Whether the body is active
    pub active: bool,
    /// Maximum velocity
    pub max_velocity: Vector2,
    /// Linear damping (air resistance)
    pub linear_damping: f32,
}

impl PhysicsBody {
    /// Create a new physics body
    pub fn new(id: u32, position: Vector2, size: Vector2, body_type: BodyType) -> Self {
        let mass = match body_type {
            BodyType::Static => f32::INFINITY,
            BodyType::Kinematic => f32::INFINITY,
            BodyType::Dynamic => size.x * size.y * 1.0, // density = 1.0
        };

        Self {
            id,
            body_type,
            position,
            velocity: Vector2::ZERO,
            acceleration: Vector2::ZERO,
            size,
            mass,
            material: PhysicsMaterial::default(),
            collision_layer: CollisionLayer::World,
            use_gravity: matches!(body_type, BodyType::Dynamic),
            on_ground: false,
            active: true,
            max_velocity: Vector2::new(400.0, TERMINAL_VELOCITY),
            linear_damping: 0.98,
        }
    }

    /// Get the collision rectangle for this body
    pub fn get_rect(&self) -> Rect {Rect::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }

    /// Get the center position of the body
    pub fn get_center(&self) -> Vector2 {
        self.position + self.size * 0.5
    }

    /// Apply a force to the body
    pub fn apply_force(&mut self, force: Vector2) {
        if self.body_type == BodyType::Dynamic && self.mass.is_finite() {
            self.acceleration += force / self.mass;
        }
    }

    /// Apply an impulse to the body (instant velocity change)
    pub fn apply_impulse(&mut self, impulse: Vector2) {
        if self.body_type == BodyType::Dynamic && self.mass.is_finite() {
            self.velocity += impulse / self.mass;
        }
    }

    /// Set the velocity directly
    pub fn set_velocity(&mut self, velocity: Vector2) {
        if self.body_type != BodyType::Static {
            self.velocity = velocity;
        }
    }

    /// Move the body to a new position
    pub fn set_position(&mut self, position: Vector2) {
        if self.body_type != BodyType::Static {
            self.position = position;
        }
    }

    /// Check if the body is moving
    pub fn is_moving(&self) -> bool {
        self.velocity.length_squared() > 0.01
    }
}

/// Physics world that manages all physics bodies and simulation
pub struct PhysicsWorld {
    /// All physics bodies
    bodies: HashMap<u32, PhysicsBody>,
    /// Collision system
    collision_system: CollisionSystem,
    /// Next available body ID
    next_id: u32,
    /// Accumulated time for fixed timestep
    accumulator: f32,
    /// Gravity vector
    gravity: Vector2,
    /// Whether physics simulation is paused
    paused: bool,
}

impl PhysicsWorld {
    /// Create a new physics world
    pub fn new() -> Self {
        Self {
            bodies: HashMap::new(),
            collision_system: CollisionSystem::new(),
            next_id: 1,
            accumulator: 0.0,
            gravity: Vector2::new(0.0, GRAVITY),
            paused: false,
        }
    }

    /// Add a physics body to the world
    pub fn add_body(&mut self, mut body: PhysicsBody) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        body.id = id;
        
        // Add collision object
        let collision_type = match body.body_type {
            BodyType::Static => CollisionType::Solid,
            BodyType::Kinematic => CollisionType::Solid,
            BodyType::Dynamic => CollisionType::Solid,
        };
        
        self.collision_system.add_object(body.get_rect(), body.collision_layer, collision_type);
        self.bodies.insert(id, body);
        
        id
    }

    /// Remove a physics body from the world
    pub fn remove_body(&mut self, id: u32) -> Option<PhysicsBody> {
        if let Some(body) = self.bodies.remove(&id) {
            self.collision_system.remove_object(id);
            Some(body)
        } else {
            None
        }
    }

    /// Get a physics body by ID
    pub fn get_body(&self, id: u32) -> Option<&PhysicsBody> {
        self.bodies.get(&id)
    }

    /// Get a physics body by ID (mutable)
    pub fn get_body_mut(&mut self, id: u32) -> Option<&mut PhysicsBody> {
        self.bodies.get_mut(&id)
    }

    /// Update the physics simulation
    pub fn update(&mut self, delta_time: f32) {
        if self.paused {
            return;
        }

        self.accumulator += delta_time;

        // Use fixed timestep for stable physics
        while self.accumulator >= FIXED_TIMESTEP {
            self.step(FIXED_TIMESTEP);
            self.accumulator -= FIXED_TIMESTEP;
        }
    }

    /// Perform one physics simulation step
    fn step(&mut self, dt: f32) {
        // Collect body IDs to avoid borrowing issues
        let body_ids: Vec<u32> = self.bodies.keys().copied().collect();

        for &body_id in &body_ids {
            if let Some(body) = self.bodies.get_mut(&body_id) {
                if !body.active || body.body_type == BodyType::Static {
                    continue;
                }

                // Apply gravity
                if body.use_gravity && body.body_type == BodyType::Dynamic {
                    body.acceleration += self.gravity;
                }

                // Integrate velocity
                if body.body_type == BodyType::Dynamic {
                    body.velocity += body.acceleration * dt;
                    
                    // Apply damping
                    body.velocity *= body.linear_damping;
                
                    // Clamp velocity to maximum
                    body.velocity.x = body.velocity.x.clamp(-body.max_velocity.x, body.max_velocity.x);
                    body.velocity.y = body.velocity.y.clamp(-body.max_velocity.y, body.max_velocity.y);
                }

                // Reset acceleration for next frame
                body.acceleration = Vector2::ZERO;
            }
        }

        // Move bodies and resolve collisions
        for &body_id in &body_ids {
            if let Some(body) = self.bodies.get_mut(&body_id) {
                if !body.active || body.body_type == BodyType::Static {
                    continue;
                }

                // Calculate new position
                let old_position = body.position;
                let new_position = old_position + body.velocity * dt;
                
                // Move and check for collisions
                self.move_body_with_collision(body_id, new_position);}
        }

        // Update collision system with new positions
        for body in self.bodies.values() {
            self.collision_system.update_object(body.id, body.get_rect());
        }
    }

    /// Move a body to a new position, resolving collisions
    fn move_body_with_collision(&mut self, body_id: u32, target_position: Vector2) {
        let body = match self.bodies.get_mut(&body_id) {
            Some(body) => body,
            None => return,
        };

        if body.body_type == BodyType::Static {
            return;
        }

        let old_position = body.position;
        body.position = target_position;
        let new_rect = body.get_rect();
        
        // Check for collisions
        let collisions = self.collision_system.check_collisions(&new_rect, body.collision_layer);
        
        if collisions.is_empty() {
            body.on_ground = false;
            return;
        }

        // Resolve collisions
        let mut resolved_position = target_position;
        let mut hit_ground = false;

        for collision in collisions {
            // Skip if it's the same object
            if collision.object.id == body_id {
                continue;
            }

            match collision.object.collision_type {
                CollisionType::Solid => {
                    // Resolve solid collision
                    let resolved_rect = self.collision_system.resolve_collision(&new_rect, &collision);
                    resolved_position = Vector2::new(resolved_rect.x, resolved_rect.y);
                    
                    // Update velocity based on collision direction
                    match collision.direction {
                        crate::math::Direction::Left | crate::math::Direction::Right => {
                            body.velocity.x = 0.0;
                        }
                        crate::math::Direction::Up => {
                            body.velocity.y = 0.0;
                        }
                        crate::math::Direction::Down => {
                            body.velocity.y = 0.0;
                hit_ground = true;
                        }
                    }
                }
                CollisionType::Platform => {
                    // One-way platform - only collide from above
                    if collision.direction == crate::math::Direction::Down && body.velocity.y >= 0.0 {
                        let resolved_rect = self.collision_system.resolve_collision(&new_rect, &collision);
                        resolved_position.y = resolved_rect.y;
                        body.velocity.y = 0.0;
                        hit_ground = true;
                    }
                }
                CollisionType::Trigger | CollisionType::Sensor => {
                    // Don't resolve position for triggers/sensors
                    // These are handled by the game logic
                }
            }
        }

        body.position = resolved_position;
        body.on_ground = hit_ground;
    }

    /// Apply a force to a body
    pub fn apply_force_to_body(&mut self, body_id: u32, force: Vector2) {
        if let Some(body) = self.bodies.get_mut(&body_id) {
            body.apply_force(force);
        }
    }

    /// Apply an impulse to a body
    pub fn apply_impulse_to_body(&mut self, body_id: u32, impulse: Vector2) {
        if let Some(body) = self.bodies.get_mut(&body_id) {
            body.apply_impulse(impulse);
        }
    }

    /// Set the velocity of a body
    pub fn set_body_velocity(&mut self, body_id: u32, velocity: Vector2) {
        if let Some(body) = self.bodies.get_mut(&body_id) {
            body.set_velocity(velocity);
        }
    }

    /// Set the position of a body
    pub fn set_body_position(&mut self, body_id: u32, position: Vector2) {
        if let Some(body) = self.bodies.get_mut(&body_id) {
            body.set_position(position);
            self.collision_system.update_object(body_id, body.get_rect());
        }
    }

    /// Perform a raycast in the physics world
    pub fn raycast(&self, start: Vector2, direction: Vector2, max_distance: f32, layer: CollisionLayer) -> Option<CollisionResult> {
        self.collision_system.raycast(start, direction, max_distance, layer)
    }

    /// Get all bodies in a rectangular area
    pub fn query_area(&self, rect: &Rect) -> Vec<u32> {
        let mut body_ids = Vec::new();
        
        for body in self.bodies.values() {
            if body.active && body.get_rect().intersects(rect) {
                body_ids.push(body.id);
            }
        }
        
        body_ids
    }

    /// Set gravity
    pub fn set_gravity(&mut self, gravity: Vector2) {
        self.gravity = gravity;
    }

    /// Get gravity
    pub fn get_gravity(&self) -> Vector2 {
        self.gravity
    }

    /// Pause/unpause physics simulation
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// Check if physics simulation is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Clear all bodies from the world
    pub fn clear(&mut self) {
        self.bodies.clear();
        self.collision_system.clear();
    }

    /// Get the number of bodies in the world
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Get all body IDs
    pub fn get_body_ids(&self) -> Vec<u32> {
        self.bodies.keys().copied().collect()
    }

    /// Get the collision system
    pub fn collision_system(&self) -> &CollisionSystem {
        &self.collision_system
    }

    /// Get the collision system (mutable)
    pub fn collision_system_mut(&mut self) -> &mut CollisionSystem {
        &mut self.collision_system
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// Physics utilities
pub mod utils {
    use super::*;

    /// Calculate the mass of a body based on its size and density
    pub fn calculate_mass(size: Vector2, density: f32) -> f32 {
        size.x * size.y * density
    }

    /// Calculate the impulse needed to reach a target velocity
    pub fn impulse_for_velocity(current_velocity: Vector2, target_velocity: Vector2, mass: f32) -> Vector2 {
        (target_velocity - current_velocity) * mass
    }

    /// Calculate the force needed to reach a target acceleration
    pub fn force_for_acceleration(acceleration: Vector2, mass: f32) -> Vector2 {
        acceleration * mass
    }

    /// Calculate jump velocity needed to reach a target height
    pub fn jump_velocity_for_height(height: f32, gravity: f32) -> f32 {
        (2.0 * gravity * height).sqrt()
    }

    /// Calculate the time it takes to fall a certain distance
    pub fn fall_time(distance: f32, gravity: f32) -> f32 {
        (2.0 * distance / gravity).sqrt()
    }

    /// Calculate the distance traveled during a jump
    pub fn jump_distance(initial_velocity: Vector2, gravity: f32) -> f32 {
        let time_to_peak = initial_velocity.y / gravity;
        let total_time = time_to_peak * 2.0;
        initial_velocity.x * total_time
    }
}

/// Physics constants for common platformer mechanics
pub mod constants {
    /// Standard gravity for platformers (pixels/second²)
    pub const PLATFORMER_GRAVITY: f32 = 980.0;
    
    /// Typical jump velocity for a character
    pub const JUMP_VELOCITY: f32 = -400.0;
    
    /// Standard walking speed
    pub const WALK_SPEED: f32 = 150.0;
    
    /// Standard running speed
    pub const RUN_SPEED: f32 = 250.0;
    
    /// Air control factor (how much control you have while in air)
    pub const AIR_CONTROL: f32 = 0.8;
    
    /// Ground friction
    pub const GROUND_FRICTION: f32 = 0.9;
    
    /// Air resistance
    pub const AIR_RESISTANCE: f32 = 0.99;
}