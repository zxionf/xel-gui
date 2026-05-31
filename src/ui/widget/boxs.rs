use crate::{render::RenderContext, ui::widget::{Rect, Widget}};

// 垂直布局
pub struct Vbox {
    widgets: Vec<Box<dyn Widget>>,
    bounds: Rect,
    visible: bool,
    debug: bool,
}
// 水平布局
pub struct Hbox {}

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

    fn bounds_mut(&mut self) -> &mut Rect {
        &mut self.bounds
    }

    fn draw(&self, renderer: &mut RenderContext) {
        for w in self.widgets.iter() {
            w.draw(renderer);
            if self.debug {
                w.draw_debug(renderer);
            }
        }
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    fn set_debug(&mut self, debuged: bool) {
        self.debug = debuged;
    }

    fn layout(&mut self) {
        let mut y = self.bounds.y;
        for widget in self.widgets.iter_mut() {
            widget.set_position(self.bounds.x, y);
            widget.set_size(self.bounds.w, widget.bounds().h);
            y += widget.bounds().h;
        }
    }
}
