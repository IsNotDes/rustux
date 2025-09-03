//! Menu GUI component for RustUX

use crate::util::Result;
use crate::math::{Vector2, Rect};
use crate::sprite::{Sprite, TextureManager, SpriteRenderer};
use crate::gui::{GuiElement, GuiEvent, GuiTheme, Button, utils};
use crate::gui::button::ButtonBuilder;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;

/// Menu item configuration
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Item identifier
    pub id: String,
    /// Display text
    pub text: String,
    /// Whether the item is enabled
    pub enabled: bool,
    /// Callback identifier
    pub callback: Option<String>,
}

impl MenuItem {
    /// Create a new menu item
    pub fn new(id: String, text: String) -> Self {
        Self {
            id,
            text,
            enabled: true,
            callback: None,
        }
    }

    /// Set the callback
    pub fn with_callback(mut self, callback: String) -> Self {
        self.callback = Some(callback);
        self
    }

    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Menu component
pub struct Menu {
    /// Menu identifier
    pub id: String,
    /// Menu position
    pub position: Vector2,
    /// Menu size
    pub size: Vector2,
    /// Menu title
    pub title: String,
    /// Menu items
    pub items: Vec<MenuItem>,
    /// Menu buttons (generated from items)
    pub buttons: HashMap<String, Button>,
    /// Currently selected item index
    pub selected_index: usize,
    /// Whether the menu is visible
    pub visible: bool,
    /// Whether the menu is enabled
    pub enabled: bool,
    /// Background sprite
    pub background_sprite: Option<Sprite>,
    /// Theme for styling
    pub theme: GuiTheme,
    /// Item spacing
    pub item_spacing: f32,
    /// Menu padding
    pub padding: Vector2,
}

impl Menu {
    /// Create a new menu
    pub fn new(id: String, position: Vector2, title: String) -> Self {
        Self {
            id,
            position,
            size: Vector2::new(400.0, 300.0),
            title,
            items: Vec::new(),
            buttons: HashMap::new(),
            selected_index: 0,
            visible: true,
            enabled: true,
            background_sprite: None,
            theme: GuiTheme::default(),
            item_spacing: 60.0,
            padding: Vector2::new(20.0, 20.0),
        }
    }

    /// Create a SuperTux-style menu
    pub fn new_supertux(id: String, position: Vector2, title: String) -> Self {
        let mut menu = Self::new(id, position, title);
        menu.theme = GuiTheme::supertux_theme();
        menu.setup_background();
        menu
    }

    /// Add a menu item
    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
        self.rebuild_buttons();
    }

    /// Add multiple menu items
    pub fn add_items(&mut self, items: Vec<MenuItem>) {
        self.items.extend(items);
        self.rebuild_buttons();
    }

    /// Remove a menu item by ID
    pub fn remove_item(&mut self, id: &str) -> bool {
        if let Some(pos) = self.items.iter().position(|item| item.id == id) {
            self.items.remove(pos);
            self.buttons.remove(id);
            
            // Adjust selected index if necessary
            if self.selected_index >= self.items.len() && !self.items.is_empty() {
                self.selected_index = self.items.len() - 1;
            }
            
            self.rebuild_buttons();
            true
        } else {
            false
        }
    }

    /// Set up the background sprite
    pub fn setup_background(&mut self) {
        self.background_sprite = Some(Sprite::with_size(
            self.theme.menu_background.clone(),
            self.position,
            self.size,
        ));
    }

    /// Rebuild buttons from menu items
    fn rebuild_buttons(&mut self) {
        self.buttons.clear();
        
        let button_width = self.size.x -2.0 * self.padding.x;
        let button_height = 40.0;
        let start_y = self.position.y + self.padding.y + 60.0; // Space for title
        
        for (index, item) in self.items.iter().enumerate() {
            let button_pos = Vector2::new(
                self.position.x + self.padding.x,
                start_y + index as f32 * self.item_spacing,
            );
            
            let mut button = ButtonBuilder::new(item.id.clone())
                .position(button_pos)
                .size(Vector2::new(button_width, button_height))
                .text(item.text.clone())
                .supertux_theme()
                .build();
            
            if let Some(ref callback) = item.callback {
                button.set_callback(callback.clone());
            }
            
            button.set_enabled(item.enabled);
            
            self.buttons.insert(item.id.clone(), button);
        }
        
        // Update menu size based on content
        let content_height = self.items.len() as f32 * self.item_spacing + 100.0; // Extra space for title and padding
        self.size.y = content_height.max(300.0);
        
        // Update background sprite size
        if let Some(ref mut bg_sprite) = self.background_sprite {
            bg_sprite.size = self.size;
        }
    }

