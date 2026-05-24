use std::mem;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex2D {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex2D {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    projection: [[f32; 4]; 4],
}

const SHADER_SRC: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color:    vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)   color:         vec4<f32>,
};

struct Uniforms {
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

const MAX_VERTICES: u64 = 16384;
const MAX_INDICES: u64 = 24576;

pub struct Renderer2D {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    vertices: Vec<Vertex2D>,
    indices: Vec<u32>,

    screen_width: f32,
    screen_height: f32,
    uploaded_index_count: u32,
}

impl Renderer2D {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        screen_width: u32,
        screen_height: u32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("xelgui-2d-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("xelgui-uniform-bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("xelgui-pipeline-layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("xelgui-2d-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex2D::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let uniforms = Self::build_projection(screen_width, screen_height);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("xelgui-uniform"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("xelgui-uniform-bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("xelgui-vbuf"),
            size: MAX_VERTICES * mem::size_of::<Vertex2D>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("xelgui-ibuf"),
            size: MAX_INDICES * mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group,
            uniform_buffer,
            vertex_buffer,
            index_buffer,
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(1536),
            screen_width: screen_width as f32,
            screen_height: screen_height as f32,
            uploaded_index_count: 0,
        }
    }

    pub fn begin_frame(&mut self, screen_width: u32, screen_height: u32) {
        self.screen_width = screen_width as f32;
        self.screen_height = screen_height as f32;
        self.vertices.clear();
        self.indices.clear();
        self.uploaded_index_count = 0;
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let base = self.vertices.len() as u32;

        self.vertices.push(Vertex2D {
            position: [x, y],
            color,
        });
        self.vertices.push(Vertex2D {
            position: [x + w, y],
            color,
        });
        self.vertices.push(Vertex2D {
            position: [x + w, y + h],
            color,
        });
        self.vertices.push(Vertex2D {
            position: [x, y + h],
            color,
        });

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// 描边矩形（空心边框）。用四条窄矩形拼成。
    pub fn stroke_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        thickness: f32,
        color: [f32; 4],
    ) {
        let t = thickness;
        self.fill_rect(x, y, w, t, color); // 上
        self.fill_rect(x, y + h - t, w, t, color); // 下
        self.fill_rect(x, y + t, t, h - 2.0 * t, color); // 左
        self.fill_rect(x + w - t, y + t, t, h - 2.0 * t, color); // 右
    }

    pub fn upload(&mut self, queue: &wgpu::Queue) {
        if self.vertices.is_empty() {
            return;
        }

        let uniforms =
            Self::build_projection(self.screen_width as u32, self.screen_height as u32);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let vdata = bytemuck::cast_slice(&self.vertices);
        let vlen = (vdata.len() as u64).min(self.vertex_buffer.size());
        queue.write_buffer(&self.vertex_buffer, 0, &vdata[..vlen as usize]);

        let idata = bytemuck::cast_slice(&self.indices);
        let ilen = (idata.len() as u64).min(self.index_buffer.size());
        queue.write_buffer(&self.index_buffer, 0, &idata[..ilen as usize]);

        self.uploaded_index_count = self.indices.len().min(MAX_INDICES as usize) as u32;
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.uploaded_index_count == 0 {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.uploaded_index_count, 0, 0..1);
    }

    fn build_projection(w: u32, h: u32) -> Uniforms {
        let proj = glam::Mat4::orthographic_rh_gl(0.0, w as f32, h as f32, 0.0, -1.0, 1.0);
        Uniforms {
            projection: proj.to_cols_array_2d(),
        }
    }
}
