use std::sync::Arc;

use log::trace;
use wgpu::Surface;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

use crate::scene::Scene;

/// Wrapper type which manages the surface and surface configuration.
///
/// As surface usage varies per platform, wrapping this up cleans up the event loop code.
struct SurfaceWrapper {
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
}

impl SurfaceWrapper {
    /// Create a new surface wrapper with no surface or configuration.
    fn new() -> Self {
        Self {
            surface: None,
            config: None,
        }
    }

    /// Called when an event which matches [`Self::start_condition`] is received.
    ///
    /// On all native platforms, this is where we create the surface.
    ///
    /// Additionally, we configure the surface based on the (now valid) window size.
    fn resume(&mut self, context: &ExampleContext, window: Arc<Window>, srgb: bool) {
        // Window size is only actually valid after we enter the event loop.
        let window_size = window.inner_size();
        let width = window_size.width.max(1);
        let height = window_size.height.max(1);

        log::info!("Surface resume {window_size:?}");

        // We didn't create the surface in pre_adapter, so we need to do so now.
        self.surface = Some(context.instance.create_surface(window).unwrap());

        // From here on, self.surface should be Some.

        let surface = self.surface.as_ref().unwrap();

        // Get the default configuration,
        let mut config = surface
            .get_default_config(&context.adapter, width, height)
            .expect("Surface isn't supported by the adapter.");
        if srgb {
            // Not all platforms (WebGPU) support sRGB swapchains, so we need to use view formats
            let view_format = config.format.add_srgb_suffix();
            config.view_formats.push(view_format);
        } else {
            // All platforms support non-sRGB swapchains, so we can just use the format directly.
            let format = config.format.remove_srgb_suffix();
            config.format = format;
            config.view_formats.push(format);
        };

        surface.configure(&context.device, &config);
        self.config = Some(config);
    }

    /// Resize the surface, making sure to not resize to zero.
    fn resize(&mut self, context: &ExampleContext, size: PhysicalSize<u32>) {
        log::info!("Surface resize {size:?}");

        let config = self.config.as_mut().unwrap();
        config.width = size.width.max(1);
        config.height = size.height.max(1);
        let surface = self.surface.as_ref().unwrap();
        surface.configure(&context.device, config);
    }

    /// Acquire the next surface texture.
    fn acquire(&mut self, context: &ExampleContext) -> wgpu::SurfaceTexture {
        let surface = self.surface.as_ref().unwrap();

        match surface.get_current_texture() {
            Ok(frame) => frame,
            // If we timed out, just try again
            Err(wgpu::SurfaceError::Timeout) => surface
                .get_current_texture()
                .expect("Failed to acquire next surface texture!"),
            Err(
                // If the surface is outdated, or was lost, reconfigure it.
                wgpu::SurfaceError::Outdated
                | wgpu::SurfaceError::Lost
                // If OutOfMemory happens, reconfiguring may not help, but we might as well try
                | wgpu::SurfaceError::OutOfMemory,
            ) => {
                surface.configure(&context.device, self.config());
                surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!")
            }
        }
    }

    /// On suspend on android, we drop the surface, as it's no longer valid.
    ///
    /// A suspend event is always followed by at least one resume event.
    fn suspend(&mut self) {
        if cfg!(target_os = "android") {
            self.surface = None;
        }
    }

    fn get(&self) -> Option<&Surface> {
        self.surface.as_ref()
    }

    fn config(&self) -> &wgpu::SurfaceConfiguration {
        self.config.as_ref().unwrap()
    }
}

