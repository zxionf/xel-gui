pub mod button;
pub mod color;
pub mod widget;

use crate::render::Renderer2D;
use widget::Widget;

pub struct UiRoot {
    widgets: Vec<Box<dyn Widget>>,
    hovered: Option<usize>,
    active: Option<usize>,
}

impl UiRoot {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            hovered: None,
            active: None,
        }
    }

    pub fn add(&mut self, widget: impl Widget + 'static) -> usize {
        let idx = self.widgets.len();
        self.widgets.push(Box::new(widget));
        idx
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Box<dyn Widget>> {
        self.widgets.get_mut(index)
    }

    pub fn draw(&self, renderer: &mut Renderer2D) {
        for w in &self.widgets {
            w.draw(renderer);
        }
    }

    pub fn handle_mouse_move(&mut self, px: f32, py: f32) -> Option<usize> {
        let mut new_hovered = None;
        for (i, w) in self.widgets.iter().enumerate().rev() {
            if w.hit_test(px, py) {
                new_hovered = Some(i);
                break;
            }
        }

        if new_hovered != self.hovered {
            if let Some(old) = self.hovered {
                self.widgets[old].on_mouse_leave();
            }
            if let Some(new) = new_hovered {
                self.widgets[new].on_mouse_enter();
            }
            self.hovered = new_hovered;
        }

        new_hovered
    }

    pub fn handle_mouse_down(&mut self, px: f32, py: f32) -> bool {
        for (i, w) in self.widgets.iter_mut().enumerate().rev() {
            if w.hit_test(px, py) {
                if w.on_mouse_down(px, py) {
                    self.active = Some(i);
                    return true;
                }
            }
        }
        false
    }

    pub fn handle_mouse_up(&mut self, px: f32, py: f32) -> bool {
        let active = self.active.take();
        if let Some(idx) = active {
            self.widgets[idx].on_mouse_up(px, py);
            return true;
        }
        if let Some(idx) = self.hovered {
            return self.widgets[idx].on_mouse_up(px, py);
        }
        false
    }
}

impl Default for UiRoot {
    fn default() -> Self {
        Self::new()
    }
}
