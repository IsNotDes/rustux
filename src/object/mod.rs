//! Game object system for RustUX

use crate::util::Result;
use crate::math::{Vector2, Rect};
use crate::sprite::{Sprite, TextureManager, SpriteRenderer};
use crate::physics::{PhysicsBody, BodyType};
use crate::collision::CollisionLayer;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::collections::HashMap;
use std::any::{Any, TypeId};

/// Unique identifier for game objects
pub type ObjectId = u32;

/// Component trait that all components must implement
pub trait Component: Any + Send + Sync {
    /// Get the component as Any for downcasting
    fn as_any(&self) -> &dyn Any;
    /// Get the component as Any (mutable) for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// Clone the component (for serialization/copying)
    fn clone_component(&self) -> Box<dyn Component>;
}

/// Transform component for position, rotation, and scale
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vector2,
    pub rotation: f32,
    pub scale: Vector2,
}

impl Transform {
    pub fn new(position: Vector2) -> Self {
        Self {
            position,
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0),
        }
    }

    pub fn with_scale(mut self, scale: Vector2) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
}

impl Component for Transform {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn clone_component(&self) -> Box<dyn Component> { Box::new(self.clone()) }
}

/// Sprite component for rendering
#[derive(Debug, Clone)]
pub struct SpriteComponent {
    pub sprite: Sprite,
    pub visible: bool,
    pub layer: i32, // Rendering layer (higher = rendered on top)
}

impl SpriteComponent {
    pub fn new(sprite: Sprite) -> Self {
        Self {
            sprite,
            visible: true,
            layer: 0,
        }
    }

    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }
}

impl Component for SpriteComponent {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn clone_component(&self) -> Box<dyn Component> { Box::new(self.clone()) }
}

/// Physics component wrapper
#[derive(Debug, Clone)]
pub struct PhysicsComponent {
    pub body_id: u32,
    pub body_type: BodyType,
    pub collision_layer: CollisionLayer,
}

impl PhysicsComponent {
    pub fn new(body_id: u32, body_type: BodyType, collision_layer: CollisionLayer) -> Self {
        Self {
            body_id,
            body_type,
            collision_layer,
        }
    }
}

impl Component for PhysicsComponent {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn clone_component(&self) -> Box<dyn Component> { Box::new(self.clone()) }
}

/// Health component for objects that can take damage
#[derive(Debug, Clone)]
pub struct Health {
    pub current: i32,
    pub maximum: i32,
    pub invulnerable: bool,
    pub invulnerability_time: f32,
}

impl Health {
    pub fn new(max_health: i32) -> Self {
        Self {
            current: max_health,
            maximum: max_health,
            invulnerable: false,
            invulnerability_time: 0.0,
        }
    }

    pub fn take_damage(&mut self, damage: i32) -> bool {
        if !self.invulnerable {
            self.current = (self.current - damage).max(0);
            return true;
        }
        false
    }

    pub fn heal(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.maximum);
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }
}

impl Component for Health {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn clone_component(&self) -> Box<dyn Component> { Box::new(self.clone()) }
}

/// Collectible component for items that can be picked up
#[derive(Debug, Clone)]
pub struct Collectible {
    pub value: i32,
    pub collected: bool,
    pub auto_collect: bool, // Automatically collected on touch
}

impl Collectible {
    pub fn new(value: i32) -> Self {
        Self {
            value,
            collected: false,
            auto_collect: true,
        }
    }

    pub fn collect(&mut self) -> i32 {
        if !self.collected {
            self.collected = true;
            self.value
        } else {
            0
        }
    }
}

impl Component for Collectible {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn clone_component(&self) -> Box<dyn Component> { Box::new(self.clone()) }
}

/// Timer component for time-based behaviors
#[derive(Debug, Clone)]
pub struct Timer {
    pub duration: f32,
    pub elapsed: f32,
    pub repeating: bool,
    pub active: bool,
}

