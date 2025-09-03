//! Collision detection system for RustUX

use crate::math::{Rect, Vector2, Direction};
use crate::util::Result;
use std::collections::HashMap;

/// Collision layer for organizing collision objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollisionLayer {
    /// Static world geometry (tiles, platforms)
    World,
    /// Player character
    Player,
    /// Enemies and badguys
    Enemy,
    /// Collectible items
    Item,
    /// Triggers and sensors
    Trigger,
    /// Projectiles
    Projectile,
}

/// Collision object type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionType {
    /// Solid collision (blocks movement)
    Solid,
    /// One-way platform (can jump through from below)
    Platform,
    /// Trigger area (doesn't block movement)
    Trigger,
    /// Sensor (detects but doesn't block)
    Sensor,
}

/// Collision object
#[derive(Debug, Clone)]
pub struct CollisionObject {
    /// Unique identifier
    pub id: u32,
    /// Collision rectangle
    pub rect: Rect,
    /// Collision layer
    pub layer: CollisionLayer,
    /// Collision type
    pub collision_type: CollisionType,
    /// Whether the object is active
    pub active: bool,
    /// Custom data for the object
    pub data: HashMap<String, String>,
}

impl CollisionObject {
    /// Create a new collision object
    pub fn new(id: u32, rect: Rect, layer: CollisionLayer, collision_type: CollisionType) -> Self {
        Self {
            id,
            rect,
            layer,
            collision_type,
            active: true,
            data: HashMap::new(),
        }
    }

    /// Check if this object intersects with another rectangle
    pub fn intersects(&self, other_rect: &Rect) -> bool {
        self.active && self.rect.intersects(other_rect)
    }

    /// Get the intersection rectangle with another rectangle
    pub fn intersection(&self, other_rect: &Rect) -> Option<Rect> {
        if self.intersects(other_rect) {
            self.rect.intersection(other_rect)
        } else {
            None
        }
    }
}

/// Collision result information
#[derive(Debug, Clone)]
pub struct CollisionResult {
    /// The collision object that was hit
    pub object: CollisionObject,
    /// Direction of the collision
    pub direction: Direction,
    /// Penetration depth
    pub penetration: f32,
    /// Contact point
    pub contact_point: Vector2,
}

/// Spatial hash grid for efficient collision detection
pub struct SpatialGrid {
    /// Grid cell size
    cell_size: f32,
    /// Grid cells containing object IDs
    cells: HashMap<(i32, i32), Vec<u32>>,
    /// All collision objects
    objects: HashMap<u32, CollisionObject>,
}

impl SpatialGrid {
    /// Create a new spatial grid
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            objects: HashMap::new(),
        }
    }

    /// Add an object to the grid
    pub fn add_object(&mut self, object: CollisionObject) {
        let id = object.id;
        let cells = self.get_cells_for_rect(&object.rect);
        
        // Add to cells
        for cell in cells {
            self.cells.entry(cell).or_insert_with(Vec::new).push(id);
        }
        
        // Store object
        self.objects.insert(id, object);
    }

    /// Remove an object from the grid
    pub fn remove_object(&mut self, id: u32) -> Option<CollisionObject> {
        if let Some(object) = self.objects.remove(&id) {
            let cells = self.get_cells_for_rect(&object.rect);
            // Remove from cells
            for cell in cells {
                if let Some(cell_objects) = self.cells.get_mut(&cell) {
                    cell_objects.retain(|&obj_id| obj_id != id);
                    if cell_objects.is_empty() {
                        self.cells.remove(&cell);
                    }
                }
            }
            
            Some(object)
        } else {
            None
        }
    }

    /// Update an object's position in the grid
    pub fn update_object(&mut self, id: u32, new_rect: Rect) {
        if let Some(object) = self.remove_object(id) {
            let mut updated_object = object;
            updated_object.rect = new_rect;
            self.add_object(updated_object);
        }
    }

    /// Get all objects that could potentially collide with the given rectangle
    pub fn query_rect(&self, rect: &Rect) -> Vec<&CollisionObject> {
        let cells = self.get_cells_for_rect(rect);
        let mut candidates = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for cell in cells {
            if let Some(cell_objects) = self.cells.get(&cell) {
                for &obj_id in cell_objects {
                    if seen.insert(obj_id) {
                        if let Some(object) = self.objects.get(&obj_id) {
                            if object.intersects(rect) {
                                candidates.push(object);
                            }
                        }
                    }
                }
            }
        }

        candidates
    }

    /// Get grid cells that a rectangle overlaps
    fn get_cells_for_rect(&self, rect: &Rect) -> Vec<(i32, i32)> {
        let min_x = (rect.left() / self.cell_size).floor() as i32;
        let max_x = (rect.right() / self.cell_size).floor() as i32;
        let min_y = (rect.top() / self.cell_size).floor() as i32;
        let max_y = (rect.bottom() / self.cell_size).floor() as i32;

        let mut cells = Vec::new();
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                cells.push((x, y));
            }
        }
        cells
    }

    /// Clear all objects from the grid
    pub fn clear(&mut self) {
        self.cells.clear();
        self.objects.clear();
    }

    /// Get the number of objects in the grid
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }
}

