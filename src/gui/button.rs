//! Button GUI component for RustUX

use crate::util::Result;
use crate::math::{Vector2, Rect};
use crate::sprite::{Sprite, TextureManager, SpriteRenderer};
use crate::gui::{GuiElement, GuiEvent, GuiTheme, utils};
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;

/// Button states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

/// Button component
pub struct Button {
    /// Button identifier
    pub id: String,
    /// Button position
    pub position: Vector2,
    /// Button size
    pub size: Vector2,
    /// Button text
    pub text: String,
    /// Current state
    pub state: ButtonState,
    /// Whether the button is visible
    pub visible: bool,
    /// Whether the button is enabled
    pub enabled: bool,
    /// Button sprites for different states
    pub sprite_normal: Option<Sprite>,
    pub sprite_hovered: Option<Sprite>,
    pub sprite_pressed: Option<Sprite>,
    pub sprite_disabled: Option<Sprite>,
    /// Callback function name or identifier
    pub callback: Option<String>,
    /// Whether the button was just clicked
    pub just_clicked: bool,
    /// Theme for styling
    pub theme: GuiTheme,
}

impl Button {
    /// Create a new button
    pub fn new(id: String, position: Vector2, size: Vector2, text: String) -> Self {
        Self {
            id,
            position,
            size,
            text,
            state: ButtonState::Normal,
            visible: true,
            enabled: true,
            sprite_normal: None,
            sprite_hovered: None,
            sprite_pressed: None,
            sprite_disabled: None,
            callback: None,
            just_clicked: false,
            theme: GuiTheme::default(),
        }
    }

    /// Create a button with SuperTux theme
    pub fn new_supertux(id: String, position: Vector2, size: Vector2, text: String) -> Self {
        let mut button = Self::new(id, position, size, text);
        button.theme = GuiTheme::supertux_theme();
        button.setup_sprites();
        button
    }

    /// Set up sprites for different button states
    pub fn setup_sprites(&mut self) {
        self.sprite_normal = Some(Sprite::with_size(
            self.theme.button_normal.clone(),
            self.position,
            self.size,
        ));

        self.sprite_hovered = Some(Sprite::with_size(
            self.theme.button_hovered.clone(),
            self.position,
            self.size,
        ));

        self.sprite_pressed = Some(Sprite::with_size(
            self.theme.button_pressed.clone(),
            self.position,
            self.size,
        ));

        self.sprite_disabled = Some(Sprite::with_size(
            self.theme.button_disabled.clone(),
            self.position,
            self.size,
        ));
    }

    /// Set the button's callback
    pub fn set_callback(&mut self, callback: String) {
        self.callback = Some(callback);
    }

    /// Get the current sprite based on state
    fn get_current_sprite(&self) -> Option<&Sprite> {
        match self.state {
            ButtonState::Normal => self.sprite_normal.as_ref(),
            ButtonState::Hovered => self.sprite_hovered.as_ref(),
            ButtonState::Pressed => self.sprite_pressed.as_ref(),
            ButtonState::Disabled => self.sprite_disabled.as_ref(),
        }
    }

    /// Get the current sprite mutably
    fn get_current_sprite_mut(&mut self) -> Option<&mut Sprite> {
        match self.state {
            ButtonState::Normal => self.sprite_normal.as_mut(),
            ButtonState::Hovered => self.sprite_hovered.as_mut(),
            ButtonState::Pressed => self.sprite_pressed.as_mut(),
            ButtonState::Disabled => self.sprite_disabled.as_mut(),
        }
    }

    /// Check if a point is inside the button
    pub fn contains_point(&self, point: Vector2) -> bool {
        utils::point_in_rect(point, self.bounds())
    }

    /// Trigger the button click
    pub fn click(&mut self) -> Option<GuiEvent> {
        if self.enabled && self.visible {
            self.just_clicked = true;
            Some(GuiEvent::ButtonClicked(self.id.clone()))
        } else {
            None
        }
    }

    /// Set the button position
    pub fn set_position(&mut self, position: Vector2) {
        self.position = position;
        // Update sprite positions
        if let Some(ref mut sprite) = self.sprite_normal {
            sprite.position = position;
        }
        if let Some(ref mut sprite) = self.sprite_hovered {
            sprite.position = position;
        }
        if let Some(ref mut sprite) = self.sprite_pressed {
            sprite.position = position;
        }
        if let Some(ref mut sprite) = self.sprite_disabled {
            sprite.position = position;
        }
    }

