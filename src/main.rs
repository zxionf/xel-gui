use std::{process::exit, sync::Arc};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use xelgui::{Button, Color, Label, Renderer2D, UiRoot, VBox};

/// 默认中文字体（思源黑体 CN）。同时覆盖 ASCII / 拉丁字符。
const DEFAULT_FONT_PATH: &str =
    "/usr/share/fonts/adobe-source-han-sans/SourceHanSansCN-Normal.otf";

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    #[allow(dead_code)]
    window: Arc<Window>,

    renderer: Renderer2D,
    ui: UiRoot,
    mouse_pos: (f32, f32),
}

impl State {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();
        println!("[winit][窗口]大小:{}x{}", size.width, size.height);
        let window = Arc::new(window);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("创建 Surface 失败");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("找不到合适的 GPU 适配器");

        let adapter_info = adapter.get_info();
        println!(
            "[GPU] 名称: {}, 厂商: {}, 设备: {:?}, 后端: {:?}",
            adapter_info.name,
            adapter_info.vendor,
            adapter_info.device_type,
            adapter_info.backend
        );

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("创建 Device 失败");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        println!(
            "[Surface] 选用格式: {:?}, 显示模式: {:?}",
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

        // 加载字体
        let font_data = std::fs::read(DEFAULT_FONT_PATH).unwrap_or_else(|e| {
            eprintln!(
                "[字体] 无法从 {} 读取字体: {e}。将使用 A rial 替代。",
                DEFAULT_FONT_PATH
            );
            // 尝试其他常见路径
            let fallbacks = [
                "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
                "/usr/share/fonts/TTF/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            ];
            for path in fallbacks {
                if let Ok(data) = std::fs::read(path) {
                    println!("[字体] 使用: {path}");
                    return data;
                }
            }
            panic!("找不到任何可用字体文件");
        });

        let mut renderer = Renderer2D::new(
            &device,
            surface_format,
            size.width,
            size.height,
            &font_data,
        );

        let mut ui = UiRoot::new();
        ui.set_debug(true);
        renderer.set_debug(true);

        // VBox 垂直布局：标题 + 按钮 + 说明文字
        let mut vbox = VBox::new(12.0);
        vbox.push(Label::new(0.0, 0.0, "Hello, XelGUI!", 32.0).with_color(Color::WHITE));
        vbox.push(Label::new(0.0, 0.0, "中文标题", 24.0).with_color(Color::new(0.3, 0.8, 0.5, 1.0)));
        vbox.push(Label::new(0.0, 0.0, "Press the button below", 18.0).with_color(Color::new(0.7, 0.7, 0.7, 1.0)));
        vbox.push(Button::new(
            0.0, 0.0, 200.0, 60.0, "Click me!",
            || {
                println!("[UI] 按钮被点击！");
                true
            },
        ));
        vbox.set_position(300.0, 150.0);
        ui.add(vbox);

        // 右下角退出按钮（独立于 VBox）
        ui.add(Button::new(
            (window.inner_size().width - 200) as f32,
            (window.inner_size().height - 60) as f32,
            200.0,
            60.0,
            "Exit",
            || {
                exit(0);
            },
        ));

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            renderer,
            ui,
            mouse_pos: (0.0, 0.0),
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            println!(
                "[窗口] 尺寸改变: {}x{} -> {}x{}",
                self.size.width,
                self.size.height,
                new_size.width,
                new_size.height
            );
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) {
        self.window.pre_present_notify();

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(wgpu::SurfaceError::Lost) => {
                eprintln!("[Render] Surface 丢失");
                return;
            }
            Err(wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                match self.surface.get_current_texture() {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("[Render] Surface 重建后仍失败: {e:?}");
                        return;
                    }
                }
            }
            Err(wgpu::SurfaceError::Timeout) => {
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                eprintln!("[Render] GPU 内存不足");
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.renderer
            .begin_frame(self.size.width, self.size.height);
        self.ui.draw(&mut self.renderer);
        self.renderer.upload(&self.queue);

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("xelgui-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.13,
                            b: 0.17,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.renderer.draw(&mut pass);
        }

        self.queue.submit([encoder.finish()]);
        frame.present();
    }
}

struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window = event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("XelGUI — wgpu 窗口")
                        .with_inner_size(winit::dpi::LogicalSize::new(800, 600)),
                )
                .expect("创建窗口失败");

            let state = pollster::block_on(State::new(window));
            self.state = Some(state);
            self.state.as_ref().unwrap().window.request_redraw();
        }
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
                state.render();
            }

            WindowEvent::CursorMoved { position, .. } => {
                let px = position.x as f32;
                let py = position.y as f32;
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
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("创建 EventLoop 失败");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App { state: None };
    event_loop.run_app(&mut app).expect("EventLoop 运行失败");
}
