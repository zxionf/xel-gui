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
    fn set_position(&mut self, x: i32, y: i32);
    // fn draw<T: Renderer>(&self, renderer: &mut T);
    fn draw(&self, renderer: &mut RenderContext);
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
