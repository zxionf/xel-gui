pub mod render;
pub mod ui;

// 重新导出渲染后端，外部项目无需单独依赖 wgpu / winit
pub use wgpu;
pub use winit;

// 重新导出常用公共类型
pub use render::Renderer2D;
pub use ui::button::Button;
pub use ui::color::Color;
pub use ui::widget::{Rect, Widget};
pub use ui::UiRoot;
