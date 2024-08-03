use winit::event::WindowEvent;

use crate::api::{Identifier, Value};

#[derive(Clone, Debug)]
pub enum EngineEvent {
    Window {
        event: WindowEvent,
    },
    Device {
        event: winit::event::DeviceEvent,
    },
    ApiCall {
        api: Identifier,
        command: Identifier,
    },
    RobotEvent {
        command: Identifier,
        parameters: Vec<Value>,
    },
    Application {
        event: ApplicationEvent,
    },
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
