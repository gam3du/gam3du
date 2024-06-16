/// captures the current state of the running program
struct State {
    entities: Vec<Entity>,
}

impl State {
    fn new() -> Self {
        State {
            entities: Vec::new(),
        }
    }

    fn add_entity(&mut self, entity: Entity) {
        self.entities_to_add.push(entity);
    }

    fn start(&mut self) {
        self.entities
            .iter_mut()
            .for_each(|component| component.start(self));
    }
}

struct Entity {
    name: String,
    components: Vec<Behavior>,
}

impl Entity {
    fn new(name: String) -> Self {
        Entity {
            name,
            components: Vec::new(),
        }
    }

    fn add_component(&mut self, component: Behavior) {
        self.components.push(component);
    }

    fn start(&mut self, state: &mut State) {
        self.components
            .iter_mut()
            .for_each(|component| component.start(state));
    }
}

trait Behavior {
    fn start(&mut self, state: &mut State) {}
}

struct Robot {}

impl Behavior for Robot {
    fn start(&mut self, state: &mut State) {
        state.add_entity(entity);
    }
}