impl Timer {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            repeating: false,
            active: true,
        }
    }

    pub fn repeating(mut self) -> Self {
        self.repeating = true;
        self
    }

    pub fn update(&mut self, delta_time: f32) -> bool {
        if !self.active {
            return false;
        }

        self.elapsed += delta_time;
        if self.elapsed >= self.duration {
            if self.repeating {
                self.elapsed = 0.0;
            } else {
                self.active = false;
            }
            return true;
        }
        false
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.active = true;
    }

    pub fn progress(&self) -> f32 {
        if self.duration > 0.0 {
            (self.elapsed / self.duration).min(1.0)
        } else {
            1.0
        }
    }
}

impl Component for Timer {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn clone_component(&self) -> Box<dyn Component> { Box::new(self.clone()) }
}

/// Game object that holds components
pub struct GameObject {
    pub id: ObjectId,
    pub active: bool,
    pub name: String,
    pub tag: String,
    components: HashMap<TypeId, Box<dyn Component>>,
}

impl GameObject {
    pub fn new(id: ObjectId, name: String) -> Self {
        Self {
            id,
            active: true,
            name,
            tag: String::new(),
            components: HashMap::new(),
        }
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.tag = tag;
        self
    }

    /// Add a component to the object
    pub fn add_component<T: Component + 'static>(&mut self, component: T) {
        self.components.insert(TypeId::of::<T>(), Box::new(component));
    }

    /// Get a component from the object
    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        self.components.get(&TypeId::of::<T>()).and_then(|c| c.as_any().downcast_ref::<T>())
    }

    /// Get a component from the object (mutable)
    pub fn get_component_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        self.components.get_mut(&TypeId::of::<T>())
            .and_then(|c| c.as_any_mut().downcast_mut::<T>())
    }

    /// Check if the object has a component
    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }

    /// Remove a component from the object
    pub fn remove_component<T: Component + 'static>(&mut self) -> Option<Box<dyn Component>> {
        self.components.remove(&TypeId::of::<T>())}

    /// Get the object's position from its Transform component
    pub fn position(&self) -> Vector2 {
        self.get_component::<Transform>()
            .map(|t| t.position)
            .unwrap_or(Vector2::ZERO)
    }

    /// Set the object's position in its Transform component
    pub fn set_position(&mut self, position: Vector2) {
        if let Some(transform) = self.get_component_mut::<Transform>() {
            transform.position = position;
        }
    }

    /// Get the object's bounding rectangle
    pub fn get_bounds(&self) -> Option<Rect> {
        if let Some(transform) = self.get_component::<Transform>() {
            if let Some(sprite_comp) = self.get_component::<SpriteComponent>() {
                let size = sprite_comp.sprite.size * transform.scale;
                return Some(Rect::new(
                    transform.position.x,
                    transform.position.y,
                    size.x,
                    size.y,
                ));
            }
        }
        None
    }
}

/// System trait for processing components
pub trait System {
    fn update(&mut self, objects: &mut HashMap<ObjectId, GameObject>, delta_time: f32) -> Result<()>;
}

/// Sprite rendering system
pub struct SpriteRenderSystem;

impl SpriteRenderSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        objects: &HashMap<ObjectId, GameObject>,
        canvas: &mut Canvas<Window>,
        texture_manager: &TextureManager,
    ) -> Result<()> {
        // Collect all visible sprites with their rendering layers
        let mut sprites_to_render: Vec<(i32, &SpriteComponent, &Transform)> = Vec::new();

        for object in objects.values() {
            if !object.active {
                continue;
            }

            if let (Some(sprite_comp), Some(transform)) = (
                object.get_component::<SpriteComponent>(),
                object.get_component::<Transform>(),
            ) {
                if sprite_comp.visible {
                    sprites_to_render.push((sprite_comp.layer, sprite_comp, transform));
                }
            }
        }

        // Sort by layer (lower layers rendered first)
        sprites_to_render.sort_by_key(|(layer, _, _)| *layer);

        // Render sprites
        for (_, sprite_comp, transform) in sprites_to_render {
            let mut sprite = sprite_comp.sprite.clone();
            sprite.position = transform.position;
            sprite.scale = transform.scale;
            sprite.rotation = transform.rotation as f64;

            SpriteRenderer::render_sprite(canvas, texture_manager, &sprite)?;
        }

        Ok(())
    }
}

/// Physics synchronization system
pub struct PhysicsSyncSystem;

