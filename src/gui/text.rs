//! Text GUI component for RustUX

use crate::util::Result;
use crate::math::{Vector2, Rect};
use crate::sprite::{TextureManager};
use crate::gui::{GuiElement, GuiEvent, GuiTheme, utils};
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Text component for displaying text
pub struct Text {
    /// Text identifier
    pub id: String,
    /// Text content
    pub content: String,
    /// Text position
    pub position: Vector2,
    /// Text color (RGB)
    pub color: (u8, u8, u8),
    /// Font size
    pub font_size: u32,
    /// Text alignment
    pub alignment: TextAlign,
    /// Whether the text is visible
    pub visible: bool,
    /// Whether the text is enabled (for interaction)
    pub enabled: bool,
    /// Maximum width for text wrapping (0 = no wrapping)
    pub max_width: f32,
    /// Line spacing multiplier
    pub line_spacing: f32,
    /// Theme for styling
    pub theme: GuiTheme,
    /// Whether the text is selectable
    pub selectable: bool,
    /// Whether the text is currently selected
    pub selected: bool,
}

impl Text {
    /// Create a new text component
    pub fn new(id: String, content: String, position: Vector2) -> Self {
        Self {
            id,
            content,
            position,
            color: (255, 255, 255),
            font_size: 16,
            alignment: TextAlign::Left,
            visible: true,
            enabled: true,
            max_width: 0.0,
            line_spacing: 1.2,
            theme: GuiTheme::default(),
            selectable: false,
            selected: false,
        }
    }

    /// Create a SuperTux-style text
    pub fn new_supertux(id: String, content: String, position: Vector2) -> Self {
        let mut text = Self::new(id, content, position);
        text.theme = GuiTheme::supertux_theme();
        text.color = text.theme.text_color;
        text.font_size = text.theme.font_size;
        text
    }