/// Main collision detection system
pub struct CollisionSystem {
    /// Spatial grid for efficient collision detection
    spatial_grid: SpatialGrid,
    /// Next available object ID
    next_id: u32,
    /// Collision layer interaction matrix
    layer_matrix: HashMap<(CollisionLayer, CollisionLayer), bool>,
}

impl CollisionSystem {
    /// Create a new collision system
    pub fn new() -> Self {
        let mut system = Self {
            spatial_grid: SpatialGrid::new(64.0), // 64x64 pixel cells
            next_id: 1,
            layer_matrix: HashMap::new(),
        };
        
        // Set up default layer interactions
        system.setup_default_layer_interactions();
        system
    }

    /// Set up default collision layer interactions
    fn setup_default_layer_interactions(&mut self) {
        use CollisionLayer::*;
        
        // World collides with everything except triggers
        self.set_layer_interaction(World, Player, true);
        self.set_layer_interaction(World, Enemy, true);
        self.set_layer_interaction(World, Item, false);
        self.set_layer_interaction(World, Trigger, false);
        self.set_layer_interaction(World, Projectile, true);
        
        // Player interactions
        self.set_layer_interaction(Player, Enemy, true);
        self.set_layer_interaction(Player, Item, true);
        self.set_layer_interaction(Player, Trigger, true);
        self.set_layer_interaction(Player, Projectile, true);
        
        // Enemy interactions
        self.set_layer_interaction(Enemy, Item, false);
        self.set_layer_interaction(Enemy, Trigger, true);
        self.set_layer_interaction(Enemy, Projectile, true);
        
        // Items don't collide with each other
        self.set_layer_interaction(Item, Item, false);
        self.set_layer_interaction(Item, Trigger, false);
        self.set_layer_interaction(Item, Projectile, false);
        
        // Triggers detect everything
        self.set_layer_interaction(Trigger, Trigger, false);
        self.set_layer_interaction(Trigger, Projectile, true);
        
        // Projectiles collide with most things
        self.set_layer_interaction(Projectile, Projectile, false);
    }

    /// Set whether two layers should interact
    pub fn set_layer_interaction(&mut self, layer1: CollisionLayer, layer2: CollisionLayer, interact: bool) {
        self.layer_matrix.insert((layer1, layer2), interact);
        self.layer_matrix.insert((layer2, layer1), interact);
    }

    /// Check if two layers should interact
    pub fn layers_interact(&self, layer1: CollisionLayer, layer2: CollisionLayer) -> bool {
        self.layer_matrix.get(&(layer1, layer2)).copied().unwrap_or(false)
    }

    /// Add a collision object
    pub fn add_object(&mut self, rect: Rect, layer: CollisionLayer, collision_type: CollisionType) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let object = CollisionObject::new(id, rect, layer, collision_type);
        self.spatial_grid.add_object(object);
        
