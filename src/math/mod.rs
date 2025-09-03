//! Mathematical utilities for RustUX

use serde::{Deserialize, Serialize};

/// 2D Vector using f32 components
pub type Vector2 = glam::Vec2;

/// 3D Vector using f32 components
pub type Vector3 = glam::Vec3;

/// 2D Rectangle
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Create a rectangle from position and size vectors
    pub fn from_pos_size(pos: Vector2, size: Vector2) -> Self {
        Self::new(pos.x, pos.y, size.x, size.y)
    }

    /// Get the left edge
    pub fn left(&self) -> f32 {
        self.x
    }

    /// Get the right edge
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Get the top edge
    pub fn top(&self) -> f32 {
        self.y
    }

    /// Get the bottom edge
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Get the center point
    pub fn center(&self) -> Vector2 {
        Vector2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Get the top-left corner
    pub fn top_left(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    /// Get the top-right corner
    pub fn top_right(&self) -> Vector2 {
        Vector2::new(self.right(), self.y)
    }

    /// Get the bottom-left corner
    pub fn bottom_left(&self) -> Vector2 {
        Vector2::new(self.x, self.bottom())
    }

    /// Get the bottom-right corner
    pub fn bottom_right(&self) -> Vector2 {
        Vector2::new(self.right(), self.bottom())
    }

    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    /// Check if this rectangle contains a point
    pub fn contains_point(&self, point: Vector2) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }

    /// Check if this rectangle completely contains another rectangle
    pub fn contains_rect(&self, other: &Rect) -> bool {
        self.left() <= other.left()
            && self.right() >= other.right()
            && self.top() <= other.top()
            && self.bottom() >= other.bottom()
    }

    /// Get the intersection of two rectangles
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }

        let left = self.left().max(other.left());
        let right = self.right().min(other.right());
        let top = self.top().max(other.top());
        let bottom = self.bottom().min(other.bottom());

        Some(Rect::new(left, top, right - left, bottom - top))
    }

    /// Expand the rectangle by the given amount in all directions
    pub fn expand(&self, amount: f32) -> Rect {
        Rect::new(
            self.x - amount,
            self.y - amount,
            self.width + 2.0 * amount,
            self.height + 2.0 * amount,
        )
    }

    /// Move the rectangle by the given offset
    pub fn translate(&self, offset: Vector2) -> Rect {
        Rect::new(self.x + offset.x, self.y + offset.y, self.width, self.height)
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

/// Direction enumeration for movement and collision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Get the opposite direction
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    /// Get the direction as a unit vector
    pub fn to_vector(&self) -> Vector2 {
        match self {
            Direction::Up => Vector2::new(0.0, -1.0),
            Direction::Down => Vector2::new(0.0, 1.0),
            Direction::Left => Vector2::new(-1.0, 0.0),
            Direction::Right => Vector2::new(1.0, 0.0),
        }
    }

    /// Check if this is a horizontal direction
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Direction::Left | Direction::Right)
    }

    /// Check if this is a vertical direction
    pub fn is_vertical(&self) -> bool {
        matches!(self, Direction::Up | Direction::Down)
    }
}

/// Mathematical constants and utility functions
pub mod constants {
    pub const PI: f32 = std::f32::consts::PI;
    pub const TAU: f32 = std::f32::consts::TAU;
    pub const EPSILON: f32 = f32::EPSILON;
}

/// Utility functions for common mathematical operations
pub mod utils {
    use super::Vector2;

    /// Convert degrees to radians
    pub fn deg_to_rad(degrees: f32) -> f32 {
        degrees * super::constants::PI / 180.0
    }

    /// Convert radians to degrees
    pub fn rad_to_deg(radians: f32) -> f32 {
        radians * 180.0 / super::constants::PI
    }

    /// Clamp a value between min and max
    pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }

    /// Linear interpolation between two values
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    /// Linear interpolation between two vectors
    pub fn lerp_vec2(a: Vector2, b: Vector2, t: f32) -> Vector2 {
        a + (b - a) * t
    }

    /// Check if two floating point numbers are approximately equal
    pub fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
        (a - b).abs() < epsilon
    }

    /// Smoothstep interpolation
    pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// Calculate distance between two points
    pub fn distance(a: Vector2, b: Vector2) -> f32 {
        (b - a).length()
    }

    /// Calculate squared distance between two points (faster than distance)
    pub fn distance_squared(a: Vector2, b: Vector2) -> f32 {
        (b - a).length_squared()
    }
}