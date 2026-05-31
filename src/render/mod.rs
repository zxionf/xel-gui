mod font_atlas;
pub mod renderer2d;
pub mod text_render;
mod text_vertex;
mod vertex;

use crate::render::renderer2d::Renderer2D;
use crate::render::text_render::TextRenderer;
use crate::ui::renderer::UIRenderer;

pub struct RenderContext {
    pub ren: Renderer2D,
    pub ui: UIRenderer,
    pub text: TextRenderer,
}

impl RenderContext {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
    ) -> Self {
        let ren = Renderer2D::new(device, config, queue);
        let ui = UIRenderer::new(device, config);
        let text = TextRenderer::new(device, config, queue);
        Self { ren, ui, text }
    }
}