    /// Set the text content
    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }

    /// Set the text color
    pub fn set_color(&mut self, color: (u8, u8, u8)) {
        self.color = color;
    }

    /// Set the font size
    pub fn set_font_size(&mut self, size: u32) {
        self.font_size = size;
    }

    /// Set the text alignment
    pub fn set_alignment(&mut self, alignment: TextAlign) {
        self.alignment = alignment;
    }

    /// Set the maximum width for text wrapping
    pub fn set_max_width(&mut self, width: f32) {
        self.max_width = width;
    }

    /// Set whether the text is selectable
    pub fn set_selectable(&mut self, selectable: bool) {
        self.selectable = selectable;
    }

    /// Get the calculated text size
    pub fn get_text_size(&self) -> Vector2 {
        if self.max_width > 0.0 {
            // Calculate wrapped text size
            self.calculate_wrapped_size()
        } else {
            // Single line text size
            utils::calculate_text_size(&self.content, self.font_size)
        }
    }

    /// Calculate the size of wrapped text
    fn calculate_wrapped_size(&self) -> Vector2 {
        let lines = self.wrap_text();
        let line_height = self.font_size as f32 * self.line_spacing;
        let height = lines.len() as f32 * line_height;
        
        let max_line_width = lines.iter()
            .map(|line| utils::calculate_text_size(line, self.font_size).x)
            .fold(0.0, f32::max);
        
        Vector2::new(max_line_width, height)
    }

    /// Wrap text to fit within max_width
    fn wrap_text(&self) -> Vec<String> {
        if self.max_width <= 0.0 {
            return vec![self.content.clone()];
        }

        let mut lines = Vec::new();
        let words: Vec<&str> = self.content.split_whitespace().collect();
        let mut current_line = String::new();

        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            let test_width = utils::calculate_text_size(&test_line, self.font_size).x;
            
            if test_width <= self.max_width {
                current_line = test_line;
            } else {
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    /// Get the rendering position for a line of text
    fn get_line_position(&self, line: &str, line_index: usize) -> Vector2 {
        let line_height = self.font_size as f32 * self.line_spacing;
        let y = self.position.y + line_index as f32 * line_height;
        
        let x = match self.alignment {
            TextAlign::Left => self.position.x,
            TextAlign::Center => {
                let line_width = utils::calculate_text_size(line, self.font_size).x;
                let available_width = if self.max_width > 0.0 { self.max_width } else { line_width };
                self.position.x + (available_width - line_width) / 2.0
            }
            TextAlign::Right => {
                let line_width = utils::calculate_text_size(line, self.font_size).x;
                let available_width = if self.max_width > 0.0 { self.max_width } else { line_width };
                self.position.x + available_width - line_width
            }
        };

        Vector2::new(x, y)
    }

    /// Check if a point is inside the text bounds
    pub fn contains_point(&self, point: Vector2) -> bool {
        utils::point_in_rect(point, self.bounds())
    }
}

impl GuiElement for Text {
    fn update(&mut self, _delta_time: f32) -> Result<()> {
        // Text doesn't need updating unless it's animated
        Ok(())
    }

    fn render(&self, _canvas: &mut Canvas<Window>, _texture_manager: &TextureManager) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        // TODO: Implement actual text rendering
        // This would require a font rendering system (like SDL2_ttf)
        // For now, this is a placeholder

        // The actual implementation would:
        // 1. Load the font
        // 2. Create a texture from the text
        // 3. Render the texture to the canvas
        // 4. Handle text wrapping and alignment

        Ok(())
    }

    fn handle_mouse(&mut self, x: i32, y: i32, pressed: bool) -> Result<bool> {
        if !self.visible || !self.enabled || !self.selectable {
            return Ok(false);
        }

        let mouse_pos = Vector2::new(x as f32, y as f32);
        let inside = self.contains_point(mouse_pos);

        if inside && pressed {
            self.selected = !self.selected;
            return Ok(true);
        }

        Ok(false)
    }

    fn handle_key(&mut self, _keycode: Keycode, _pressed: bool) -> Result<bool> {
        // Text components typically don't handle keyboard input
        // unless they're editable text fields
        Ok(false)
    }

    fn bounds(&self) -> Rect {
        let size = self.get_text_size();
        Rect::new(self.position.x, self.position.y, size.x, size.y)
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
    }
}

/// Text builder for easy text creation
pub struct TextBuilder {
    id: String,
    content: String,
    position: Vector2,
    color: (u8, u8, u8),
    font_size: u32,
    alignment: TextAlign,
    max_width: f32,
    selectable: bool,
    theme: GuiTheme,
}

impl TextBuilder {
    /// Create a new text builder
    pub fn new(id: String, content: String) -> Self {
        Self {
            id,
            content,
            position: Vector2::ZERO,
            color: (255, 255, 255),
            font_size: 16,
            alignment: TextAlign::Left,
            max_width: 0.0,
            selectable: false,
            theme: GuiTheme::default(),
        }
    }

    /// Set the text position
    pub fn position(mut self, position: Vector2) -> Self {
        self.position = position;
        self
    }

    /// Set the text color
    pub fn color(mut self, color: (u8, u8, u8)) -> Self {
        self.color = color;
        self
    }

    /// Set the font size
    pub fn font_size(mut self, size: u32) -> Self {
        self.font_size = size;
        self
    }

    /// Set the text alignment
    pub fn alignment(mut self, alignment: TextAlign) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set the maximum width for wrapping
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }

    /// Make the text selectable
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Use SuperTux theme
    pub fn supertux_theme(mut self) -> Self {
        self.theme = GuiTheme::supertux_theme();
        self.color = self.theme.text_color;
        self.font_size = self.theme.font_size;
        self
    }

    /// Build the text component
    pub fn build(self) -> Text {
        let mut text = Text::new(self.id, self.content, self.position);
        text.color = self.color;
        text.font_size = self.font_size;
        text.alignment = self.alignment;
        text.max_width = self.max_width;
        text.selectable = self.selectable;
        text.theme = self.theme;
        text
    }
}