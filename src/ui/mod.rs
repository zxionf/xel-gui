pub mod button;
pub mod color;
pub mod label;
pub mod widget;

use crate::render::Renderer2D;
use widget::Widget;

const DEBUG_BORDER_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 0.8];
const DEBUG_BORDER_WIDTH: f32 = 1.0;

pub struct UiRoot {
    widgets: Vec<Box<dyn Widget>>,
    hovered: Option<usize>,
    active: Option<usize>,
    /// 调试模式：打印事件 + 绘制边框。
    debug: bool,
}

impl UiRoot {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            hovered: None,
            active: None,
            debug: false,
        }
    }

    /// 开启/关闭调试模式。
    pub fn set_debug(&mut self, enabled: bool) {
        self.debug = enabled;
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
        // 调试：为每个 widget 绘制绿色边框
        if self.debug {
            for w in &self.widgets {
                let b = w.bounds();
                renderer.stroke_rect(
                    b.x,
                    b.y,
                    b.w,
                    b.h,
                    DEBUG_BORDER_WIDTH,
                    DEBUG_BORDER_COLOR,
                );
            }
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
            if self.debug {
                eprintln!(
                    "[UiRoot] hover {:?} → {:?} @ ({px:.0},{py:.0})",
                    self.hovered, new_hovered
                );
            }
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
        if self.debug {
            eprintln!("[UiRoot] mouse_down @ ({px:.0},{py:.0})");
        }
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
        if self.debug {
            eprintln!("[UiRoot] mouse_up @ ({px:.0},{py:.0})");
        }
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
