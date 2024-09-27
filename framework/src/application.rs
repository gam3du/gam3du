use crate::{
    event::{ApplicationEvent, EngineEvent},
    graphics_context::GraphicsContext,
    renderer::{self, Renderer},
    surface_wrapper::SurfaceWrapper,
};
use log::{debug, info, trace};
use std::{
    sync::{mpsc::Sender, Arc},
    time::{Duration, Instant},
};
use wgpu;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

pub struct Application<RendererBuilder: renderer::RendererBuilder> {
    renderer_builder: RendererBuilder,
    renderer: Option<RendererBuilder::Renderer>,
    pub(super) surface: SurfaceWrapper,
    context: GraphicsContext,
    window: Option<Arc<Window>>,
    title: String,
    frame_counter: u32,
    frame_time: Instant,
    event_sink: Sender<EngineEvent>,
}

impl<RendererBuilder: renderer::RendererBuilder> Application<RendererBuilder> {
    pub async fn new(
        title: String,
        event_sender: Sender<EngineEvent>,
        renderer_builder: RendererBuilder,
    ) -> Self {
        let mut surface = SurfaceWrapper::new();
        let context = GraphicsContext::init_async(&mut surface).await;

        Self {
            renderer: None,
            surface,
            context,
            window: None,
            title,
            frame_counter: 0,
            frame_time: Instant::now(),
            event_sink: event_sender,
            renderer_builder,
        }
    }

    fn update_fps(&mut self) {
        self.frame_counter += 1;
        let span = self.frame_time.elapsed();
        if span >= Duration::from_secs(1) {
            debug!(
                "{} fps",
                ((self.frame_counter as f32) / span.as_secs_f32()).round()
            );
            self.frame_counter = 0;
            self.frame_time += span;
        }
    }
}

impl<RendererBuilder: renderer::RendererBuilder> ApplicationHandler<EngineEvent>
    for Application<RendererBuilder>
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default().with_title(&self.title);

        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        self.surface
            .resume(&self.context, Arc::clone(&window), true);

        self.window = Some(window);

        // First-time init of the scene
        if self.renderer.is_none() {
            self.renderer.replace(self.renderer_builder.build(
                &self.context.adapter,
                &self.context.device,
                &self.context.queue,
                self.surface.config(),
            ));
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: EngineEvent) {
        match event {
            EngineEvent::Application {
                event: ApplicationEvent::Exit,
            } => {
                info!("Window event loop received an ExitEvent. Shutting down event loop.");
                event_loop.exit();
            }
            other => todo!("unknown event: {other:?}"),
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                trace!("WindowEvent::Resized({size:?})");

                self.surface.resize(&self.context, size);
                self.renderer.as_mut().unwrap().resize(
                    &self.context.device,
                    &self.context.queue,
                    self.surface.config(),
                );

                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::CloseRequested => {
                trace!("WindowEvent::CloseRequested()");
                self.event_sink.send(ApplicationEvent::Exit.into()).unwrap();
            }

            WindowEvent::KeyboardInput {
                device_id,
                event: ref key_event,
                is_synthetic,
            } => {
                trace!("WindowEvent::KeyboardInput({device_id:?}, {key_event:?}, {is_synthetic})");
                let KeyEvent {
                    physical_key: _,
                    ref logical_key,
                    text: _,
                    location: _,
                    state: _,
                    repeat: _,
                    ..
                } = *key_event;

                match *logical_key {
                    Key::Named(key) => {
                        trace!("WindowEvent::KeyboardInput::logical_key::Named({key:?})");
                        match key {
                            NamedKey::Escape => {
                                self.event_sink.send(ApplicationEvent::Exit.into()).unwrap();
                            }
                            _ => {
                                self.event_sink.send(EngineEvent::Window { event }).unwrap();
                            }
                        }
                    }
                    Key::Character(ref key) => {
                        trace!("WindowEvent::KeyboardInput::logical_key::Character({key:?})");
                        self.event_sink.send(EngineEvent::Window { event }).unwrap();
                    }
                    Key::Unidentified(ref key) => {
                        trace!("WindowEvent::KeyboardInput::logical_key::Unidentified({key:?})");
                        self.event_sink.send(EngineEvent::Window { event }).unwrap();
                    }
                    Key::Dead(key) => {
                        trace!("WindowEvent::KeyboardInput::logical_key::Dead({key:?})");
                        self.event_sink.send(EngineEvent::Window { event }).unwrap();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // On MacOS, currently redraw requested comes in _before_ Init does.
                // If this happens, just drop the requested redraw on the floor.
                //
                // See https://github.com/rust-windowing/winit/issues/3235 for some discussion
                let Some(renderer) = self.renderer.as_mut() else {
                    return;
                };

                let frame = self.surface.acquire(&self.context);
                let texture_view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                    format: Some(self.surface.config().view_formats[0]),
                    ..wgpu::TextureViewDescriptor::default()
                });

                renderer.update();
                renderer.render(&texture_view, &self.context.device, &self.context.queue);

                frame.present();
                self.update_fps();

                self.window.as_ref().unwrap().request_redraw();
                // self.event_sink.send(EngineEvent::Window { event }).unwrap();
            }
            _ => {
                self.event_sink.send(EngineEvent::Window { event }).unwrap();
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
        trace!("window event loop is exiting");
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        trace!("window event loop was suspended");
        self.surface.suspend();
    }
}

// pub fn start(mut app: Application) {
//     let event_loop = EventLoop::new().unwrap();

//     // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
//     // dispatched any events. This is ideal for games and similar applications.
//     event_loop.set_control_flow(ControlFlow::Poll);

//     let proxy = event_loop.create_proxy();

//     //proxy.send_event(event);

//     //let app = Application::new(title, receiver, event_sender);
//     log::info!("Entering event loop...");
//     event_loop.run_app(&mut app).unwrap();
// }