    /// Select the next menu item
    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.items.len();
            self.update_button_states();
        }
    }

    /// Select the previous menu item
    pub fn select_previous(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.items.len() - 1
            } else {
                self.selected_index - 1
            };
            self.update_button_states();
        }
    }

    /// Activate the currently selected item
    pub fn activate_selected(&mut self) -> Option<GuiEvent> {
        if let Some(item) = self.items.get(self.selected_index) {
            if item.enabled {
                return Some(GuiEvent::MenuItemSelected(item.id.clone()));
            }
        }
        None
    }

    /// Update button visual states based on selection
    fn update_button_states(&mut self) {
        for (index, item) in self.items.iter().enumerate() {
            if let Some(button) = self.buttons.get_mut(&item.id) {
                // Highlight selected button
                if index == self.selected_index {
                    // TODO: Set button to highlighted state
                    // This would require extending the button state system
                }
            }
        }
    }

    /// Set the menu position
    pub fn set_position(&mut self, position: Vector2) {
        self.position = position;
        
        // Update background sprite position
        if let Some(ref mut bg_sprite) = self.background_sprite {
            bg_sprite.position = position;
        }
        
        // Rebuild buttons to update their positions
        self.rebuild_buttons();
    }

    /// Set the menu size
    pub fn set_size(&mut self, size: Vector2) {
        self.size = size;
        
        // Update background sprite size
        if let Some(ref mut bg_sprite) = self.background_sprite {
            bg_sprite.size = size;
        }
        
        // Rebuild buttons to update their positions and sizes
        self.rebuild_buttons();
    }

    /// Get the currently selected item
    pub fn get_selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected_index)
    }

    /// Set the selected item by ID
    pub fn set_selected_item(&mut self, id: &str) -> bool {
        if let Some(index) = self.items.iter().position(|item| item.id == id) {
            self.selected_index = index;
            self.update_button_states();
            true
        } else {
            false
        }
    }

    /// Get the title rendering position
    pub fn get_title_position(&self) -> Vector2 {
        let title_size = utils::calculate_text_size(&self.title, self.theme.font_size + 4);
        Vector2::new(
            self.position.x + (self.size.x - title_size.x) / 2.0,
            self.position.y + self.padding.y,
        )
    }
}

impl GuiElement for Menu {
    fn update(&mut self, delta_time: f32) -> Result<()> {
        // Update background sprite
        if let Some(ref mut bg_sprite) = self.background_sprite {
            bg_sprite.update(delta_time);
        }
        
        // Update all buttons
        for button in self.buttons.values_mut() {
            button.update(delta_time)?;
        }
        
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas<Window>, texture_manager: &TextureManager) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        // Render background
        if let Some(ref bg_sprite) = self.background_sprite {
            SpriteRenderer::render_sprite(canvas, texture_manager, bg_sprite)?;
        }

        // Render all buttons
        for button in self.buttons.values() {
            button.render(canvas, texture_manager)?;
        }

        // TODO: Render title text (would need font rendering system)

        Ok(())
    }

    fn handle_mouse(&mut self, x: i32, y: i32, pressed: bool) -> Result<bool> {
        if !self.visible || !self.enabled {
            return Ok(false);
        }

        // Check if mouse is within menu bounds
        let mouse_pos = Vector2::new(x as f32, y as f32);
        if !utils::point_in_rect(mouse_pos, self.bounds()) {
            return Ok(false);
        }

        // Handle button interactions
        for (item_id, button) in &mut self.buttons {
            if button.handle_mouse(x, y, pressed)? {
                // Update selected index based on clicked button
                if let Some(index) = self.items.iter().position(|item| item.id == *item_id) {
                    self.selected_index = index;
                    self.update_button_states();
                }
                return Ok(true);
            }
        }

        Ok(true) // Consumed the input even if no button was clicked
    }

    fn handle_key(&mut self, keycode: Keycode, pressed: bool) -> Result<bool> {
        if !self.visible || !self.enabled || !pressed {
            return Ok(false);
        }

        match keycode {
            Keycode::Up => {
                self.select_previous();
                Ok(true)
            }
            Keycode::Down => {
                self.select_next();
                Ok(true)
            }
            Keycode::Return | Keycode::Space => {
                // Activate selected item
                if let Some(item) = self.items.get(self.selected_index) {
                    if let Some(button) = self.buttons.get_mut(&item.id) {
                        button.handle_key(keycode, pressed)?;
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn bounds(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;for button in self.buttons.values_mut() {
            button.set_visible(visible);
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        for button in self.buttons.values_mut() {
            button.set_enabled(enabled);
        }
    }
}

/// Menu builder for easy menu creation
pub struct MenuBuilder {
    id: String,
    position: Vector2,
    title: String,
    items: Vec<MenuItem>,
    theme: GuiTheme,
}

impl MenuBuilder {
    /// Create a new menu builder
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            position: Vector2::ZERO,
            title,
            items: Vec::new(),
            theme: GuiTheme::default(),
        }
    }

    /// Set the menu position
    pub fn position(mut self, position: Vector2) -> Self {
        self.position = position;
        self
    }

    /// Add a menu item
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add multiple menu items
    pub fn items(mut self, items: Vec<MenuItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// Use SuperTux theme
    pub fn supertux_theme(mut self) -> Self {
        self.theme = GuiTheme::supertux_theme();
        self
    }

    /// Build the menu
    pub fn build(self) -> Menu {
        let mut menu = Menu::new(self.id, self.position, self.title);
        let is_supertux = matches!(self.theme.menu_background.as_str(), "supertux_menu_bg");
        menu.theme = self.theme;
        menu.add_items(self.items);
        if is_supertux {
            menu.setup_background();
        }
        menu
    }
}