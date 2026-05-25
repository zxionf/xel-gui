use crate::render::Renderer2D;
use crate::ui::color::Color;
use crate::ui::widget::{Rect, Widget};

#[derive(Copy, Clone, Debug, PartialEq)]
enum ButtonState {
    Normal,
    Hovered,
    Pressed,
}

pub struct Button {
    bounds: Rect,
    #[allow(dead_code)]
    label: String,
    state: ButtonState,
    on_click: Box<dyn FnMut() -> bool>,
}

impl Button {
    pub fn new(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        label: impl Into<String>,
        on_click: impl FnMut() -> bool + 'static,
    ) -> Self {
        Self {
            bounds: Rect::new(x, y, w, h),
            label: label.into(),
            state: ButtonState::Normal,
            on_click: Box::new(on_click),
        }
    }

    fn current_color(&self) -> Color {
        match self.state {
            ButtonState::Normal => Color::new(0.25, 0.55, 0.9, 1.0),
            ButtonState::Hovered => Color::new(0.3, 0.6, 1.0, 1.0),
            ButtonState::Pressed => Color::new(0.15, 0.4, 0.7, 1.0),
        }
    }
}

impl Widget for Button {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn draw(&self, renderer: &mut Renderer2D) {
        renderer.fill_rect(
            self.bounds.x,
            self.bounds.y,
            self.bounds.w,
            self.bounds.h,
            self.current_color().to_array(),
        );
        renderer.draw_text(
            &self.label,
            self.bounds.x,
            self.bounds.y,
            16.0,
            Color::BLACK.to_array(),
        );
    }

    fn on_mouse_down(&mut self, _px: f32, _py: f32) -> bool {
        self.state = ButtonState::Pressed;
        true
    }

    fn on_mouse_up(&mut self, px: f32, py: f32) -> bool {
        if self.state == ButtonState::Pressed {
            self.state = ButtonState::Hovered;
            if self.bounds.contains(px, py) {
                return (self.on_click)();
            }
        }
        self.state = ButtonState::Normal;
        true
    }

    fn on_mouse_enter(&mut self) {
        self.state = ButtonState::Hovered;
    }

    fn on_mouse_leave(&mut self) {
        self.state = ButtonState::Normal;
    }
}
