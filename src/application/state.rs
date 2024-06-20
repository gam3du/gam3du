use std::{collections::HashMap, mem};

use super::{entity::Entity, event_subscriber::EventSubscriber};

pub struct State {
    pub(crate) stop: bool,

    pub(crate) entities: HashMap<String, Entity>,

    pub(crate) event_subscribers: Vec<Box<dyn EventSubscriber>>,

    pub(crate) tps: f64,
    pub(crate) ftps: f64,
    pub(crate) fps: f64,

    pub(crate) delta_tick_time: f64,
    pub(crate) delta_frame_time: f64,

    pub(crate) measured_ticks_per_second: f64,
    pub(crate) measured_frames_per_second: f64,
}

impl State {
    pub fn new() -> State {
        let mut state = State {
            stop: false,

            entities: HashMap::new(),
            event_subscribers: Vec::new(),

            tps: 60.0,
            ftps: 60.0,
            fps: 60.0,

            delta_tick_time: 0.0,
            delta_frame_time: 0.0,

            measured_ticks_per_second: 0.0,
            measured_frames_per_second: 0.0,
        };

        state.add_subscriber(Box::new(EntityComponentSystem::new()));

        state
    }

    pub fn add_subscriber(&mut self, subscriber: Box<dyn EventSubscriber>) {
        self.event_subscribers.push(subscriber);
    }

    pub fn create_entity(&mut self, name: String) -> &mut Entity {
        self.entities
            .entry(name.clone())
            .or_insert(Entity::new(name))
    }

    pub fn get_entity(&mut self, name: &str) -> Option<&mut Entity> {
        self.entities.get_mut(name)
    }
}

struct EntityComponentSystem {
    //it could make sense to store enitites here
    //entities: Vec<Entity>,
}

impl EntityComponentSystem {
    pub fn new() -> EntityComponentSystem {
        EntityComponentSystem {
            //entities: Vec::new(),
        }
    }
}

impl EventSubscriber for EntityComponentSystem {
    fn start(&mut self, state: &mut State) {
        let mut entities = mem::take(&mut state.entities);

        for entity in &mut entities {
            entity.1.start(state);
        }

        state.entities.extend(entities);
    }

    fn update(&mut self, state: &mut State) {
        let mut entities = mem::take(&mut state.entities);

        for entity in &mut entities {
            entity.1.update(state);
        }

        state.entities.extend(entities);
    }

    fn render(&mut self, state: &mut State) {
        let mut entities = mem::take(&mut state.entities);

        for entity in &mut entities {
            entity.1.render(state);
        }

        state.entities.extend(entities);
    }

    fn stop(&mut self, state: &mut State) {
        let mut entities = mem::take(&mut state.entities);

        for entity in &mut entities {
            entity.1.stop(state);
        }

        state.entities.extend(entities);
    }
}
