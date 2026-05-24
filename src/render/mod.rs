mod glyph_cache;

use std::mem;
use wgpu::util::DeviceExt;

use glyph_cache::GlyphCache;

// ── 纯色 Vertex ──────────────────────────────────────────────

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

// ── 纹理 Vertex ──────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextVertex {
    position: [f32; 2],
    texcoord: [f32; 2],
    color: [f32; 4],
}

impl TextVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// ── Uniform ──────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    projection: [[f32; 4]; 4],
}

// ── 常量 ─────────────────────────────────────────────────────

const SOLID_SHADER_SRC: &str = r#"
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

const TEXT_SHADER_SRC: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
    @location(2) color:    vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)   texcoord:      vec2<f32>,
    @location(1)   color:         vec4<f32>,
};

struct Uniforms {
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var tex: texture_2d<f32>;

@group(0) @binding(2)
var samp: sampler;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);
    out.texcoord = in.texcoord;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(tex, samp, in.texcoord);
    return vec4<f32>(in.color.rgb, in.color.a * tex_color.a);
}
"#;

const MAX_VERTICES: u64 = 16384;
const MAX_INDICES: u64 = 24576;
const ATLAS_SIZE: u32 = 1024;

// ── Renderer2D ───────────────────────────────────────────────

pub struct Renderer2D {
    // 纯色
    solid_pipeline: wgpu::RenderPipeline,
    solid_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    vertices: Vec<Vertex2D>,
    indices: Vec<u32>,
    uploaded_index_count: u32,

    // 纹理
    text_pipeline: wgpu::RenderPipeline,
    text_bind_group: wgpu::BindGroup,
    text_vertex_buffer: wgpu::Buffer,
    text_index_buffer: wgpu::Buffer,
    glyph_texture: wgpu::Texture,
    _glyph_sampler: wgpu::Sampler,

    text_vertices: Vec<TextVertex>,
    text_indices: Vec<u32>,
    uploaded_text_index_count: u32,

    glyph_cache: GlyphCache,

    /// 调试模式：为每个字符绘制边界框。
    debug: bool,

    screen_width: f32,
    screen_height: f32,
}