impl PhysicsSyncSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn sync_from_physics(
        &self,
        objects: &mut HashMap<ObjectId, GameObject>,
        physics_world: &crate::physics::PhysicsWorld,
    ) -> Result<()> {
        for object in objects.values_mut() {
            if !object.active {
                continue;
            }

            // Get body_id first
            let body_id = if let Some(physics_comp) = object.get_component::<PhysicsComponent>() {
                physics_comp.body_id
            } else {
                continue;
            };

            // Then get transform and update it
            if let Some(transform) = object.get_component_mut::<Transform>() {
                if let Some(physics_body) = physics_world.get_body(body_id) {
                    transform.position = physics_body.position;
                }
            }
        }

        Ok(())
    }

    pub fn sync_to_physics(
        &self,
        objects: &HashMap<ObjectId, GameObject>,
        physics_world: &mut crate::physics::PhysicsWorld,
    ) -> Result<()> {
        for object in objects.values() {
            if !object.active {
                continue;
            }

            if let (Some(physics_comp), Some(transform)) = (
                object.get_component::<PhysicsComponent>(),
                object.get_component::<Transform>(),
            ) {
                physics_world.set_body_position(physics_comp.body_id, transform.position);
            }
        }

        Ok(())
    }
}

/// Timer system for updating all timer components
pub struct TimerSystem;

impl TimerSystem {
    pub fn new() -> Self {
        Self
    }
}

impl System for TimerSystem {
    fn update(&mut self, objects: &mut HashMap<ObjectId, GameObject>, delta_time: f32) -> Result<()> {
        for object in objects.values_mut() {
            if !object.active {
                continue;
            }

            if let Some(timer) = object.get_component_mut::<Timer>() {
                timer.update(delta_time);
            }
        }

        Ok(())
    }
}

/// Health system for managing health and invulnerability
pub struct HealthSystem;

impl HealthSystem {
    pub fn new() -> Self {
        Self
    }
}

impl System for HealthSystem {
    fn update(&mut self, objects: &mut HashMap<ObjectId, GameObject>, delta_time: f32) -> Result<()> {
        for object in objects.values_mut() {
            if !object.active {
                continue;
            }

            if let Some(health) = object.get_component_mut::<Health>() {
                if health.invulnerable && health.invulnerability_time > 0.0 {
                    health.invulnerability_time -= delta_time;
                    if health.invulnerability_time <= 0.0 {
                        health.invulnerable = false;
                    }
                }
            }
        }

        Ok(())
    }
}

/// Game object manager that handles all game objects and systems
pub struct GameObjectManager {
    objects: HashMap<ObjectId, GameObject>,
    next_id: ObjectId,
    systems: Vec<Box<dyn System>>,
    sprite_render_system: SpriteRenderSystem,
    physics_sync_system: PhysicsSyncSystem,
}

impl GameObjectManager {
    pub fn new() -> Self {
        let mut manager = Self {
            objects: HashMap::new(),
            next_id: 1,
            systems: Vec::new(),
            sprite_render_system: SpriteRenderSystem::new(),
            physics_sync_system: PhysicsSyncSystem::new(),
        };

        // Add default systems
        manager.add_system(Box::new(TimerSystem::new()));
        manager.add_system(Box::new(HealthSystem::new()));

        manager
    }

    /// Create a new game object
    pub fn create_object(&mut self, name: String) -> ObjectId {
        let id = self.next_id;
        self.next_id += 1;

        let object = GameObject::new(id, name);
        self.objects.insert(id, object);

        id
    }

    /// Add an existing game object
    pub fn add_object(&mut self, object: GameObject) -> ObjectId {
        let id = object.id;
        self.objects.insert(id, object);
        id
    }

    /// Remove a game object
    pub fn remove_object(&mut self, id: ObjectId) -> Option<GameObject> {
        self.objects.remove(&id)
    }

    /// Get a game object by ID
    pub fn get_object(&self, id: ObjectId) -> Option<&GameObject> {
        self.objects.get(&id)
    }

    /// Get a game object by ID (mutable)
    pub fn get_object_mut(&mut self, id: ObjectId) -> Option<&mut GameObject> {
        self.objects.get_mut(&id)
    }

