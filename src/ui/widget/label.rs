use super::{Rect, Widget};
use crate::render::RenderContext;

pub struct Label {
    bounds: Rect,
    text: String,
    visible: bool,
    debug: bool,
}

impl Label {
    pub fn new(text:String) -> Self {
        Self {
            bounds: Rect::new(0, 0, 0, 0),
            text,
            visible: true,
            debug: false,
        }
    }
}

impl Widget for Label {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: i32, y: i32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn draw(&self, renderer: &mut RenderContext) {
        todo!()
    }
}