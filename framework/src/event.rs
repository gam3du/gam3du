use winit::event::WindowEvent;

#[derive(Clone, Debug)]
pub enum EngineEvent {
    Window { event: WindowEvent },
    Device { event: winit::event::DeviceEvent },
    Application { event: ApplicationEvent },
}

#[derive(Clone, Debug)]
pub enum ApplicationEvent {
    Exit,
}

impl From<ApplicationEvent> for EngineEvent {
    fn from(event: ApplicationEvent) -> Self {
        Self::Application { event }
    }
}
