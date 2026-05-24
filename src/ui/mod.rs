pub mod button;
pub mod color;
pub mod widget;

use crate::render::Renderer2D;
use widget::Widget;

/// UI 根容器，持有所有组件并负责事件分发。
pub struct UiRoot {
    widgets: Vec<Box<dyn Widget>>,
    /// 当前被悬停的 widget 索引（用于 enter/leave 事件）。
    hovered: Option<usize>,
    /// 当前被按下的 widget 索引（用于拖拽/释放）。
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

    /// 添加一个组件，返回其索引。
    pub fn add(&mut self, widget: impl Widget + 'static) -> usize {
        let idx = self.widgets.len();
        self.widgets.push(Box::new(widget));
        idx
    }

    /// 获取组件的可变引用（按索引）。
    #[allow(dead_code)]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Box<dyn Widget>> {
        self.widgets.get_mut(index)
    }

    /// 绘制所有组件。
    pub fn draw(&self, renderer: &mut Renderer2D) {
        for w in &self.widgets {
            w.draw(renderer);
        }
    }

    /// 分发鼠标移动事件。返回消费了事件的 widget 索引（如果有）。
    pub fn handle_mouse_move(&mut self, px: f32, py: f32) -> Option<usize> {
        let mut new_hovered = None;
        for (i, w) in self.widgets.iter().enumerate().rev() {
            if w.hit_test(px, py) {
                new_hovered = Some(i);
                break;
            }
        }

        if new_hovered != self.hovered {
            eprintln!(
                "[UiRoot] hover 变化: {:?} → {:?}  @ ({px:.0},{py:.0})",
                self.hovered, new_hovered
            );
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

    /// 分发鼠标按下事件。
    pub fn handle_mouse_down(&mut self, px: f32, py: f32) -> bool {
        eprintln!("[UiRoot] mouse_down @ ({px:.0},{py:.0})");
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

    /// 分发鼠标释放事件。
    pub fn handle_mouse_up(&mut self, px: f32, py: f32) -> bool {
        eprintln!("[UiRoot] mouse_up @ ({px:.0},{py:.0})");
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
