pub mod debug;
pub mod renderer;
pub mod widget;

use debug::UIDebugFlag;
use widget::Widget;

use crate::render::RenderContext;

pub struct UIRoot {
    widgets: Vec<Box<dyn Widget>>,
    hovered: Option<usize>,
    active: Option<usize>,
    debug_flags: UIDebugFlag,
}

impl UIRoot {
    pub fn new() -> Self {
        Self {
            widgets: vec![],
            hovered: None,
            active: None,
            debug_flags: UIDebugFlag::NONE,
        }
    }

    pub fn set_debug(&mut self, flags: UIDebugFlag) {
        self.debug_flags = flags;
    }

    pub fn add(&mut self, widget: Box<dyn Widget>) {
        self.widgets.push(widget);
    }

    /// 遍历所有 widget，向 UI 渲染器提交绘制数据。
    /// 调用前应先 `renderer.begin_frame(w, h)`，调用后应 `renderer.upload(queue)`。
    pub fn draw(&mut self, render_context: &mut RenderContext) {
        for w in self.widgets.iter() {
            w.draw(render_context);

            if self.debug_flags.contains(UIDebugFlag::DEBUG_WIDGETS) {
                let b = w.bounds();
                render_context.ui.stroke_rect(b.x as f32, b.y as f32, b.w as f32, b.h as f32, 2.0, [0.0, 1.0, 0.0, 1.0]);
            }
        }
    }

    pub fn handle_mouse_move(&mut self, px: i32, py: i32) -> Option<usize> {
        let mut new_hovered = None;
        for (i, w) in self.widgets.iter().enumerate().rev() {
            if w.hit_test(px, py) {
                new_hovered = Some(i);
                break;
            }
        }

        if new_hovered != self.hovered {
            if self.debug_flags.contains(UIDebugFlag::DEBUG_EVENTS) {
                eprintln!(
                    "[UIRoot] hover {:?} → {:?} @ ({px:.0},{py:.0})",
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

    pub fn handle_mouse_down(&mut self, px: i32, py: i32) -> bool {
        if self.debug_flags.contains(UIDebugFlag::DEBUG_EVENTS) {
            eprintln!("[UIRoot] mouse_down @ ({px:.0},{py:.0})");
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

    pub fn handle_mouse_up(&mut self, px: i32, py: i32) -> bool {
        if self.debug_flags.contains(UIDebugFlag::DEBUG_EVENTS) {
            eprintln!("[UIRoot] mouse_up @ ({px:.0},{py:.0})");
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

    pub fn handle_mouse_wheel(&mut self, dx:f32, dy:f32) {
        if self.debug_flags.contains(UIDebugFlag::DEBUG_EVENTS) {
            eprintln!("[UIRoot] mouse_wheel {dx} {dy}");
        }
        for w in self.widgets.iter_mut() {
            w.on_mouse_wheel(dx, dy);
        }
    }
}
