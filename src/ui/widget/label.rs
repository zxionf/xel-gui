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
            bounds: Rect::new(0, 0, 0, 60),
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

    fn bounds_mut(&mut self) -> &mut Rect {
        &mut self.bounds
    }

    fn draw(&self, renderer: &mut RenderContext) {
        renderer.text.draw_text(self.bounds.x as f32, self.bounds.y as f32, 16.0, 32.0, &self.text[..], [1.0,0.0,0.0,1.0]);
    }
}