use std::{collections::HashMap, num::NonZeroU128, sync::mpsc::Sender};

// struct Subscriber {
//     id: NonZeroU128,
//     sender: Sender<GameEvent>,
// }

// impl Hash for Subscriber {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.id.hash(state);
//     }
// }

// impl PartialEq for Subscriber {
//     fn eq(&self, other: &Self) -> bool {
//         self.id == other.id
//     }
// }

// impl Eq for Subscriber {}

#[derive(Default)]
pub(crate) struct EventRegistries {
    pub(crate) robot_stopped: EventRegistry,
}

#[derive(Default)]
pub(crate) struct EventRegistry {
    subscribers: HashMap<NonZeroU128, Sender<GameEvent>>,
}

impl EventRegistry {
    pub(crate) fn subscribe(&mut self, id: NonZeroU128, sender: Sender<GameEvent>) {
        self.subscribers.insert(id, sender);
    }
    pub(crate) fn unsubscribe(&mut self, id: NonZeroU128) {
        self.subscribers.remove(&id);
    }
    pub(crate) fn notify(&mut self) {
        let event = GameEvent::RobotStopped;
        for subscriber in self.subscribers.values_mut() {
            subscriber.send(event.clone());
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum GameEvent {
    RobotStopped,
}
