#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    pub(crate) position: [f32; 2],
    pub(crate) tex_coords: [f32; 2],
    pub(crate) color: [f32; 4],
}

#[allow(unused)] pub const COLOR_BLUE:   [f32;4] = [0.0, 0.0, 1.0, 1.0];
#[allow(unused)] pub const COLOR_WHITE:  [f32;4] = [1.0, 1.0, 1.0, 1.0];
#[allow(unused)] pub const COLOR_BLACK:  [f32;4] = [0.0, 0.0, 0.0, 1.0];
#[allow(unused)] pub const COLOR_RED:    [f32;4] = [1.0, 0.0, 0.0, 1.0];
#[allow(unused)] pub const COLOR_GREEN:  [f32;4] = [0.0, 1.0, 0.0, 1.0];


impl TextVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