    /// Set the button size
    pub fn set_size(&mut self, size: Vector2) {
        self.size = size;
        // Update sprite sizes
        if let Some(ref mut sprite) = self.sprite_normal {
            sprite.size = size;
        }
        if let Some(ref mut sprite) = self.sprite_hovered {
            sprite.size = size;
        }
        if let Some(ref mut sprite) = self.sprite_pressed {
            sprite.size = size;
        }
        if let Some(ref mut sprite) = self.sprite_disabled {
            sprite.size = size;
        }
    }

    /// Set the button text
    pub fn set_text(&mut self, text: String) {
        self.text = text;}

    /// Get the text rendering position (centered on button)
    pub fn get_text_position(&self) -> Vector2 {
        let text_size = utils::calculate_text_size(&self.text, self.theme.font_size);
        Vector2::new(
            self.position.x + (self.size.x - text_size.x) / 2.0,
            self.position.y + (self.size.y - text_size.y) / 2.0,
        )
    }
}

impl GuiElement for Button {
    fn update(&mut self, delta_time: f32) -> Result<()> {
        // Update current sprite animation
        if let Some(sprite) = self.get_current_sprite_mut() {
            sprite.update(delta_time);
        }

        // Reset just_clicked flag
        self.just_clicked = false;

        Ok(())
    }

    fn render(&self, canvas: &mut Canvas<Window>, texture_manager: &TextureManager) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        // Render button sprite
        if let Some(sprite) = self.get_current_sprite() {
            SpriteRenderer::render_sprite(canvas, texture_manager, sprite)?;
        }

        // TODO: Render text (would need font rendering system)
        // For now, we'll just render the sprite

        Ok(())
    }

    fn handle_mouse(&mut self, x: i32, y: i32, pressed: bool) -> Result<bool> {
        if !self.visible || !self.enabled {
            return Ok(false);
        }

        let mouse_pos = Vector2::new(x as f32, y as f32);
        let inside = self.contains_point(mouse_pos);

        if inside {
            if pressed {
                self.state = ButtonState::Pressed;
                self.just_clicked = true;
            } else {
                self.state = ButtonState::Hovered;
            }
            return Ok(true);
        } else {
            if self.state != ButtonState::Disabled {
                self.state = ButtonState::Normal;
            }
        }

        Ok(false)
    }

    fn handle_key(&mut self, keycode: Keycode, pressed: bool) -> Result<bool> {
        if !self.visible || !self.enabled {
            return Ok(false);
        }

        // Handle Enter/Space as button activation
        if pressed && (keycode == Keycode::Return || keycode == Keycode::Space) {
            self.just_clicked = true;
            return Ok(true);
        }

        Ok(false)
    }

    fn bounds(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.state = if enabled {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        };
    }
}

/// Button builder for easy button creation
pub struct ButtonBuilder {
    id: String,
    position: Vector2,
    size: Vector2,
    text: String,
    callback: Option<String>,
    theme: GuiTheme,
}

impl ButtonBuilder {
    /// Create a new button builder
    pub fn new(id: String) -> Self {
        Self {
            id,
            position: Vector2::ZERO,
            size: Vector2::new(200.0, 50.0),
            text: String::new(),
            callback: None,
            theme: GuiTheme::default(),
        }
    }

    /// Set the button position
    pub fn position(mut self, position: Vector2) -> Self {
        self.position = position;
        self
    }

    /// Set the button size
    pub fn size(mut self, size: Vector2) -> Self {
        self.size = size;
        self
    }

    /// Set the button text
    pub fn text(mut self, text: String) -> Self {
        self.text = text;
        self
    }

    /// Set the button callback
    pub fn callback(mut self, callback: String) -> Self {
        self.callback = Some(callback);
        self
    }

    /// Use SuperTux theme
    pub fn supertux_theme(mut self) -> Self {
        self.theme = GuiTheme::supertux_theme();
        self
    }

    /// Build the button
    pub fn build(self) -> Button {
        let mut button = Button::new(self.id, self.position, self.size, self.text);
        button.theme = self.theme;
        button.callback = self.callback;
        button.setup_sprites();
        button
    }
}