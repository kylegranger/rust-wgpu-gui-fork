use std::time::Duration;
use tracing::{error, info, warn};
cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use web_time::SystemTime;
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsValue;
    } else {
        use std::time::SystemTime;
    }
}
use winit::{
    dpi::PhysicalPosition,
    event::{
        Event::{self, UserEvent},
        WindowEvent,
    },
    event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
    keyboard::{Key, NamedKey},
    window::{Window, WindowBuilder},
};

// #[repr(C)]
// #[derive(Clone, Debug, Copy, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct Vertex {
//     position: [f32; 3],
//     color: [f32; 3],
//     rect: [f32; 4],
//     border_color: [f32; 3],
// }

pub struct Id(usize);

enum GUIEvent {
    SuccessEvent(Id),
}

struct State<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
}

impl<'window> State<'window> {
    async fn new(window: Window, event_loop_proxy: EventLoopProxy<GUIEvent>) -> State<'window> {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let size = winit::dpi::PhysicalSize::new(500, 500);
                let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::GL,
                    ..Default::default()
                });
                let limits = wgpu::Limits::downlevel_webgl2_defaults();
            } else {
                let size = window.inner_size();
                let instance = wgpu::Instance::default();
                let limits = wgpu::Limits::default();
            }
        }

        let mouse_coords = PhysicalPosition { x: 0.0, y: 0.0 };

        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window).unwrap())
        }
        .expect("can create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("can create adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                    required_features: wgpu::Features::empty(),
                    required_limits: limits,
                    label: None,
                },
                None,
            )
            .await
            .expect("can create a new device");

        // let config = surface
        //     .get_default_config(&adapter, size.width, size.height)
        //     .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            // present_mode: surface_caps.present_modes[0],
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        // let mut font_system =
        //     FontSystem::new_with_locale_and_db("en-US".into(), glyphon::fontdb::Database::new());
        // let font = include_bytes!("./fonts/font.ttf");
        // let emoji = include_bytes!("./fonts/emoji.ttf");
        // font_system.db_mut().load_font_data(font.to_vec());
        // font_system.db_mut().load_font_data(emoji.to_vec());

        // let text_cache = SwashCache::new();
        // let cache = Cache::new(&device);
        // let viewport = Viewport::new(&device, &cache);
        // let mut text_atlas = TextAtlas::new(&device, &queue, &cache, config.format);
        // let text_renderer = TextRenderer::new(
        //     &mut text_atlas,
        //     &device,
        //     wgpu::MultisampleState::default(),
        //     None,
        // );

        // let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        //     label: None,
        //     source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        // });

        // let render_pipeline_layout =
        //     device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //         label: None,
        //         bind_group_layouts: &[],
        //         push_constant_ranges: &[],
        //     });

        // let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //     label: None,
        //     layout: Some(&render_pipeline_layout),
        //     vertex: wgpu::VertexState {
        //         module: &shader,
        //         entry_point: Some("vertex"),
        //         buffers: &[Vertex::desc()],
        //         compilation_options: wgpu::PipelineCompilationOptions::default(),
        //     },
        //     fragment: Some(wgpu::FragmentState {
        //         module: &shader,
        //         entry_point: Some("fragment"),
        //         targets: &[Some(wgpu::ColorTargetState {
        //             format: config.format,
        //             blend: Some(wgpu::BlendState::REPLACE),
        //             write_mask: wgpu::ColorWrites::ALL,
        //         })],
        //         compilation_options: wgpu::PipelineCompilationOptions::default(),
        //     }),
        //     primitive: wgpu::PrimitiveState {
        //         topology: wgpu::PrimitiveTopology::TriangleList,
        //         strip_index_format: None,
        //         front_face: wgpu::FrontFace::Ccw,
        //         cull_mode: Some(wgpu::Face::Back),
        //         unclipped_depth: false,
        //         polygon_mode: wgpu::PolygonMode::Fill,
        //         conservative: false,
        //     },
        //     multisample: wgpu::MultisampleState::default(),
        //     depth_stencil: None,
        //     multiview: None,
        //     cache: None,
        // });

        let events_proxy_clone = event_loop_proxy.clone();
        // let button = button::Button::new(
        //     button::ButtonConfig {
        //         rect_pos: RectPos {
        //             top: 125,
        //             left: 100,
        //             bottom: 225,
        //             right: 400,
        //         },
        //         fill_color: [0.5, 0.0, 0.5],
        //         fill_color_active: [1.0, 0.0, 1.0],
        //         border_color: [0.0, 0.0, 0.0],
        //         border_color_active: [0.5, 0.5, 0.5],
        //         text: "Submit 🚀",
        //         text_color: Color::rgb(200, 200, 200),
        //         text_color_active: Color::rgb(255, 255, 255),
        //         on_click: Box::new(move || {
        //             let _ = events_proxy_clone.send_event(GUIEvent::SuccessEvent(Id(1)));
        //         }),
        //     },
        //     &mut font_system,
        // );

        // let text_field = text_field::TextField::new(
        //     text_field::TextFieldConfig {
        //         rect_pos: RectPos {
        //             top: 50,
        //             left: 100,
        //             bottom: 120,
        //             right: 400,
        //         },
        //         fill_color: [0.9, 0.9, 0.9],
        //         fill_color_active: [1.0, 1.0, 1.0],
        //         border_color: [0.3, 0.3, 0.3],
        //         border_color_active: [0.1, 0.1, 0.1],
        //         text_color: Color::rgb(10, 10, 10),
        //     },
        //     &mut font_system,
        // );

        // let components = vec![
        //     Component::Button(Id(0), button),
        //     Component::TextField(Id(1), text_field),
        // ];

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn handle_click(&mut self) {}

    fn input(&mut self, event: &WindowEvent, elwt: &EventLoopWindowTarget<GUIEvent>) -> bool {
        false
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

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
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            //     label: None,
            //     color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            //         view: &view,
            //         resolve_target: None,
            //         ops: wgpu::Operations {
            //             load: wgpu::LoadOp::Clear(wgpu::Color {
            //                 r: 1.0,
            //                 g: 0.0,
            //                 b: 0.3,
            //                 a: 1.0,
            //             }),
            //             store: wgpu::StoreOp::Store,
            //         },
            //     })],
            //     depth_stencil_attachment: None,
            //     timestamp_writes: None,
            //     occlusion_query_set: None,
            // });

            //     render_pass.set_pipeline(&self.render_pipeline);
            //     render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            //     render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            //     render_pass.draw_indexed(0..num_indices, 0, 0..1);

            //     self.text_renderer
            //         .render(&self.text_atlas, &self.viewport, &mut render_pass)
            //         .unwrap();
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            // std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            // console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            console_error_panic_hook::set_once();
            tracing_wasm::set_as_global_default();
        } else {
            tracing_subscriber::fmt::init();
        }
    }

    info!("sample info log");
    warn!("sample warn log");
    error!("sample error log");

    let event_loop = EventLoopBuilder::<GUIEvent>::with_user_event()
        .build()
        .unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let canvas = web_sys::Element::from(window.canvas().unwrap());
                doc.get_element_by_id("app")?.append_child(&canvas).ok()?;
                Some(())
            })
            .unwrap();
        let window = web_sys::window().unwrap();
        info!("window: {:?}", window);
        let location = window.location();
        info!("location: {:?}", location);
        let href = location.href().unwrap();
        info!("href: {:?}", href);
        // let window = window.into_serde();
        // info!("window SERDE: {:?}", window);
        // let navigator = window.navigator();
        // let example = serde_wasm_bindgen::from_value(navigator).unwrap();
        // info!("example: {:?}", example);
        // info!("navigator: {:?}", navigator);
        // let navigator: web_sys::Navigator = navigator.into_serde().unwrap();
        // info!("navigator: {:?}", navigator);
        // let do_not_track = navigator.do_not_track();
        // let test_value: String = do_not_track.into_serde().unwrap();
        // info!("do_not_track: {:?}", do_not_track.as_string());

        let document = window.document().expect("should have a document on window");
        info!("document: {:?}", document);
        let body = document.body().expect("document should have a body");
        info!("body: {:?}", body);
    }
    run_app(event_loop, window).await;
}

fn handle_success_event(state: &mut State, ev: &GUIEvent) {}

async fn run_app(event_loop: EventLoop<GUIEvent>, window: Window) {
    let mut state = State::new(window, event_loop.create_proxy()).await;

    let mut then = SystemTime::now();
    let mut now = SystemTime::now();
    let mut fps = 0;

    event_loop
        .run(move |event, elwt| match event {
            UserEvent(ev) => handle_success_event(&mut state, &ev),
            Event::WindowEvent { window_id, event }
                if window_id == state.window().id() && !state.input(&event, elwt) =>
            {
                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        match state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                            Err(e) => error!("render error: {e:?}"),
                        }

                        fps += 1;
                        if now.duration_since(then).unwrap() > Duration::from_secs(1) {
                            state.window().set_title(&format!("FPS: {}", fps));
                            fps = 0;
                            then = now;
                        }
                        now = SystemTime::now();
                    }
                    _ => (),
                };
            }
            Event::AboutToWait => {
                state.window.request_redraw();
            }
            _ => (),
        })
        .expect("event loop runs");
}
