use std::{
    sync::{mpsc::Sender, Arc},
    time::{Duration, Instant},
};

use bindings::event::{ApplicationEvent, EngineEvent, EventRouter};
use engine_robot::{Renderer, RendererBuilder};
use log::{debug, info, trace};
use wgpu;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

use crate::{graphics_context::GraphicsContext, surface_wrapper::SurfaceWrapper};

pub struct Application {
    renderer_builder: RendererBuilder,
    renderer: Option<Renderer>,
    pub(super) surface: SurfaceWrapper,
    context: GraphicsContext,
    // window: Arc<Window>,
    window: Option<Arc<Window>>,
    title: String,
    frame_counter: u32,
    frame_time: Instant,
    // receiver: Receiver<EngineEvent>,
    // current_command: Option<EngineEvent>,
    event_sink: Sender<EngineEvent>,
}

impl Application {
    pub async fn new(
        title: String,
        event_router: &mut EventRouter,
        renderer_builder: RendererBuilder,
    ) -> Self {
        let mut surface = SurfaceWrapper::new();
        let context = GraphicsContext::init_async(&mut surface).await;
        let event_sender = event_router.clone_sender();

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
}

impl ApplicationHandler<EngineEvent> for Application {
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
                info!("Application event loop received an ExitEvent. Shutting down event loop.");
                event_loop.exit();
            }
            other => todo!("unknown event: {other:?}"),
        }
    }

    // TODO maybe the trace output can be moved elsewhere?
    #[allow(clippy::too_many_lines)]
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
                self.event_sink
                    .send(EngineEvent::Application {
                        event: ApplicationEvent::Exit,
                    })
                    .unwrap();
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
                        // more branches will follow for sure …
                        #[allow(clippy::single_match)]
                        match key {
                            NamedKey::Escape => {
                                self.event_sink
                                    .send(EngineEvent::Application {
                                        event: ApplicationEvent::Exit,
                                    })
                                    .unwrap();
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
                // if self.current_command.is_none() {
                //     match self.receiver.try_recv() {
                //         Ok(command) => {
                //             self.current_command.replace(command);
                //         }
                //         Err(TryRecvError::Disconnected | TryRecvError::Empty) => {}
                //     }
                // }

                // if let Some(EngineEvent::ApiCall { api, ref command }) = self.current_command.take()
                // {
                //     if let Some(scene) = self.renderer.as_mut() {
                //         if scene.is_idle() {
                //             scene.process_command(command);
                //         }
                //     }
                // }

                // On MacOS, currently redraw requested comes in _before_ Init does.
                // If this happens, just drop the requested redraw on the floor.
                //
                // See https://github.com/rust-windowing/winit/issues/3235 for some discussion
                let Some(renderer) = self.renderer.as_mut() else {
                    return;
                };

                renderer.update();

                let frame = self.surface.acquire(&self.context);
                let texture_view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                    format: Some(self.surface.config().view_formats[0]),
                    ..wgpu::TextureViewDescriptor::default()
                });

                renderer.render(&texture_view, &self.context.device, &self.context.queue);

                frame.present();

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
