use std::mem;
use crate::render::text_vertex::{TextVertex, COLOR_GREEN};
use crate::texture::Texture;
use wgpu::util::DeviceExt;

#[allow(unused)]
pub struct TextRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    vertices: Vec<TextVertex>,
    indices: Vec<u16>,
    projection: glam::Mat4,
    proj_buffer: wgpu::Buffer,
    proj_bind_group_layout: wgpu::BindGroupLayout,
    proj_bind_group: wgpu::BindGroup,
    texture: Texture,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

const MAX_TEXT_VERTICES: u64 = 4096;
const MAX_TEXT_INDICES: u64 = 8192;

impl TextRenderer {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
    ) -> Self {
        let texture_bytes = include_bytes!("../res/font_e.png");
        let mut texture = Texture::from_bytes(device, queue, texture_bytes, "font_e.png").unwrap();
        texture.set_pixel_sampler(device);
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("text_texture_bind_group_layout"),
            });
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("text_texture_bind_group"),
        });

        let proj_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Text Proj BindGroup Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./text_shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Text Render Pipeline Layout"),
                bind_group_layouts: &[&proj_bind_group_layout,&texture_bind_group_layout],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[TextVertex::desc()],
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
                // cull_mode: Some(wgpu::Face::Back),// 背面剔除
                cull_mode: None,
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

        // Vertex buffer
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UI Vertex Buffer"),
            size: MAX_TEXT_VERTICES * mem::size_of::<TextVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Index buffer
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UI Index Buffer"),
            size: MAX_TEXT_INDICES * mem::size_of::<u16>() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let width = config.width;
        let height = config.height;
        let projection = TextRenderer::build_projection(width, height);

        let proj_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Text Proj Buffer"),
            contents: bytemuck::cast_slice(&[projection]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let proj_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Proj Bind Group"),
            layout: &proj_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: proj_buffer.as_entire_binding(),
            }],
        });

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            vertices: Vec::with_capacity(MAX_TEXT_VERTICES as usize),
            indices: Vec::with_capacity(MAX_TEXT_INDICES as usize),
            num_indices: 0,
            texture,
            texture_bind_group_layout,
            texture_bind_group,
            projection,
            proj_buffer,
            proj_bind_group_layout,
            proj_bind_group,
            width,
            height,
        }
    }

    pub fn begin_frame(&mut self, width: u32, height: u32) {
        self.vertices.clear();
        self.indices.clear();
        self.num_indices = 0;
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
        }
    }

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

    pub fn draw_texture(&mut self, x:f32,y:f32, w:f32, h:f32) {
        let base = self.vertices.len() as u16;
        self.vertices.push(TextVertex { position: [x, y], tex_coords: [0.0, 0.0], color:COLOR_GREEN, });
        self.vertices.push(TextVertex { position: [x, y + h], tex_coords: [0.0, 1.0], color:COLOR_GREEN, });
        self.vertices.push(TextVertex { position: [x + w, y + h], tex_coords: [1.0, 1.0], color:COLOR_GREEN, });
        self.vertices.push(TextVertex { position: [x + w, y], tex_coords: [1.0, 0.0], color:COLOR_GREEN, });

        self.indices.push(base);
        self.indices.push(base + 1);
        self.indices.push(base + 3);
        self.indices.push(base + 3);
        self.indices.push(base + 1);
        self.indices.push(base + 2);

        self.num_indices += 6;
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.num_indices == 0 {
            return;
        }
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.proj_bind_group, &[]);
        render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    fn build_projection(width:u32,height:u32) -> glam::Mat4 {
        let w = width as f32;
        let h = height as f32;
        return glam::Mat4::orthographic_rh(0.0, w, h, 0.0, -1.0, 1.0);
        // glam::Mat4::from_cols_array(&[
        //     2.0 / w, 0.0, 0.0, 0.0, // 列0
        //     0.0, -2.0 / h, 0.0, 0.0, // 列1
        //     0.0, 0.0, 1.0, 0.0, // 列2
        //     -1.0, 1.0, 0.0, 1.0, // 列3
        // ])
    }
}
