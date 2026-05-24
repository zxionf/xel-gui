use crate::render::Renderer2D;
use crate::ui::color::Color;
use crate::ui::widget::{Rect, Widget};

/// 简单的文本标签。
pub struct Label {
    bounds: Rect,
    text: String,
    font_size: f32,
    color: Color,
}

impl Label {
    pub fn new(x: f32, y: f32, text: impl Into<String>, font_size: f32) -> Self {
        Self {
            bounds: Rect::new(x, y, 0.0, font_size),
            text: text.into(),
            font_size,
            color: Color::WHITE,
        }
    }

    /// 设置文本颜色。
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// 设置文本内容。
    #[allow(dead_code)]
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// 获取当前文本。
    #[allow(dead_code)]
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl Widget for Label {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn draw(&self, renderer: &mut Renderer2D) {
        renderer.draw_text(
            &self.text,
            self.bounds.x,
            self.bounds.y,
            self.font_size,
            self.color.to_array(),
        );
    }
}