        id
    }

    /// Remove a collision object
    pub fn remove_object(&mut self, id: u32) -> bool {
        self.spatial_grid.remove_object(id).is_some()
    }

    /// Update an object's position
    pub fn update_object(&mut self, id: u32, new_rect: Rect) {
        self.spatial_grid.update_object(id, new_rect);
    }

    /// Check collision between two rectangles
    pub fn check_collision(&self, rect1: &Rect, rect2: &Rect) -> bool {
        rect1.intersects(rect2)
    }

    /// Get collision results for a moving rectangle
    pub fn check_collisions(&self, rect: &Rect, layer: CollisionLayer) -> Vec<CollisionResult> {
        let mut results = Vec::new();
        let candidates = self.spatial_grid.query_rect(rect);

        for object in candidates {
            if !self.layers_interact(layer, object.layer) {
                continue;
            }

            if let Some(intersection) = object.intersection(rect) {
                let direction = self.get_collision_direction(rect, &object.rect);
                let penetration = match direction {
                    Direction::Left | Direction::Right => intersection.width,
                    Direction::Up | Direction::Down => intersection.height,
                };

                results.push(CollisionResult {
                    object: object.clone(),
                    direction,
                    penetration,
                    contact_point: intersection.center(),
                });
            }
        }

        results
    }

    /// Resolve collision by moving the rectangle out of collision
    pub fn resolve_collision(&self, rect: &Rect, collision: &CollisionResult) -> Rect {
        let mut resolved_rect = *rect;
        
        match collision.direction {
            Direction::Left => {
                resolved_rect.x = collision.object.rect.right();
            }
            Direction::Right => {
                resolved_rect.x = collision.object.rect.left() - resolved_rect.width;
            }
            Direction::Up => {
                resolved_rect.y = collision.object.rect.bottom();
            }
            Direction::Down => {
                resolved_rect.y = collision.object.rect.top() - resolved_rect.height;
            }
        }
        
        resolved_rect
    }

    /// Get the primary collision direction
    fn get_collision_direction(&self, moving_rect: &Rect, static_rect: &Rect) -> Direction {
        let center1 = moving_rect.center();
        let center2 = static_rect.center();
        let diff = center1 - center2;

        if diff.x.abs() > diff.y.abs() {
            if diff.x > 0.0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else {
            if diff.y > 0.0 {
                Direction::Down
            } else {
                Direction::Up
            }
        }
    }

    /// Perform a raycast and return the first collision
    pub fn raycast(&self, start: Vector2, direction: Vector2, max_distance: f32, layer: CollisionLayer) -> Option<CollisionResult> {
        let end = start + direction.normalize() * max_distance;
        let ray_rect = Rect::new(
            start.x.min(end.x),
            start.y.min(end.y),
            (end.x - start.x).abs(),
            (end.y - start.y).abs(),
        );

        let candidates = self.spatial_grid.query_rect(&ray_rect);
        let mut closest_collision = None;
        let mut closest_distance = max_distance;

        for object in candidates {
            if !self.layers_interact(layer, object.layer) {
                continue;
            }

            // Simple ray-rectangle intersection
            if let Some(intersection_point) = self.ray_rect_intersection(start, direction, &object.rect) {
                let distance = (intersection_point - start).length();
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_collision = Some(CollisionResult {
                        object: object.clone(),
                        direction: self.get_collision_direction(&Rect::new(start.x, start.y, 1.0, 1.0), &object.rect),
                        penetration: 0.0,
                        contact_point: intersection_point,
                    });
                }
            }
        }

        closest_collision
    }

    /// Ray-rectangle intersection
    fn ray_rect_intersection(&self, ray_start: Vector2, ray_dir: Vector2, rect: &Rect) -> Option<Vector2> {
        let inv_dir = Vector2::new(1.0 / ray_dir.x, 1.0 / ray_dir.y);
        
        let t1 = (rect.left() - ray_start.x) * inv_dir.x;
        let t2 = (rect.right() - ray_start.x) * inv_dir.x;
        let t3 = (rect.top() - ray_start.y) * inv_dir.y;
        let t4 = (rect.bottom() - ray_start.y) * inv_dir.y;
        
        let tmin = t1.min(t2).max(t3.min(t4));
        let tmax = t1.max(t2).min(t3.max(t4));
        
        if tmax < 0.0 || tmin > tmax {
            None
        } else {
            let t = if tmin < 0.0 { tmax } else { tmin };
            Some(ray_start + ray_dir * t)
        }
    }

    /// Clear all collision objects
    pub fn clear(&mut self) {
        self.spatial_grid.clear();
    }

    /// Get the number of collision objects
    pub fn object_count(&self) -> usize {
        self.spatial_grid.object_count()
    }
}

impl Default for CollisionSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Collision detection utilities
pub mod utils {
    use super::*;

    /// Check if a point is inside a rectangle
    pub fn point_in_rect(point: Vector2, rect: &Rect) -> bool {
        rect.contains_point(point)
    }

    /// Get the closest point on a rectangle to a given point
    pub fn closest_point_on_rect(point: Vector2, rect: &Rect) -> Vector2 {
        Vector2::new(
            point.x.clamp(rect.left(), rect.right()),
            point.y.clamp(rect.top(), rect.bottom()),
        )
    }

    /// Calculate the minimum translation vector to separate two rectangles
    pub fn minimum_translation_vector(rect1: &Rect, rect2: &Rect) -> Option<Vector2> {
        if !rect1.intersects(rect2) {
            return None;
        }

        let overlap_x = (rect1.right() - rect2.left()).min(rect2.right() - rect1.left());
        let overlap_y = (rect1.bottom() - rect2.top()).min(rect2.bottom() - rect1.top());

        if overlap_x < overlap_y {
            let direction = if rect1.center().x < rect2.center().x { -1.0 } else { 1.0 };
            Some(Vector2::new(overlap_x * direction, 0.0))
        } else {
            let direction = if rect1.center().y < rect2.center().y { -1.0 } else { 1.0 };
            Some(Vector2::new(0.0, overlap_y * direction))
        }
    }
}