/// Context containing global wgpu resources.
struct ExampleContext {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}
impl ExampleContext {
    /// Initializes the example context.
    async fn init_async(surface: &mut SurfaceWrapper) -> Self {
        log::info!("Initializing wgpu...");

        let backends = wgpu::util::backend_bits_from_env().unwrap_or_default();
        let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();
        let gles_minor_version = wgpu::util::gles_minor_version_from_env().unwrap_or_default();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            flags: wgpu::InstanceFlags::from_build_config().with_env(),
            dx12_shader_compiler,
            gles_minor_version,
        });
        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, surface.get())
            .await
            .expect("No suitable GPU adapters found on the system!");

        let adapter_info = adapter.get_info();
        log::info!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

        let optional_features = Scene::optional_features();
        let required_features = Scene::required_features();
        let adapter_features = adapter.features();
        assert!(
            adapter_features.contains(required_features),
            "Adapter does not support required features for this example: {:?}",
            required_features - adapter_features
        );

        let required_downlevel_capabilities = Scene::required_downlevel_capabilities();
        let downlevel_capabilities = adapter.get_downlevel_capabilities();
        assert!(
            downlevel_capabilities.shader_model >= required_downlevel_capabilities.shader_model,
            "Adapter does not support the minimum shader model required to run this example: {:?}",
            required_downlevel_capabilities.shader_model
        );
        assert!(
            downlevel_capabilities
                .flags
                .contains(required_downlevel_capabilities.flags),
            "Adapter does not support the downlevel capabilities required to run this example: {:?}",
            required_downlevel_capabilities.flags - downlevel_capabilities.flags
        );

        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the surface.
        let needed_limits = Scene::required_limits().using_resolution(adapter.limits());

        let trace_dir = std::env::var("WGPU_TRACE");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: (optional_features & adapter_features) | required_features,
                    required_limits: needed_limits,
                },
                trace_dir.ok().as_ref().map(std::path::Path::new),
            )
            .await
            .expect("Unable to find a suitable GPU adapter!");

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
}

struct Application {
    example: Option<Scene>,
    surface: SurfaceWrapper,
    context: ExampleContext,
    // window: Arc<Window>,
    window: Option<Arc<Window>>,
    title: String,
}

impl Application {
    async fn new(title: String) -> Self {
        let mut surface = SurfaceWrapper::new();
        let context = ExampleContext::init_async(&mut surface).await;

        Self {
            example: None,
            surface,
            context,
            window: None,
            title,
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default().with_title(&self.title);

        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        self.surface.resume(&self.context, window.clone(), true);

        self.window = Some(window);

        // If we haven't created the example yet, do so now.
        if self.example.is_none() {
            self.example.replace(Scene::init(
                self.surface.config(),
                &self.context.adapter,
                &self.context.device,
                &self.context.queue,
            ));
        }
    }

    // TODO maybe the trace output can be moved elsewhere?
    #[allow(clippy::too_many_lines)]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                trace!("WindowEvent::Resized({size:?})");

                self.surface.resize(&self.context, size);
                self.example.as_mut().unwrap().resize(
                    self.surface.config(),
                    &self.context.device,
                    &self.context.queue,
                );

                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::ActivationTokenDone { serial, token } => {
                trace!("WindowEvent::ActivationTokenDone({serial:?}, {token:?})");
            }
            WindowEvent::Moved(position) => {
                trace!("WindowEvent::Moved({position:?})");
            }
            WindowEvent::CloseRequested => {
                trace!("WindowEvent::CloseRequested()");
                event_loop.exit();
            }
            WindowEvent::Destroyed => {
                trace!("WindowEvent::Destroyed()");
            }
            WindowEvent::DroppedFile(path) => {
                trace!("WindowEvent::DroppedFile({path})", path = path.display());
            }
            WindowEvent::HoveredFile(path) => {
                trace!("WindowEvent::HoveredFile({path})", path = path.display());
            }
            WindowEvent::HoveredFileCancelled => {
                trace!("WindowEvent::HoveredFileCancelled()");
            }
            WindowEvent::Focused(focused) => {
                trace!("WindowEvent::Focused({focused})");
            }

            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                trace!("WindowEvent::KeyboardInput({device_id:?}, {event:?}, {is_synthetic})");
                let KeyEvent {
                    physical_key: _,
                    logical_key,
                    text: _,
                    location: _,
                    state: _,
                    repeat: _,
                    ..
                } = event;

                match logical_key {
                    Key::Named(key) => {
                        trace!("WindowEvent::KeyboardInput::logical_key::Named({key:?})");
                        // more branches will follow for sure …
                        #[allow(clippy::single_match)]
                        match key {
                            NamedKey::Escape => {
                                event_loop.exit();
                            }
                            _ => {}
                        }
                    }
                    Key::Character(key) => {
                        trace!("WindowEvent::KeyboardInput::logical_key::Character({key:?})");
                        if key == "r" {
                            println!("{:#?}", self.context.instance.generate_report());
                        }
                    }
                    Key::Unidentified(key) => {
                        trace!("WindowEvent::KeyboardInput::logical_key::Unidentified({key:?})");
                    }
                    Key::Dead(key) => {
                        trace!("WindowEvent::KeyboardInput::logical_key::Dead({key:?})");
                    }
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                trace!("WindowEvent::ModifiersChanged({modifiers:?})");
            }
            WindowEvent::Ime(ime) => {
                trace!("WindowEvent::Ime({ime:?})");
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                trace!("WindowEvent::CursorMoved({device_id:?}, {position:?})");
            }
            WindowEvent::CursorEntered { device_id } => {
                trace!("WindowEvent::CursorEntered({device_id:?})");
            }
            WindowEvent::CursorLeft { device_id } => {
                trace!("WindowEvent::CursorLeft({device_id:?})");
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                trace!("WindowEvent::MouseWheel({device_id:?}, {delta:?}, {phase:?})");
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                trace!("WindowEvent::MouseInput({device_id:?}, {state:?}, {button:?})");
            }
            WindowEvent::PinchGesture {
                device_id,
                delta,
                phase,
            } => {
                trace!("WindowEvent::PinchGesture({device_id:?}, {delta:?}, {phase:?})");
            }
            WindowEvent::PanGesture {
                device_id,
                delta,
                phase,
            } => {
                trace!("WindowEvent::PanGesture({device_id:?}, {delta:?}, {phase:?})");
            }
            WindowEvent::DoubleTapGesture { device_id } => {
                trace!("WindowEvent::DoubleTapGesture({device_id:?})");
            }
            WindowEvent::RotationGesture {
                device_id,
                delta,
                phase,
            } => {
                trace!("WindowEvent::RotationGesture({device_id:?}, {delta}, {phase:?})");
            }
            WindowEvent::TouchpadPressure {
                device_id,
                pressure,
                stage,
            } => {
                trace!("WindowEvent::TouchpadPressure({device_id:?}, {pressure}, {stage})");
            }
            WindowEvent::AxisMotion {
                device_id,
                axis,
                value,
            } => {
                trace!("WindowEvent::AxisMotion({device_id:?}, {axis}, {value})");
            }
            WindowEvent::Touch(touch) => {
                trace!("WindowEvent::Touch({touch:?})");
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer,
            } => {
                trace!("WindowEvent::ScaleFactorChanged({scale_factor}, {inner_size_writer:?})");
            }
            WindowEvent::ThemeChanged(theme) => {
                trace!("WindowEvent::ThemeChanged({theme:?})");
            }
            WindowEvent::Occluded(occluded) => {
                trace!("WindowEvent::Occluded({occluded})");
            }
            WindowEvent::RedrawRequested => {
                // On MacOS, currently redraw requested comes in _before_ Init does.
                // If this happens, just drop the requested redraw on the floor.
                //
                // See https://github.com/rust-windowing/winit/issues/3235 for some discussion
                if self.example.is_none() {
                    return;
                }

                let frame = self.surface.acquire(&self.context);
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                    format: Some(self.surface.config().view_formats[0]),
                    ..wgpu::TextureViewDescriptor::default()
                });

