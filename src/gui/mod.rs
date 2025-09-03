//! GUI system for RustUX

use crate::util::Result;
use crate::math::{Vector2, Rect};
use crate::sprite::{Sprite, TextureManager, SpriteRenderer};
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;

pub mod button;
pub mod menu;
pub mod text;

pub use button::Button;
pub use menu::{Menu, MenuItem};
pub use text::Text;

/// GUI element trait
pub trait GuiElement {
    /// Update the element
    fn update(&mut self, delta_time: f32) -> Result<()>;
    
    /// Render the element
    fn render(&self, canvas: &mut Canvas<Window>, texture_manager: &TextureManager) -> Result<()>;
    
    /// Handle mouse input
    fn handle_mouse(&mut self, x: i32, y: i32, pressed: bool) -> Result<bool>;
    
    /// Handle keyboard input
    fn handle_key(&mut self, keycode: Keycode, pressed: bool) -> Result<bool>;
    
    /// Get the element's bounding rectangle
    fn bounds(&self) -> Rect;
    /// Check if the element is visible
    fn is_visible(&self) -> bool;
    
    /// Set visibility
    fn set_visible(&mut self, visible: bool);
    
    /// Check if the element is enabled
    fn is_enabled(&self) -> bool;
    
    /// Set enabled state
    fn set_enabled(&mut self, enabled: bool);
}

/// GUI event types
#[derive(Debug, Clone)]
pub enum GuiEvent {
    ButtonClicked(String),
    MenuItemSelected(String),
    TextChanged(String, String),
}

/// GUI manager for handling all GUI elements
pub struct GuiManager {
    elements: HashMap<String, Box<dyn GuiElement>>,
    event_queue: Vec<GuiEvent>,
    focused_element: Option<String>,
    mouse_position: Vector2,
}

