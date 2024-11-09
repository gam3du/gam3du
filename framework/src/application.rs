use crate::{render_surface::RenderSurface, renderer};
use gam3du_framework_common::event::{ApplicationEvent, FrameworkEvent};
use std::sync::mpsc::{Receiver, Sender};
use tracing::{debug, info, trace};
use web_time::{Duration, Instant};

// #[cfg(not(target_arch = "wasm32"))]
// use winit::platform::x11::WindowAttributesExtX11;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    window::{WindowAttributes, WindowId},
};

pub struct Application<RendererBuilder: renderer::RendererBuilder, Updater: FnMut()> {
    renderer_builder: Option<RendererBuilder>,
    title: String,
    frame_counter: u32,
    frame_time: Instant,
    event_sink: Sender<FrameworkEvent>,
    framework_events: Receiver<FrameworkEvent>,
    render_surface: Option<RenderSurface<RendererBuilder::Renderer>>,
    game_loop_updater: Updater,
}

impl<RendererBuilder: renderer::RendererBuilder, Updater: FnMut()>
    Application<RendererBuilder, Updater>
{
    pub fn new(
        title: impl Into<String>,
        event_sender: Sender<FrameworkEvent>,
        renderer_builder: RendererBuilder,
        framework_events: Receiver<FrameworkEvent>,
        game_loop_updater: Updater,
    ) -> Self {
        Self {
            title: title.into(),
            frame_counter: 0,
            frame_time: Instant::now(),
            event_sink: event_sender,
            renderer_builder: Some(renderer_builder),
            framework_events,
            render_surface: None,
            game_loop_updater,
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

impl<RendererBuilder: renderer::RendererBuilder, Updater: FnMut()> ApplicationHandler
    for Application<RendererBuilder, Updater>
{
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        // Create initial window.
        #[allow(unused_mut, reason = "required for target=wasm")]
        let mut window_attributes = WindowAttributes::default()
            .with_title(&self.title)
            .with_surface_size(PhysicalSize::new(1600, 900));

        #[cfg(any(x11_platform, wayland_platform))]
        if let Some(token) = event_loop.read_token_from_env() {
            startup_notify::reset_activation_token_env();
            info!("Using token {:?} to activate a window", token);
            window_attributes = window_attributes.with_activation_token(token);
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWeb;
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

        let window = event_loop.create_window(window_attributes).unwrap();

        let now = Instant::now();
        let render_surface = pollster::block_on(RenderSurface::new(
            window,
            self.renderer_builder.take().unwrap(),
        ));
        debug!("creating the render surface took {:?}", now.elapsed());

        assert!(
            self.render_surface.replace(render_surface).is_none(),
            "Double initialization of the main window"
        );
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        // TODO handle receiver errors
        while let Ok(event) = self.framework_events.recv() {
            match event {
                FrameworkEvent::Application {
                    event: ApplicationEvent::Exit,
                } => {
                    info!("Window event loop received an ExitEvent. Shutting down event loop.");
                    event_loop.exit();
                }
                other => todo!("unknown event: {other:?}"),
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::SurfaceResized(size) => {
                trace!("WindowEvent::SurfaceResized({size:?})");
                let Some(render_surface) = self.render_surface.as_mut() else {
                    trace!("cannot resize a not (yet?) existing surface");
                    return;
                };
                render_surface.resize(size);
            }

            WindowEvent::RedrawRequested => {
                // trace!("WindowEvent::RedrawRequested");
                let Some(render_surface) = self.render_surface.as_mut() else {
                    trace!("cannot redraw a not (yet?) existing surface");
                    return;
                };
                (self.game_loop_updater)();
                render_surface.redraw();
                self.update_fps();
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
                                self.event_sink
                                    .send(FrameworkEvent::Window { event })
                                    .unwrap();
                            }
                        }
                    }
                    Key::Character(ref key) => {
                        trace!("WindowEvent::KeyboardInput::logical_key::Character({key:?})");
                        self.event_sink
                            .send(FrameworkEvent::Window { event })
                            .unwrap();
                    }
                    Key::Unidentified(ref key) => {
                        trace!("WindowEvent::KeyboardInput::logical_key::Unidentified({key:?})");
                        self.event_sink
                            .send(FrameworkEvent::Window { event })
                            .unwrap();
                    }
                    Key::Dead(key) => {
                        trace!("WindowEvent::KeyboardInput::logical_key::Dead({key:?})");
                        self.event_sink
                            .send(FrameworkEvent::Window { event })
                            .unwrap();
                    }
                }
            }
            _ => {
                self.event_sink
                    .send(FrameworkEvent::Window { event })
                    .unwrap();
            }
        }
    }

    fn exiting(&mut self, _event_loop: &dyn ActiveEventLoop) {
        debug!("window event loop is exiting");
    }
}
