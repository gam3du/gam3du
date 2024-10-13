use winit::event::WindowEvent;

#[derive(Clone, Debug)]
pub enum FrameworkEvent {
    Window { event: WindowEvent },
    Device { event: winit::event::DeviceEvent },
    Application { event: ApplicationEvent },
}

#[derive(Clone, Debug)]
pub enum ApplicationEvent {
    Exit,
}

impl From<ApplicationEvent> for FrameworkEvent {
    fn from(event: ApplicationEvent) -> Self {
        Self::Application { event }
    }
}
