use std::sync::mpsc::{channel, Receiver, Sender};

use winit::event::WindowEvent;

use crate::api::Identifier;

#[derive(Clone)]
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
    Application {
        event: ApplicationEvent,
    },
}

#[derive(Clone, Debug)]
pub enum ApplicationEvent {
    Exit,
}

pub trait EventHandler: Send {
    fn handle_event(&self, event: EngineEvent) -> Option<EngineEvent>;
}

pub struct EventRouter {
    sender: Sender<EngineEvent>,
    receiver: Receiver<EngineEvent>,
    handlers: Vec<Box<dyn Fn(EngineEvent) -> Option<EngineEvent> + Send>>,
}

impl EventRouter {
    #[must_use]
    pub fn clone_sender(&self) -> Sender<EngineEvent> {
        self.sender.clone()
    }

    pub fn add_handler(&mut self, handler: Box<dyn Fn(EngineEvent) -> Option<EngineEvent> + Send>) {
        self.handlers.push(handler);
    }

    pub fn run(&self) {
        'next_event: while let Ok(mut event) = self.receiver.recv() {
            for handler in &self.handlers {
                let Some(handled_event) = handler(event) else {
                    continue 'next_event;
                };
                event = handled_event;
            }
            // unhandled event
        }
    }
}

impl Default for EventRouter {
    fn default() -> Self {
        let (sender, receiver) = channel();

        EventRouter {
            sender,
            receiver,
            handlers: Vec::new(),
        }
    }
}