impl GuiManager {
    /// Create a new GUI manager
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            event_queue: Vec::new(),
            focused_element: None,
            mouse_position: Vector2::ZERO,
        }
    }

    /// Add a GUI element
    pub fn add_element(&mut self, name: String, element: Box<dyn GuiElement>) {
        self.elements.insert(name, element);
    }

    /// Remove a GUI element
    pub fn remove_element(&mut self, name: &str) -> Option<Box<dyn GuiElement>> {
        self.elements.remove(name)
    }

    /// Get a mutable reference to an element
    pub fn get_element_mut(&mut self, name: &str) -> Option<&mut Box<dyn GuiElement>> {
        self.elements.get_mut(name)
    }

    /// Update all GUI elements
    pub fn update(&mut self, delta_time: f32) -> Result<()> {
        for element in self.elements.values_mut() {
            element.update(delta_time)?;
        }
        Ok(())
    }

    /// Render all GUI elements
    pub fn render(&self, canvas: &mut Canvas<Window>, texture_manager: &TextureManager) -> Result<()> {
        for element in self.elements.values() {
            if element.is_visible() {
                element.render(canvas, texture_manager)?;
            }
        }
        Ok(())
    }

    /// Handle mouse input
    pub fn handle_mouse(&mut self, x: i32, y: i32, pressed: bool) -> Result<()> {
        self.mouse_position = Vector2::new(x as f32, y as f32);
        
        for (name, element) in &mut self.elements {
            if element.is_visible() && element.is_enabled() {
                if element.handle_mouse(x, y, pressed)? {
                    // Element handled the input
                    if pressed {
                        self.focused_element = Some(name.clone());
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, keycode: Keycode, pressed: bool) -> Result<()> {
        // First try focused element
        if let Some(ref focused_name) = self.focused_element.clone() {
            if let Some(element) = self.elements.get_mut(focused_name) {
                if element.is_visible() && element.is_enabled() {
                    if element.handle_key(keycode, pressed)? {
                        return Ok(());
                    }
                }
            }
        }

        // If no focused element handled it, try all elements
        for element in self.elements.values_mut() {
            if element.is_visible() && element.is_enabled() {
                if element.handle_key(keycode, pressed)? {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Get and clear pending events
    pub fn get_events(&mut self) -> Vec<GuiEvent> {
        std::mem::take(&mut self.event_queue)
    }

    /// Add an event to the queue
    pub fn add_event(&mut self, event: GuiEvent) {
        self.event_queue.push(event);
    }

    /// Clear focus
    pub fn clear_focus(&mut self) {
        self.focused_element = None;
    }

    /// Set focus to a specific element
    pub fn set_focus(&mut self, name: &str) {
        if self.elements.contains_key(name) {
            self.focused_element = Some(name.to_string());
        }
    }

    /// Get the current mouse position
    pub fn mouse_position(&self) -> Vector2 {
        self.mouse_position
    }

    /// Show all elements
    pub fn show_all(&mut self) {
        for element in self.elements.values_mut() {
            element.set_visible(true);
        }
    }

    /// Hide all elements
    pub fn hide_all(&mut self) {
        for element in self.elements.values_mut() {
            element.set_visible(false);
        }
    }

    /// Enable all elements
    pub fn enable_all(&mut self) {
        for element in self.elements.values_mut() {
            element.set_enabled(true);
        }
    }

    /// Disable all elements
    pub fn disable_all(&mut self) {
        for element in self.elements.values_mut() {
            element.set_enabled(false);
        }
    }
}

impl Default for GuiManager {
    fn default() -> Self {
        Self::new()
    }
}

/// GUI layout utilities
pub mod layout {
    use super::*;

    /// Align elements horizontally
    pub fn align_horizontal(elements: &mut [&mut Box<dyn GuiElement>], spacing: f32, start_x: f32, y: f32) {
        let mut current_x = start_x;
        for element in elements {
            let bounds = element.bounds();
            // Note: This is a simplified approach - in a real implementation,
            // you'd need to modify the element's position through a setter method
            current_x += bounds.width + spacing;
        }
    }

    /// Align elements vertically
    pub fn align_vertical(elements: &mut [&mut Box<dyn GuiElement>], spacing: f32, x: f32, start_y: f32) {
        let mut current_y = start_y;
        for element in elements {
            let bounds = element.bounds();
            // Note: This is a simplified approach - in a real implementation,
            // you'd need to modify the element's position through a setter method
            current_y += bounds.height + spacing;
        }
    }

    /// Center elements in a container
    pub fn center_in_container(
        elements: &mut [&mut Box<dyn GuiElement>],
        container_rect: Rect,
    ) {
        for element in elements {
            let bounds = element.bounds();
            let center_x = container_rect.x + (container_rect.width - bounds.width) / 2.0;
            let center_y = container_rect.y + (container_rect.height - bounds.height) / 2.0;
            // Note: This is a simplified approach - in a real implementation,
            // you'd need to modify the element's position through a setter method
            let _ = (center_x, center_y); // Suppress unused variable warning
        }
    }
}

/// GUI theme system for consistent styling
#[derive(Debug, Clone)]
pub struct GuiTheme {
    /// Button normal texture
    pub button_normal: String,
    /// Button hovered texture
    pub button_hovered: String,
    /// Button pressed texture
    pub button_pressed: String,
    /// Button disabled texture
    pub button_disabled: String,
    /// Menu background texture
    pub menu_background: String,
    /// Text color (as RGB values)
    pub text_color: (u8, u8, u8),
    /// Highlight color
    pub highlight_color: (u8, u8, u8),
    /// Default font size
    pub font_size: u32,
}

impl Default for GuiTheme {
    fn default() -> Self {
        Self {
            button_normal: "button_normal".to_string(),
            button_hovered: "button_hovered".to_string(),
            button_pressed: "button_pressed".to_string(),
            button_disabled: "button_disabled".to_string(),
            menu_background: "menu_background".to_string(),
            text_color: (255, 255, 255),
            highlight_color: (255, 255, 0),
            font_size: 16,
        }
    }
}

impl GuiTheme {
    /// Create a SuperTux-style theme
    pub fn supertux_theme() -> Self {
        Self {
            button_normal: "supertux_button_normal".to_string(),
            button_hovered: "supertux_button_hovered".to_string(),
            button_pressed: "supertux_button_pressed".to_string(),
            button_disabled: "supertux_button_disabled".to_string(),
            menu_background: "supertux_menu_bg".to_string(),
            text_color: (255, 255, 255),
            highlight_color: (255, 255, 0),
            font_size: 20,
        }
    }
}

/// GUI utilities for common operations
pub mod utils {
    use super::*;

    /// Check if a point is inside a rectangle
    pub fn point_in_rect(point: Vector2, rect: Rect) -> bool {
        point.x >= rect.x
            && point.x <= rect.x + rect.width
            && point.y >= rect.y
            && point.y <= rect.y + rect.height
    }

    /// Calculate text size (placeholder - would need actual font rendering)
    pub fn calculate_text_size(text: &str, font_size: u32) -> Vector2 {
        // Simplified calculation - in a real implementation, you'd use actual font metrics
        Vector2::new(text.len() as f32 * font_size as f32 * 0.6, font_size as f32)
    }

    /// Create a centered rectangle
    pub fn centered_rect(center: Vector2, size: Vector2) -> Rect {
        Rect::new(
            center.x - size.x / 2.0,
            center.y - size.y / 2.0,
            size.x,
            size.y,
        )
    }
}