impl Renderer2D {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        screen_width: u32,
        screen_height: u32,
        font_data: &[u8],
    ) -> Self {
        // ── 纯色 pipeline ─────────────────────────────────

        let solid_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("xelgui-solid-shader"),
            source: wgpu::ShaderSource::Wgsl(SOLID_SHADER_SRC.into()),
        });

        let uniform_bgl =
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

        let solid_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("xelgui-solid-pipeline-layout"),
                bind_group_layouts: &[&uniform_bgl],
                push_constant_ranges: &[],
            });

        let solid_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("xelgui-solid-pipeline"),
                layout: Some(&solid_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &solid_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex2D::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &solid_shader,
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

        // ── 纹理 pipeline ─────────────────────────────────

        let text_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("xelgui-text-shader"),
            source: wgpu::ShaderSource::Wgsl(TEXT_SHADER_SRC.into()),
        });

        let text_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("xelgui-text-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let text_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("xelgui-text-pipeline-layout"),
                bind_group_layouts: &[&text_bgl],
                push_constant_ranges: &[],
            });

        let text_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("xelgui-text-pipeline"),
                layout: Some(&text_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &text_shader,
                    entry_point: "vs_main",
                    buffers: &[TextVertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &text_shader,
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

        // ── 共享资源 ─────────────────────────────────────

        let uniforms = Self::build_projection(screen_width, screen_height);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("xelgui-uniform"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let solid_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("xelgui-solid-bg"),
            layout: &uniform_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // 字形图集纹理
        let glyph_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("xelgui-glyph-atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let glyph_texture_view = glyph_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let glyph_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("xelgui-glyph-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let text_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("xelgui-text-bg"),
            layout: &text_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&glyph_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&glyph_sampler),
                },
            ],
        });

        // ── 缓冲区 ────────────────────────────────────────

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

        let text_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("xelgui-text-vbuf"),
            size: MAX_VERTICES * mem::size_of::<TextVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let text_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("xelgui-text-ibuf"),
            size: MAX_INDICES * mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let glyph_cache =
            GlyphCache::new(font_data, ATLAS_SIZE).expect("创建 GlyphCache 失败");

        Self {
            solid_pipeline,
            solid_bind_group,
            uniform_buffer,
            vertex_buffer,
            index_buffer,
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(1536),
            uploaded_index_count: 0,

            text_pipeline,
            text_bind_group,
            text_vertex_buffer,
            text_index_buffer,
            glyph_texture,
            _glyph_sampler: glyph_sampler,

            text_vertices: Vec::with_capacity(1024),
            text_indices: Vec::with_capacity(1536),
            uploaded_text_index_count: 0,

            glyph_cache,

            debug: false,

            screen_width: screen_width as f32,
            screen_height: screen_height as f32,
        }
    }

    // ── 帧控制 ────────────────────────────────────────────

    /// 开启/关闭调试模式。开启后每个文字字符会绘制品红色边框。
    pub fn set_debug(&mut self, enabled: bool) {
        self.debug = enabled;
    }

    pub fn begin_frame(&mut self, screen_width: u32, screen_height: u32) {
        self.screen_width = screen_width as f32;
        self.screen_height = screen_height as f32;
        self.vertices.clear();
        self.indices.clear();
        self.uploaded_index_count = 0;
        self.text_vertices.clear();
        self.text_indices.clear();
        self.uploaded_text_index_count = 0;
    }

    // ── 纯色绘制 ──────────────────────────────────────────

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

    // ── 文本绘制 ──────────────────────────────────────────

    /// 绘制文本。`(x, y)` 为基线起始位置，`px` 为像素字号。
    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, px: f32, color: [f32; 4]) {
        let atlas_w = ATLAS_SIZE as f32;
        let atlas_h = ATLAS_SIZE as f32;
        let mut pen_x = x;

        // 调试边框颜色（品红半透明）
        const DEBUG_GLYPH_BORDER: [f32; 4] = [1.0, 0.0, 1.0, 0.7];

        for ch in text.chars() {
            let glyph = match self.glyph_cache.get_or_rasterize(ch, px) {
                Some(g) => g.clone(),
                None => continue,
            };

            if glyph.width > 0 && glyph.height > 0 {
                let base = self.text_vertices.len() as u32;
                let gx = pen_x + glyph.xmin;
                let gy = y + glyph.ymin;
                let gw = glyph.width as f32;
                let gh = glyph.height as f32;

                let u0 = glyph.atlas_x as f32 / atlas_w;
                let v0 = glyph.atlas_y as f32 / atlas_h;
                let u1 = (glyph.atlas_x + glyph.width) as f32 / atlas_w;
                let v1 = (glyph.atlas_y + glyph.height) as f32 / atlas_h;

                self.text_vertices.push(TextVertex {
                    position: [gx, gy],
                    texcoord: [u0, v0],
                    color,
                });
                self.text_vertices.push(TextVertex {
                    position: [gx + gw, gy],
                    texcoord: [u1, v0],
                    color,
                });
                self.text_vertices.push(TextVertex {
                    position: [gx + gw, gy + gh],
                    texcoord: [u1, v1],
                    color,
                });
                self.text_vertices.push(TextVertex {
                    position: [gx, gy + gh],
                    texcoord: [u0, v1],
                    color,
                });

                self.text_indices.extend_from_slice(&[
                    base,
                    base + 1,
                    base + 2,
                    base,
                    base + 2,
                    base + 3,
                ]);

                // 调试：字符边界框
                if self.debug {
                    self.stroke_rect(gx, gy, gw, gh, 1.0, DEBUG_GLYPH_BORDER);
                }
            } else if self.debug {
                // 空格等空字符：画一条细竖线标记位置
                self.stroke_rect(pen_x, y - px * 0.2, 1.0, px * 0.4, 1.0, DEBUG_GLYPH_BORDER);
            }

            pen_x += glyph.advance_width;
        }
    }

    // ── 上传 ──────────────────────────────────────────────

    pub fn upload(&mut self, queue: &wgpu::Queue) {
        let uniforms =
            Self::build_projection(self.screen_width as u32, self.screen_height as u32);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // 纯色
        if !self.vertices.is_empty() {
            let vdata = bytemuck::cast_slice(&self.vertices);
            let vlen = (vdata.len() as u64).min(self.vertex_buffer.size());
            queue.write_buffer(&self.vertex_buffer, 0, &vdata[..vlen as usize]);

            let idata = bytemuck::cast_slice(&self.indices);
            let ilen = (idata.len() as u64).min(self.index_buffer.size());
            queue.write_buffer(&self.index_buffer, 0, &idata[..ilen as usize]);

            self.uploaded_index_count =
                self.indices.len().min(MAX_INDICES as usize) as u32;
        }

        // 纹理
        if !self.text_vertices.is_empty() {
            // 更新图集纹理（如果脏了）
            if self.glyph_cache.is_dirty() {
                let (data, aw, ah) = self.glyph_cache.atlas_texture_data();
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &self.glyph_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    data,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * aw),
                        rows_per_image: Some(ah),
                    },
                    wgpu::Extent3d {
                        width: aw,
                        height: ah,
                        depth_or_array_layers: 1,
                    },
                );
                self.glyph_cache.clear_dirty();
            }

            let vdata = bytemuck::cast_slice(&self.text_vertices);
            let vlen = (vdata.len() as u64).min(self.text_vertex_buffer.size());
            queue.write_buffer(&self.text_vertex_buffer, 0, &vdata[..vlen as usize]);

            let idata = bytemuck::cast_slice(&self.text_indices);
            let ilen = (idata.len() as u64).min(self.text_index_buffer.size());
            queue.write_buffer(&self.text_index_buffer, 0, &idata[..ilen as usize]);

            self.uploaded_text_index_count =
                self.text_indices.len().min(MAX_INDICES as usize) as u32;
        }
    }

    // ── 绘制 ──────────────────────────────────────────────

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // 纯色
        if self.uploaded_index_count > 0 {
            render_pass.set_pipeline(&self.solid_pipeline);
            render_pass.set_bind_group(0, &self.solid_bind_group, &[]);
            render_pass
                .set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..self.uploaded_index_count, 0, 0..1);
        }

        // 纹理
        if self.uploaded_text_index_count > 0 {
            render_pass.set_pipeline(&self.text_pipeline);
            render_pass.set_bind_group(0, &self.text_bind_group, &[]);
            render_pass
                .set_vertex_buffer(0, self.text_vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.text_index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..self.uploaded_text_index_count, 0, 0..1);
        }
    }

    // ── 工具 ──────────────────────────────────────────────

    fn build_projection(w: u32, h: u32) -> Uniforms {
        let proj = glam::Mat4::orthographic_rh_gl(0.0, w as f32, h as f32, 0.0, -1.0, 1.0);
        Uniforms {
            projection: proj.to_cols_array_2d(),
        }
    }
}
