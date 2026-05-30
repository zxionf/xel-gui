// 垂直布局
pub struct Vbox {
    widgets: Vec<Box<dyn Widget>>,
    bounds: Rect,
    visible: bool,
    debug: bool,
}
// 水平布局
pub struct Hbox {}

use crate::ui::widget::{Rect, Widget};

impl Vbox {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            widgets: vec![],
            bounds: Rect::new(x, y, w, h),
            visible: true,
            debug: false,
        }
    }
    pub fn add(&mut self, widget: Box<dyn Widget>) {
        self.widgets.push(widget);
        self.layout();
    }
}

impl Widget for Vbox {
    fn bounds(&self) -> super::Rect {
        self.bounds
    }

    fn set_position(&mut self, x: i32, y: i32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn draw(&self, renderer: &mut crate::ui::renderer::UIRenderer) {
        todo!()
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    fn set_debug(&mut self, debuged: bool) {
        self.debug = debuged;
    }
}
