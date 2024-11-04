use crate::renderer::{self, RendererBuilder};
use log::{debug, info, trace};
use std::sync::Arc;
use wgpu::PresentMode;
use winit::{dpi::PhysicalSize, window::Window};

pub(crate) struct RenderSurface<Renderer: renderer::Renderer> {
    window: Arc<dyn Window>,
    /// the overall WGPU environment which is responsible for the resource and configuration management
    #[expect(
        unused,
        reason = "this might come in handy later and doesn't do any harm"
    )]
    instance: wgpu::Instance,
    /// the specific surface of our main window where we want WGPU to draw all content
    surface: wgpu::Surface<'static>,
    /// the physical WGPU-adapter performing the actual workload of drawing on the surface
    #[expect(
        unused,
        reason = "this might come in handy later and doesn't do any harm"
    )]
    adapter: wgpu::Adapter,
    /// the current configuration of the shown surface
    /// this may change over time (e.g. for resizing)
    config: wgpu::SurfaceConfiguration,
    /// the logical device used to render on the surface
    device: wgpu::Device,
    /// the the command queue where to schedule the workload
    queue: wgpu::Queue,
    renderer: Renderer,
}

impl<Renderer: renderer::Renderer> RenderSurface<Renderer> {
    /// Create a new render surface for the given window backed by properly set up wgpu-managed resources.
    pub(crate) async fn new(
        window: Box<dyn Window>,
        renderer_builder: impl RendererBuilder<Renderer = Renderer>,
    ) -> Self {
        info!("Creating new render surface");
        // make the window available to multiple parties
        let window: Arc<dyn Window> = Arc::from(window);

        // caution: the window size can be (0, 0) as the resizing seems to occur later on some platforms
        let surface_size = window.surface_size();
        debug!("window size: {surface_size:?}");

        debug!("creating new wgpu instance");
        // all default but with some extra-debugging enabled
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::advanced_debugging(),
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        };
        let instance = wgpu::Instance::new(instance_descriptor);

        debug!("create wgpu surface for window");
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        debug!("get an adapter responsible for drawing on the surface");
        let request_adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        };
        let adapter = instance
            .request_adapter(&request_adapter_options)
            .await
            .expect("Failed to find an appropriate adapter");

        let adapter_info = adapter.get_info();
        info!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
        debug!("get a logical device with queue for the adapter");
        let required_limits =
            wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
        let device_descriptor = wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits,
            memory_hints: wgpu::MemoryHints::MemoryUsage,
        };
        let (device, queue) = adapter
            .request_device(&device_descriptor, None)
            .await
            .expect("Failed to create device");

        debug!("create the start configuration of the surface");
        let mut config = surface
            .get_default_config(&adapter, surface_size.width, surface_size.height)
            .expect("Surface isn't supported by the adapter.");
        config.present_mode = PresentMode::AutoVsync;

        debug!("create renderer");
        let renderer = renderer_builder.build(&adapter, &device, &queue, &config);

        Self {
            window,
            instance,
            surface,
            adapter,
            config,
            device,
            queue,
            renderer,
        }
    }

    /// Resize the surface, making sure to not resize to zero.
    pub(super) fn resize(&mut self, size: PhysicalSize<u32>) {
        debug!("Surface resize {size:?}");
        if size.width == 0 || size.height == 0 {
            trace!("surface would be empty");
            return;
        }

        self.config.width = size.width;
        self.config.height = size.height;
        debug!("setting changed surface configuration: {:?}", self.config);
        self.surface.configure(&self.device, &self.config);

        debug!("notify renderer about the change in size");
        self.renderer
            .resize(&self.device, &self.queue, size.width, size.height);

        debug!("request redraw after resize");
        self.window.request_redraw();
    }

    pub(crate) fn redraw(&mut self) {
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // update the renderer state according to the current game state
        // (this might block briefly while the game state is being updated by the engine)
        self.renderer.update();
        // perform the actual render operation into the frame
        self.renderer
            .render(&texture_view, &self.device, &self.queue);

        // make the newly drawn frame visible
        self.window.pre_present_notify();
        surface_texture.present();
        self.window.request_redraw();
    }
}