                self.example.as_mut().unwrap().render(
                    &view,
                    &self.context.device,
                    &self.context.queue,
                );

                frame.present();

                self.window.as_ref().unwrap().request_redraw();
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::Added => {
                trace!("DeviceEvent::Added");
            }
            DeviceEvent::Removed => {
                trace!("DeviceEvent::Removed");
            }
            DeviceEvent::MouseMotion {
                delta: (_delta_x, _delta_y),
            } => {
                // these are super-noisy
                // trace!("DeviceEvent::MouseMotion({delta_x}, {delta_y})");
            }
            DeviceEvent::MouseWheel { delta } => {
                trace!("DeviceEvent::MouseWheel({delta:?})");
            }
            DeviceEvent::Motion { axis: _, value: _ } => {
                // these are super-noisy
                // trace!("DeviceEvent::Motion({axis}, {value})");
            }
            DeviceEvent::Button { button, state } => {
                trace!("DeviceEvent::Button({button}, {state:?})");
            }
            DeviceEvent::Key(key) => {
                trace!("DeviceEvent::Key({key:?})");
            }
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        trace!("event loop is exiting");
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        trace!("event loop was suspended");
        self.surface.suspend();
    }
}

pub(crate) async fn start(title: String) {
    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    // event_loop.set_control_flow(ControlFlow::Wait);

    let app = Application::new(title);
    log::info!("Entering event loop...");
    event_loop.run_app(&mut app.await).unwrap();
}
