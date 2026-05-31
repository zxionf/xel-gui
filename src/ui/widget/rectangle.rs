use crate::render::RenderContext;
use crate::ui::widget::{Rect, Widget};

/// 纯色矩形 Widget。
///
/// 示例：
/// ```ignore
/// ui.add(Box::new(Rectangle::new(10, 20, 200, 100, [0.2, 0.5, 0.8, 1.0])));
/// ```
pub struct Rectangle {
    bounds: Rect,
    color: [f32; 4],
    visible: bool,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, w: i32, h: i32, color: [f32; 4]) -> Self {
        Self {
            bounds: Rect::new(x, y, w, h),
            color,
            visible: true,
        }
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }
}

impl Widget for Rectangle {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn bounds_mut(&mut self) -> &mut Rect {
        &mut self.bounds
    }

    fn draw(&self, renderer: &mut RenderContext) {
        if self.visible {
            renderer.ui.fill_rect(
                self.bounds.x as f32,
                self.bounds.y as f32,
                self.bounds.w as f32,
                self.bounds.h as f32,
                self.color,
            );
        }
    }
}
