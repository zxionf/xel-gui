use std::sync::Arc;
use wgpu::{ExperimentalFeatures, MemoryHints, SurfaceError};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::EventLoopError,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use xelgui::{camera::controller::CameraController, ui::widget::boxs::Vbox};
use xelgui::ui::UIRoot;
use xelgui::ui::debug::UIDebugFlag;
use xelgui::ui::widget::rectangle::Rectangle;
use xelgui::render::RenderContext;

/// 默认中文字体（思源黑体 CN）。同时覆盖 ASCII / 拉丁字符。
// const DEFAULT_FONT_PATH: &str = "/usr/share/fonts/adobe-source-han-sans/SourceHanSansCN-Normal.otf";

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    #[allow(dead_code)]
    window: Arc<Window>,
    size: PhysicalSize<u32>,
    ui: UIRoot,
    render_context: RenderContext,
    camera_controller: CameraController,
    mouse_pos: (i32, i32),
}

#[derive(Default)]
pub struct App {
    state: Option<State>,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        println!("[winit][window] 大小:{}x{}", size.width, size.height);

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("[err][wgpu] 创建 Surface 失败");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("[err][wgpu] 找不到合适的 GPU 适配器");

        let adapter_info = adapter.get_info();
        println!(
            "[wgpu][Adapter] 名称: {}, 厂商: {}, 设备: {:?}, 后端: {:?}",
            adapter_info.name, adapter_info.vendor, adapter_info.device_type, adapter_info.backend
        );

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: ExperimentalFeatures::default(),
                memory_hints: MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("[err][egpu] Device 创建失败");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        println!(
            "[wgpu][Surface] 选用格式: {:?}, 显示模式: {:?}",
            surface_format, surface_caps.present_modes[0]
        );

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let camera_controller = CameraController::new(0.2);

        let mut ui = UIRoot::new();
        ui.set_debug(UIDebugFlag::DEBUG_WIDGETS | UIDebugFlag::DEBUG_EVENTS);

        let render_context = RenderContext::new(&device, &config, &queue);

        let mut vbox = Box::new(Vbox::new(20, 20, 30, 80));
        vbox.add(Box::new(Rectangle::new(
            50,
            50,
            200,
            100,
            [0.8, 0.2, 0.2, 0.9],
        )));
        ui.add(vbox);

        // 添加测试矩形
        ui.add(Box::new(Rectangle::new(
            50,
            50,
            200,
            100,
            [0.8, 0.2, 0.2, 0.9],
        )));
        ui.add(Box::new(Rectangle::new(
            100,
            200,
            300,
            60,
            [0.2, 0.6, 0.2, 0.7],
        )));
        ui.add(Box::new(Rectangle::new(
            20,
            350,
            150,
            150,
            [0.2, 0.3, 0.9, 0.8],
        )));

        // 加载字体
        // let font_data = std::fs::read(DEFAULT_FONT_PATH).unwrap_or_else(|e| {
        //     eprintln!(
        //         "[字体] 无法从 {} 读取字体: {e}。将使用 A rial 替代。",
        //         DEFAULT_FONT_PATH
        //     );
        //     // 尝试其他常见路径
        //     let fallbacks = [
        //         "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        //         "/usr/share/fonts/TTF/DejaVuSans.ttf",
        //         "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        //     ];
        //     for path in fallbacks {
        //         if let Ok(data) = std::fs::read(path) {
        //             println!("[字体] 使用: {path}");
        //             return data;
        //         }
        //     }
        //     panic!("找不到任何可用字体文件");
        // });

        Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: true,
            window,
            size,
            ui,
            render_context,
            camera_controller,
            mouse_pos: (0, 0),
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            println!(
                "[窗口] 尺寸改变: {}x{} -> {}x{}",
                self.size.width, self.size.height, new_size.width, new_size.height
            );
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {
                self.camera_controller.handle_key(code, is_pressed);
            }
        }
    }

    fn update(&mut self) {
        self.camera_controller
            .update_camera(&mut self.render_context.ren.camera.camera);
        self.render_context.ren
            .camera
            .camera_uniform
            .update_view_proj(&self.render_context.ren.camera.camera);
        self.queue.write_buffer(
            &self.render_context.ren.camera.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.render_context.ren.camera.camera_uniform]),
        );
    }

    pub fn render(&mut self) {
        self.window.pre_present_notify();

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(SurfaceError::Lost) => {
                eprintln!("[err][wgpu] Surface丢失");
                return;
            }
            Err(SurfaceError::Outdated) => {
                eprintln!("[err][wgpu] Surface已过时");
                self.surface.configure(&self.device, &self.config);
                match self.surface.get_current_texture() {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("[err][wgpu] Surface 重建后仍失败: {e:?}");
                        return;
                    }
                }
            }
            Err(SurfaceError::Timeout) => {
                eprintln!("[err][wgpu] 获取帧超时");
                return;
            }
            Err(SurfaceError::OutOfMemory) => {
                eprintln!("[err][wgpu] 显存不足");
                return;
            }
            Err(SurfaceError::Other) => {
                eprintln!("[err][wgpu] 其他错误: 未知错误");
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // --- UI 准备：收集顶点 ---
        self.render_context.ui.begin_frame(self.size.width, self.size.height);
        self.ui.draw(&mut self.render_context);
        self.render_context.ui.upload(&self.queue);

        self.render_context.text.begin_frame(self.size.width, self.size.height);
        self.render_context.text.draw_texture(200.0, 200.0, 600.0, 600.0);
        self.render_context.text.draw_text(100.0, 100.0, 64.0, 128.0, "zxionf", [0.0, 0.0, 0.0, 1.0]);
        self.render_context.text.upload(&self.queue);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // 3D 场景
            self.render_context.ren.draw(&mut render_pass);
            // UI 层（叠加在 3D 之上）
            // self.ui.renderer.draw(&mut render_pass);
            self.render_context.text.draw(&mut render_pass);
        }

        self.queue.submit([encoder.finish()]);
        frame.present();

        self.window.request_redraw();
    }
}

impl App {
    pub fn run() -> Result<(), EventLoopError> {
        env_logger::init();
        let event_loop: EventLoop<State> = EventLoop::<State>::with_user_event().build()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = App::default();
        event_loop.run_app(&mut app)
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attributes = Window::default_attributes();
            let window = Arc::new(
                event_loop
                    .create_window(window_attributes.with_title("xel-gui test"))
                    .unwrap(),
            );
            self.state = Some(pollster::block_on(State::new(window)));
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: State) {
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                println!("[Event] 关闭窗口");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                state.resize(new_size);
                state.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                state.update();
                state.render();
            }

            WindowEvent::CursorMoved { position, .. } => {
                let px = position.x as i32;
                let py = position.y as i32;
                state.mouse_pos = (px, py);
                state.ui.handle_mouse_move(px, py);
                state.window.request_redraw();
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                let (px, py) = state.mouse_pos;
                state.ui.handle_mouse_down(px, py);
                state.window.request_redraw();
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                let (px, py) = state.mouse_pos;
                state.ui.handle_mouse_up(px, py);
                state.window.request_redraw();
            }
            #[allow(unused)]
            // TODO MORE
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                let (px, py) = state.mouse_pos;
                state.ui.handle_mouse_move(px, py);
                state.window.request_redraw();

                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };

                // if dy != 0.0 {
                //     // 用 dy 的正负增加或减少缩放系数
                //     state.zoom = (state.zoom + dy * 0.1).clamp(0.1, 10.0);
                // }
                state.ui.handle_mouse_wheel(dx, dy);

                // 请求重绘以反映变化
                state.window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }
}
