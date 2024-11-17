#![expect(clippy::unwrap_used, clippy::expect_used, reason = "just demo code")]
//! Simple winit application.

use ::tracing::{debug, info};
use gam3du_framework::init_logger;
use std::error::Error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
// #[cfg(macos_platform)]
// use winit::platform::macos::{
//     ApplicationHandlerExtMacOS, OptionAsAlt, WindowAttributesExtMacOS, WindowExtMacOS,
// };
// #[cfg(any(x11_platform, wayland_platform))]
// use winit::platform::startup_notify::{
//     self, EventLoopExtStartupNotify, WindowAttributesExtStartupNotify, WindowExtStartupNotify,
// };
#[cfg(target_family = "wasm")]
use winit::platform::web::WindowAttributesExtWeb;
// use winit::platform::web::{ActiveEventLoopExtWeb, WindowAttributesExtWeb};
use winit::window::{Icon, Window, WindowAttributes, WindowId};

fn main() -> Result<(), Box<dyn Error>> {
    init_logger();
    info!("Logging start");

    debug!("Creating new event loop");
    let event_loop = EventLoop::new()?;
    let app = Application::new();

    Ok(event_loop.run_app(app)?)
}

/// Application state and event handling.
struct Application {
    /// Application icon.
    icon: Icon,
    window_state: Option<WindowState>,
    // /// Drawing context.
    // ///
    // /// With OpenGL it could be `EGLDisplay`.
    // context: Option<Context<DisplayHandle<'static>>>,
}

impl Application {
    fn new() -> Self {
        let icon = load_icon(include_bytes!("../icon.png"));

        Self {
            icon,
            window_state: None,
        }
    }
}

impl ApplicationHandler for Application {
    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window_state) = self.window_state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::ActivationTokenDone { token: _token, .. } => {
                // #[cfg(any(x11_platform, wayland_platform))]
                // {
                //     startup_notify::set_activation_token_env(_token);
                //     if let Err(err) = self.create_window(event_loop, None) {
                //         error!("Error creating new window: {err}");
                //     }
                // }
            }
            WindowEvent::SurfaceResized(size) => {
                window_state.resize(size);
            }
            WindowEvent::RedrawRequested => {
                window_state.draw();
                // if let Err(err) = window_state.draw() {
                //     error!("Error drawing window: {err}");
                // }
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                info!("Keyboard input {key_event:?}");
                if key_event.logical_key == Key::Named(NamedKey::Escape) {
                    event_loop.exit();
                }
            }
            WindowEvent::CloseRequested => {
                info!("Closing Window={window_id:?}");
                drop(self.window_state.take());
            }
            WindowEvent::Focused(_)
            | WindowEvent::ScaleFactorChanged { .. }
            | WindowEvent::ThemeChanged(_)
            | WindowEvent::Occluded(_)
            | WindowEvent::ModifiersChanged(_)
            | WindowEvent::MouseWheel { .. }
            | WindowEvent::PointerButton { .. }
            | WindowEvent::PointerLeft { .. }
            | WindowEvent::PointerMoved { .. }
            | WindowEvent::Ime(_)
            | WindowEvent::DoubleTapGesture { .. }
            | WindowEvent::PanGesture { .. }
            | WindowEvent::RotationGesture { .. }
            | WindowEvent::PinchGesture { .. }
            | WindowEvent::TouchpadPressure { .. }
            | WindowEvent::HoveredFileCancelled
            | WindowEvent::PointerEntered { .. }
            | WindowEvent::DroppedFile(_)
            | WindowEvent::HoveredFile(_)
            | WindowEvent::Destroyed
            | WindowEvent::Moved(_) => (),
        }
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        info!("Ready to create surfaces");

        // Create initial window.
        #[allow(unused_mut, reason = "required for target=wasm")]
        let mut window_attributes = WindowAttributes::default()
            .with_title("Demo window")
            .with_surface_size(PhysicalSize::new(800, 600))
            .with_window_icon(Some(self.icon.clone()));

        // #[cfg(any(x11_platform, wayland_platform))]
        // if let Some(token) = event_loop.read_token_from_env() {
        //     startup_notify::reset_activation_token_env();
        //     info!("Using token {:?} to activate a window", token);
        //     window_attributes = window_attributes.with_activation_token(token);
        // }

        #[cfg(target_family = "wasm")]
        {
            use wasm_bindgen::JsCast;
            // use winit::platform::web::WindowBuilderExtWebSys;
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();
            window_attributes = window_attributes.with_canvas(Some(canvas));

            // window_attributes = window_attributes.with_canvas(canvas).with_append(true);
        }

        let window = event_loop
            .create_window(window_attributes)
            .expect("failed to create initial window");

        let window_state =
            pollster::block_on(WindowState::new(window)).expect("failed to create initial window");

        let window_id = window_state.window.id();
        info!("Created new window with id={window_id:?}");
        self.window_state.replace(window_state);
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window_state.is_none() {
            info!("No windows left, exiting...");
            event_loop.exit();
        }
    }

    fn exiting(&mut self, _event_loop: &dyn ActiveEventLoop) {}
}

/// State of the window.
struct WindowState {
    /// The actual winit Window.
    window: Arc<dyn Window>,
    _instance: wgpu::Instance,
    /// Render surface.
    ///
    /// NOTE: This surface must be dropped before the `Window`.
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    // surface: Surface<DisplayHandle<'static>, Arc<dyn Window>>,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl WindowState {
    async fn new(window: Box<dyn Window>) -> Result<Self, Box<dyn Error>> {
        let window: Arc<dyn Window> = Arc::from(window);
        let size = window.surface_size();
        debug!("new window state; size: {size:?}");

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        // // Load the shaders from disk
        // let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        //     label: None,
        //     source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        // });

        // let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //     label: None,
        //     bind_group_layouts: &[],
        //     push_constant_ranges: &[],
        // });

        // let swapchain_capabilities = surface.get_capabilities(&adapter);
        // let swapchain_format = swapchain_capabilities.formats[0];

        // let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //     label: None,
        //     layout: Some(&pipeline_layout),
        //     vertex: wgpu::VertexState {
        //         module: &shader,
        //         entry_point: Some("vs_main"),
        //         buffers: &[],
        //         compilation_options: Default::default(),
        //     },
        //     fragment: Some(wgpu::FragmentState {
        //         module: &shader,
        //         entry_point: Some("fs_main"),
        //         compilation_options: Default::default(),
        //         targets: &[Some(swapchain_format.into())],
        //     }),
        //     primitive: wgpu::PrimitiveState::default(),
        //     depth_stencil: None,
        //     multisample: wgpu::MultisampleState::default(),
        //     multiview: None,
        //     cache: None,
        // });

        let surface_config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .unwrap();
        surface.configure(&device, &surface_config);

        let state = Self {
            window,
            _instance: instance,
            surface,
            surface_config,
            _adapter: adapter,
            device,
            queue,
        };

        // state.resize(size);
        Ok(state)
    }

    /// Resize the surface to the new size.
    fn resize(&mut self, size: PhysicalSize<u32>) {
        info!("Surface resized to {size:?}");

        if size.width == 0 || size.height == 0 {
            debug!("cannot resize to zero size");
            return;
        }

        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);

        self.window.request_redraw();
    }

    /// Draw the window contents.
    fn draw(&mut self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            // let mut rpass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.4,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            // rpass.set_pipeline(&self.render_pipeline);
            // rpass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));

        self.window.pre_present_notify();
        frame.present();
    }
}

fn load_icon(bytes: &[u8]) -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(bytes).unwrap().into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
