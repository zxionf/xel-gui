use crate::render::Renderer2D;

/// 矩形区域（像素坐标，原点左上角）
#[derive(Copy, Clone, Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// 点是否在矩形内。
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }
}

/// 所有 UI 组件的抽象 trait。
///
/// 方法都是 object-safe 的，可以通过 `&dyn Widget` 调用。
pub trait Widget {
    /// 返回组件当前的包围矩形。
    fn bounds(&self) -> Rect;

    /// 设置组件位置（左上角）。
    fn set_position(&mut self, x: f32, y: f32);

    /// 将绘制命令提交给渲染器。
    fn draw(&self, renderer: &mut Renderer2D);

    /// 点是否命中组件（用于鼠标事件分发）。
    fn hit_test(&self, px: f32, py: f32) -> bool {
        self.bounds().contains(px, py)
    }

    /// 鼠标按下事件。返回 `true` 表示事件已消费。
    fn on_mouse_down(&mut self, _px: f32, _py: f32) -> bool {
        false
    }

    /// 鼠标释放事件。
    fn on_mouse_up(&mut self, _px: f32, _py: f32) -> bool {
        false
    }

    /// 鼠标进入组件区域。
    fn on_mouse_enter(&mut self) {}

    /// 鼠标离开组件区域。
    fn on_mouse_leave(&mut self) {}
}
