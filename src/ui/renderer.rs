use std::mem;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UIVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl UIVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<UIVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// 每帧最大 UI 顶点/索引数
const MAX_UI_VERTICES: u64 = 4096;
const MAX_UI_INDICES: u64 = 8192;

/// UI 专用 2D 渲染管线。
///
/// 使用正交投影将像素坐标映射到 NDC，支持批量矩形绘制。
/// 在同一个 render pass 中，应在 3D 管线之后调用 `draw()`，
/// 通过 alpha 混合叠加到场景之上。
pub struct UIRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertices: Vec<UIVertex>,
    indices: Vec<u16>,
    num_indices: u32,
    projection: glam::Mat4,
    proj_buffer: wgpu::Buffer,
    #[allow(dead_code)]
    proj_bind_group_layout: wgpu::BindGroupLayout,
    proj_bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

impl UIRenderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let proj_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("UI Proj BindGroup Layout"),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&proj_bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[UIVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // UI 无需背面剔除；NDC y 翻转可能导致正面被误剔
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let width = config.width;
        let height = config.height;
        let projection = Self::build_projection(width, height);

        let proj_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Proj Buffer"),
            contents: bytemuck::cast_slice(&[projection]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let proj_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &proj_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: proj_buffer.as_entire_binding(),
            }],
            label: Some("UI Proj BindGroup"),
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UI Vertex Buffer"),
            size: MAX_UI_VERTICES * mem::size_of::<UIVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UI Index Buffer"),
            size: MAX_UI_INDICES * mem::size_of::<u16>() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            vertices: Vec::with_capacity(MAX_UI_VERTICES as usize),
            indices: Vec::with_capacity(MAX_UI_INDICES as usize),
            num_indices: 0,
            projection,
            proj_buffer,
            proj_bind_group_layout,
            proj_bind_group,
            width,
            height,
        }
    }

    /// 每帧开始时调用，清空顶点缓冲。
    /// 若窗口尺寸变化则更新正交投影矩阵。
    pub fn begin_frame(&mut self, width: u32, height: u32) {
        self.vertices.clear();
        self.indices.clear();
        self.num_indices = 0;

        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
        }
    }

    /// 向批处理缓冲区追加一个纯色矩形。
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let base = self.vertices.len() as u16;
        self.vertices.push(UIVertex {
            position: [x, y],
            color,
        });
        self.vertices.push(UIVertex {
            position: [x + w, y],
            color,
        });
        self.vertices.push(UIVertex {
            position: [x + w, y + h],
            color,
        });
        self.vertices.push(UIVertex {
            position: [x, y + h],
            color,
        });

        // 两个三角形，逆时针（CCW 正面）
        self.indices.push(base);
        self.indices.push(base + 1);
        self.indices.push(base + 3);
        self.indices.push(base + 1);
        self.indices.push(base + 2);
        self.indices.push(base + 3);

        self.num_indices += 6;
    }

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
        self.fill_rect(x, y, w, t, color);
        self.fill_rect(x, y + h - t, w, t, color);
        self.fill_rect(x, y + t, t, h - 2.0 * t, color);
        self.fill_rect(x + w - t, y + t, t, h - 2.0 * t, color);
    }

    /// 将 CPU 端积累的顶点/索引数据上传到 GPU 缓冲区，
    /// 并更新投影矩阵。
    pub fn upload(&mut self, queue: &wgpu::Queue) {
        if self.vertices.is_empty() {
            return;
        }

        // 更新投影矩阵
        self.projection = Self::build_projection(self.width, self.height);
        queue.write_buffer(
            &self.proj_buffer,
            0,
            bytemuck::cast_slice(&[self.projection]),
        );

        // 上传顶点
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));

        // 上传索引
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));
    }

    /// 在 render pass 中提交 UI 绘制命令。
    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.num_indices == 0 {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.proj_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    /// 构造正交投影矩阵。
    ///
    /// 将像素坐标 (0,0)左上 → (W,H)右下 映射到 NDC (-1,1)左上 → (1,-1)右下。
    fn build_projection(width: u32, height: u32) -> glam::Mat4 {
        let w = width as f32;
        let h = height as f32;
        glam::Mat4::from_cols_array(&[
            2.0 / w,
            0.0,
            0.0,
            0.0, // 列0
            0.0,
            -2.0 / h,
            0.0,
            0.0, // 列1
            0.0,
            0.0,
            1.0,
            0.0, // 列2
            -1.0,
            1.0,
            0.0,
            1.0, // 列3
        ])
    }
}
