use crate::render::Renderer2D;
use crate::ui::widget::{Rect, Widget};

/// 垂直布局容器。
///
/// 子组件按添加顺序自上而下排列，`spacing` 控制间距。
/// 容器 bounds 随子组件自动扩展。
pub struct VBox {
    bounds: Rect,
    children: Vec<Box<dyn Widget>>,
    spacing: f32,
}

impl VBox {
    /// 创建空的垂直布局容器。
    pub fn new(spacing: f32) -> Self {
        Self {
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            children: Vec::new(),
            spacing,
        }
    }

    /// 设置容器位置，同时移动所有子组件。
    pub fn set_position(&mut self, x: f32, y: f32) {
        let dx = x - self.bounds.x;
        let dy = y - self.bounds.y;
        for child in &mut self.children {
            let b = child.bounds();
            child.set_position(b.x + dx, b.y + dy);
        }
        self.bounds.x = x;
        self.bounds.y = y;
    }

    /// 添加子组件，自动计算新组件的垂直位置。
    pub fn push(&mut self, mut child: impl Widget + 'static) {
        let child_x = self.bounds.x;
        let child_y = if let Some(last) = self.children.last() {
            let b = last.bounds();
            b.y + b.h + self.spacing
        } else {
            self.bounds.y
        };
        child.set_position(child_x, child_y);
        self.children.push(Box::new(child));
        self.recalc_bounds();
    }

    /// 根据子组件重新计算容器 bounds。
    fn recalc_bounds(&mut self) {
        if self.children.is_empty() {
            self.bounds.w = 0.0;
            self.bounds.h = 0.0;
            return;
        }

        let mut max_w = 0.0f32;
        let mut max_y = 0.0f32;

        for child in &self.children {
            let b = child.bounds();
            let right = b.x + b.w - self.bounds.x;
            let bottom = b.y + b.h - self.bounds.y;
            max_w = max_w.max(right);
            max_y = max_y.max(bottom);
        }

        self.bounds.w = max_w;
        self.bounds.h = max_y;
    }
}

impl Widget for VBox {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        let dx = x - self.bounds.x;
        let dy = y - self.bounds.y;
        for child in &mut self.children {
            let b = child.bounds();
            child.set_position(b.x + dx, b.y + dy);
        }
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn draw(&self, renderer: &mut Renderer2D) {
        for child in &self.children {
            child.draw(renderer);
        }
    }

    fn hit_test(&self, px: f32, py: f32) -> bool {
        // 先检查容器自身 bounds
        if self.bounds.contains(px, py) {
            return true;
        }
        // 也检查子组件（子组件可能超出容器 bounds？不会，但保留检查）
        for child in &self.children {
            if child.hit_test(px, py) {
                return true;
            }
        }
        false
    }

    fn on_mouse_down(&mut self, px: f32, py: f32) -> bool {
        for child in self.children.iter_mut().rev() {
            if child.hit_test(px, py) && child.on_mouse_down(px, py) {
                return true;
            }
        }
        false
    }

    fn on_mouse_up(&mut self, px: f32, py: f32) -> bool {
        for child in self.children.iter_mut().rev() {
            if child.hit_test(px, py) && child.on_mouse_up(px, py) {
                return true;
            }
        }
        false
    }
}