    /// Find objects by tag
    pub fn find_objects_by_tag(&self, tag: &str) -> Vec<ObjectId> {
        self.objects
            .iter()
            .filter(|(_, obj)| obj.tag == tag)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Find objects by name
    pub fn find_objects_by_name(&self, name: &str) -> Vec<ObjectId> {
        self.objects
            .iter()
            .filter(|(_, obj)| obj.name == name)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Add a system to the manager
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    /// Update all objects and systems
    pub fn update(&mut self, delta_time: f32) -> Result<()> {
        // Update all systems
        for system in &mut self.systems {
            system.update(&mut self.objects, delta_time)?;
        }

        // Update sprite animations
        for object in self.objects.values_mut() {
            if let Some(sprite_comp) = object.get_component_mut::<SpriteComponent>() {
                sprite_comp.sprite.update(delta_time);
            }
        }

        Ok(())
    }

    /// Render all objects
    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        texture_manager: &TextureManager,
    ) -> Result<()> {
        self.sprite_render_system.render(&self.objects, canvas, texture_manager)
    }

    /// Synchronize object positions from physics world
    pub fn sync_from_physics(&mut self, physics_world: &crate::physics::PhysicsWorld) -> Result<()> {
        self.physics_sync_system.sync_from_physics(&mut self.objects, physics_world)
    }

    /// Synchronize object positions to physics world
    pub fn sync_to_physics(&self, physics_world: &mut crate::physics::PhysicsWorld) -> Result<()> {
        self.physics_sync_system.sync_to_physics(&self.objects, physics_world)
    }

    /// Get all object IDs
    pub fn get_object_ids(&self) -> Vec<ObjectId> {
        self.objects.keys().copied().collect()
    }

    /// Get the number of objects
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// Clear all objects
    pub fn clear(&mut self) {
        self.objects.clear();
    }

    /// Get objects in a rectangular area
    pub fn get_objects_in_area(&self, area: &Rect) -> Vec<ObjectId> {
        self.objects
            .iter()
            .filter(|(_, obj)| {
                if let Some(bounds) = obj.get_bounds() {
                    bounds.intersects(area)
                } else {
                    false
                }
            })
            .map(|(&id, _)| id)
            .collect()
    }
}

impl Default for GameObjectManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory functions for creating common game objects
pub mod factory {
    use super::*;
    use crate::sprite::{Sprite, Animation};

    /// Create a simple static sprite object
    pub fn create_static_sprite(
        manager: &mut GameObjectManager,
        name: String,
        position: Vector2,
        texture_name: String,
        size: Vector2,
    ) -> ObjectId {
        let id = manager.create_object(name);
        if let Some(object) = manager.get_object_mut(id) {
            object.add_component(Transform::new(position));
            let sprite = Sprite::with_size(texture_name, Vector2::ZERO, size);
            object.add_component(SpriteComponent::new(sprite));
        }
        
        id
    }

    /// Create a collectible item
    pub fn create_collectible(
        manager: &mut GameObjectManager,
        name: String,
        position: Vector2,
        texture_name: String,
        size: Vector2,
        value: i32,
    ) -> ObjectId {
        let id = create_static_sprite(manager, name, position, texture_name, size);
        
        if let Some(object) = manager.get_object_mut(id) {
            object.add_component(Collectible::new(value));object.tag = "collectible".to_string();
        }
        
        id
    }

    /// Create a physics-enabled object
    pub fn create_physics_object(
        manager: &mut GameObjectManager,
        physics_world: &mut crate::physics::PhysicsWorld,
        name: String,
        position: Vector2,
        size: Vector2,
        body_type: BodyType,
        collision_layer: CollisionLayer,
    ) -> ObjectId {
        let id = manager.create_object(name);
        
        // Create physics body
        let physics_body = crate::physics::PhysicsBody::new(0, position, size, body_type);
        let body_id = physics_world.add_body(physics_body);
        
        if let Some(object) = manager.get_object_mut(id) {
            object.add_component(Transform::new(position));
            object.add_component(PhysicsComponent::new(body_id, body_type, collision_layer));
        }
        
        id
    }
}