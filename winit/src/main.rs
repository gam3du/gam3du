#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unwrap_in_result,
    reason = "just demo code"
)]
//! Simple winit application.

use ::tracing::{error, info};
#[cfg(not(android_platform))]
use raw_window_handle::{DisplayHandle, HasDisplayHandle};
#[cfg(not(android_platform))]
use softbuffer::{Context, Surface};
use std::error::Error;
use std::mem;
#[cfg(not(android_platform))]
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
#[cfg(macos_platform)]
use winit::platform::macos::{
    ApplicationHandlerExtMacOS, OptionAsAlt, WindowAttributesExtMacOS, WindowExtMacOS,
};
#[cfg(any(x11_platform, wayland_platform))]
use winit::platform::startup_notify::{
    self, EventLoopExtStartupNotify, WindowAttributesExtStartupNotify, WindowExtStartupNotify,
};
#[cfg(web_platform)]
use winit::platform::web::{ActiveEventLoopExtWeb, WindowAttributesExtWeb};
use winit::window::{Icon, Window, WindowAttributes, WindowId};

mod tracing;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(web_platform)]
    console_error_panic_hook::set_once();

    tracing::init();

    let event_loop = EventLoop::new()?;

    let app = Application::new(&event_loop);
    Ok(event_loop.run_app(app)?)
}

/// Application state and event handling.
struct Application {
    /// Application icon.
    icon: Icon,
    window_state: Option<WindowState>,
    /// Drawing context.
    ///
    /// With OpenGL it could be `EGLDisplay`.
    #[cfg(not(android_platform))]
    context: Option<Context<DisplayHandle<'static>>>,
}

impl Application {
    fn new(event_loop: &EventLoop) -> Self {
        #[cfg(not(android_platform))]
        #[expect(unsafe_code, reason = "this was taken from the winit example code")]
        let context = Some(
            // SAFETY: we drop the context right before the event loop is stopped, thus making it safe.
            Context::new(unsafe {
                mem::transmute::<DisplayHandle<'_>, DisplayHandle<'static>>(
                    event_loop.display_handle().unwrap(),
                )
            })
            .unwrap(),
        );

        let icon = load_icon(include_bytes!("../icon.png"));

        Self {
            #[cfg(not(android_platform))]
            context,
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
                #[cfg(any(x11_platform, wayland_platform))]
                {
                    startup_notify::set_activation_token_env(_token);
                    if let Err(err) = self.create_window(event_loop, None) {
                        error!("Error creating new window: {err}");
                    }
                }
            }
            WindowEvent::SurfaceResized(size) => {
                window_state.resize(size);
            }
            WindowEvent::RedrawRequested => {
                if let Err(err) = window_state.draw() {
                    error!("Error drawing window: {err}");
                }
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
        let this = &mut *self;
        // TODO read-out activation token.

        let window_attributes = WindowAttributes::default()
            .with_title("Demo window")
            .with_window_icon(Some(this.icon.clone()));

        #[cfg(any(x11_platform, wayland_platform))]
        if let Some(token) = event_loop.read_token_from_env() {
            startup_notify::reset_activation_token_env();
            info!("Using token {:?} to activate a window", token);
            window_attributes = window_attributes.with_activation_token(token);
        }

        #[cfg(web_platform)]
        {
            window_attributes = window_attributes.with_append(true);
        }

        let window = event_loop
            .create_window(window_attributes)
            .expect("failed to create initial window");

        let window_state = WindowState::new(this, window).expect("failed to create initial window");
        let window_id = window_state.window.id();
        info!("Created new window with id={window_id:?}");
        this.window_state.replace(window_state);
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window_state.is_none() {
            info!("No windows left, exiting...");
            event_loop.exit();
        }
    }

    #[cfg(not(android_platform))]
    fn exiting(&mut self, _event_loop: &dyn ActiveEventLoop) {
        // We must drop the context here.
        drop(self.context.take());
    }
}

/// State of the window.
struct WindowState {
    /// Render surface.
    ///
    /// NOTE: This surface must be dropped before the `Window`.
    #[cfg(not(android_platform))]
    surface: Surface<DisplayHandle<'static>, Arc<dyn Window>>,
    /// The actual winit Window.
    window: Arc<dyn Window>,
}

impl WindowState {
    fn new(app: &Application, window: Box<dyn Window>) -> Result<Self, Box<dyn Error>> {
        let window: Arc<dyn Window> = Arc::from(window);

        // SAFETY: the surface is dropped before the `window` which provided it with handle, thus
        // it doesn't outlive it.
        #[cfg(not(android_platform))]
        let surface = Surface::new(app.context.as_ref().unwrap(), Arc::clone(&window))?;

        let size = window.surface_size();
        let mut state = Self {
            #[cfg(not(android_platform))]
            surface,
            window,
        };

        state.resize(size);
        Ok(state)
    }

    /// Resize the surface to the new size.
    fn resize(&mut self, size: PhysicalSize<u32>) {
        info!("Surface resized to {size:?}");
        #[cfg(not(android_platform))]
        {
            let (Some(width), Some(height)) =
                (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
            else {
                return;
            };
            self.surface
                .resize(width, height)
                .expect("failed to resize inner buffer");
        }
        self.window.request_redraw();
    }

    /// Draw the window contents.
    #[cfg(not(android_platform))]
    fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        const BACKGROUND_COLOR: u32 = 0xff_18_18_58;

        let mut buffer = self.surface.buffer_mut()?;
        buffer.fill(BACKGROUND_COLOR);
        self.window.pre_present_notify();
        buffer.present()?;
        Ok(())
    }

    #[cfg(android_platform)]
    fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Drawing but without rendering...");
        Ok(())
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
