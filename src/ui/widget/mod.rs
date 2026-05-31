pub mod rectangle;
pub mod boxs;
pub mod layout;
pub mod label;

use crate::render::RenderContext;

#[derive(Copy, Clone, Debug, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }
}

#[allow(unused)]
pub trait Widget {
    fn bounds(&self) -> Rect;
    fn bounds_mut(&mut self) -> &mut Rect;
    // fn draw<T: Renderer>(&self, renderer: &mut T);
    fn draw(&self, renderer: &mut RenderContext);
    fn draw_debug(&self, renderer: &mut RenderContext) {
        let b = self.bounds();
        renderer.ui.stroke_rect(b.x as f32, b.y as f32, b.w as f32, b.h as f32, 2.0, [0.0, 1.0, 0.0, 1.0]);
    }
    fn set_bounds(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.set_position(x, y);
        self.set_size(w, h);
    }
    fn set_bounds_rect(&mut self, rect: Rect) { self.set_bounds(rect.x, rect.y, rect.w, rect.h); }
    fn set_position(&mut self, x: i32, y: i32) {
        self.bounds_mut().x = x;
        self.bounds_mut().y = y;
    }
    fn set_size(&mut self, w: i32, h: i32) {
        self.bounds_mut().w = w;
        self.bounds_mut().h = h;
    }
    fn set_visible(&mut self, visible: bool) { }
    fn set_debug(&mut self, debuged: bool) { }
    fn hit_test(&self, px: i32, py: i32) -> bool {
        self.bounds().contains(px, py)
    }
    fn on_mouse_down(&mut self, px: i32, py: i32) -> bool { false }
    fn on_mouse_up(&mut self, px: i32, py: i32) -> bool { false }
    fn on_mouse_enter(&mut self) { }
    fn on_mouse_leave(&mut self) { }
    fn on_mouse_wheel(&mut self, dx:f32, dy:f32) {}

    fn layout(&mut self) { }
}
