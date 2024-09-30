use super::{component::Component, state::State};
use std::{any::TypeId, collections::HashMap, mem};

pub struct Entity {
    #[expect(
        dead_code,
        reason = "TODO likely used in future at least for debugging"
    )]
    name: String,
    components: HashMap<TypeId, Vec<Box<dyn Component>>>,
}

impl Entity {
    #[must_use]
    pub fn new(name: String) -> Self {
        Entity {
            name,
            components: HashMap::new(),
        }
    }

    pub fn start(&mut self, state: &mut State) {
        for components in self.components.values_mut() {
            let mut components_copy = mem::take(components);

            for component in &mut components_copy {
                component.start(state);
            }

            components.extend(components_copy);
        }
    }

    pub fn update(&mut self, state: &mut State) {
        for components in self.components.values_mut() {
            let mut components_copy = mem::take(components);

            for component in &mut components_copy {
                component.update(state);
            }

            components.extend(components_copy);
        }
    }

    pub fn render(&mut self, state: &mut State) {
        for components in self.components.values_mut() {
            let mut components_copy = mem::take(components);

            for component in &mut components_copy {
                component.render(state);
            }

            components.extend(components_copy);
        }
    }

    pub fn stop(&mut self, state: &mut State) {
        for components in self.components.values_mut() {
            let mut components_copy = mem::take(components);

            for component in &mut components_copy {
                component.stop(state);
            }

            components.extend(components_copy);
        }
    }

    pub fn add_component<T: Component + 'static>(&mut self, component: T) {
        self.components
            .entry(TypeId::of::<T>())
            .or_default()
            .push(Box::new(component));
    }

    pub fn get_components<T>(&mut self) -> Vec<&mut T>
    where
        T: Component + 'static,
    {
        let mut result = Vec::new();

        let components = self.components.get_mut(&TypeId::of::<T>()).unwrap();

        for component in components {
            unsafe {
                result.push(
                    &mut *(component as *mut Box<dyn Component> as *mut Box<T>)
                        .as_mut()
                        .unwrap()
                        .as_mut(),
                );

                //let component_casted_to_t = &mut *(component as *mut Box<dyn Component> as *mut Box<T>).as_mut().unwrap().as_mut();
                //result.push(Some(component_casted_to_t));
            }
        }

        result
    }
}
