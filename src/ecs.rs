// TODO remove this once the design finalizes
#![allow(dead_code)]

use std::{
    collections::{hash_map, HashMap},
    num::NonZeroU64,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use glam::Vec3;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct EntityId(NonZeroU64);

struct State {
    /// holds the id to be used for the next entity that will be created
    next_entity_id: AtomicU64,

    /// holds the components of all entities
    /// The `Option<Box<_>>` will make it cheap to temporarily detach the structure from this [`State`]
    /// So we can immutably pass the state along with the components to each systems.
    /// TODO this is rather hacky; check for what `state` will be used and whether this can be encapsulated in a separate struct so that we don't need to pass the entire state.
    components: Option<Box<Components>>,

    /// a list of systems that will be executed in order to update the associated components
    systems: Option<Vec<Box<dyn System>>>,

    /// time that has elapsed since the recent update
    time_delta: Duration,
}

impl Default for State {
    fn default() -> Self {
        Self {
            next_entity_id: AtomicU64::new(1),
            systems: Some(Vec::new()),
            components: Some(Box::new(Components::default())),
            time_delta: Duration::default(),
        }
    }
}

impl State {
    /// returns a unique [`EntityId`]
    fn generate_entity_id(&self) -> EntityId {
        let id = self.next_entity_id.fetch_add(1, Ordering::Relaxed);
        EntityId(
            id.try_into()
                .expect("Looks like someone created a state with a 0 stored as `next_entity_id`"),
        )
    }

    /// This is meant to be called in the game loop
    fn update(&mut self) {
        // temporarily detach components to obtain mutable access that can be shared
        let mut components = self.components.take().unwrap();
        // temporarily detach systems to obtain mutable access that can be shared
        let mut systems = self.systems.take().unwrap();

        systems
            .iter_mut()
            .for_each(|system| system.update(self, &mut components));

        // re-attach components and systems
        self.components = Some(components);
        self.systems = Some(systems);
    }
}

#[derive(Default)]
struct Components {
    position: ComponentCollection<PositionComponent>,
    physics: ComponentCollection<PhysicsComponent>,
}

struct ComponentCollection<Item: Component> {
    components: HashMap<EntityId, Item>,
}

impl<Item: Component> ComponentCollection<Item> {
    fn get_mut(&mut self, entity_id: EntityId) -> Option<&mut Item> {
        self.components.get_mut(&entity_id)
    }
}

impl<Item: Component> Default for ComponentCollection<Item> {
    fn default() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
}

/// permits using `&ComponentCollection` in a `for`-loop to iterate over `&Item`
impl<'iter, Item: Component> IntoIterator for &'iter ComponentCollection<Item> {
    type Item = &'iter Item;

    type IntoIter = hash_map::Values<'iter, EntityId, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.components.values()
    }
}

/// permits using `&mut ComponentCollection` in a `for`-loop to iterate over `&mut Item`
impl<'iter, Item: Component> IntoIterator for &'iter mut ComponentCollection<Item> {
    type Item = &'iter mut Item;

    type IntoIter = hash_map::ValuesMut<'iter, EntityId, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.components.values_mut()
    }
}

struct PositionComponent {
    entity_id: EntityId,
    position: Vec3,
}

impl PositionComponent {
    fn move_by(&mut self, delta: Vec3) {
        self.position += delta;
    }

    fn update(&mut self, state: &State, physics: &PhysicsComponent) {
        self.move_by(physics.velocity * state.time_delta.as_secs_f32());
    }
}

impl Component for PositionComponent {
    fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}

struct PhysicsComponent {
    entity_id: EntityId,
    velocity: Vec3,
    acceleration: Vec3,
}

impl PhysicsComponent {
    fn update(&mut self, state: &State) {
        self.velocity += self.acceleration * state.time_delta.as_secs_f32();
    }
}

impl Component for PhysicsComponent {
    fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}

trait Component {
    fn entity_id(&self) -> EntityId;

    // this would only make sense if every component is required to implement an update without neeing access to any other component
    // fn update(&mut self, state: &State);
}

trait System {
    fn update(&mut self, state: &State, components: &mut Components);
}

struct MovementSystem;

impl System for MovementSystem {
    fn update(&mut self, state: &State, components: &mut Components) {
        let Components {
            ref mut position,
            ref mut physics,
        } = *components;

        for physics_component in physics {
            let entity_id = physics_component.entity_id();
            let Some(position_component) = position.get_mut(entity_id) else {
                continue;
            };

            physics_component.update(state);
            position_component.update(state, physics_component);
        }
    }
}
