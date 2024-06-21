// has false positives; enable every now and then to see whether there are actually missed opportunities
#![allow(missing_copy_implementations)]
// usually too noisy. Disable every now and then to see whether there are actually identifiers that need to be improved.
#![allow(unused_crate_dependencies)]
// TODO re-enable this later and review all occurrences
#![allow(clippy::cast_precision_loss)]
// TODO remove before release
#![allow(clippy::missing_panics_doc)]
#![allow(missing_docs)]
#![allow(clippy::print_stdout)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::panic)]

// TODO enable hand-picked clippy lints from the `restriction` group

use gam3du::{
    application::{self, event_subscriber::EventSubscriber},
    transform::TransformComponent,
};
use glam::Vec3;

fn main() {
    println!("Hello, world!");

    let mut app = application::Application::default();

    {
        let state_arc = app.get_state_arc();

        let mut state = state_arc.write().unwrap();

        state.create_entity("Test".to_string());

        let entity = state.get_entity("Test").unwrap();

        entity.add_component(TransformComponent::default());

        let mut components = entity.get_components::<TransformComponent>();
        let component = components.get_mut(0).unwrap();

        component.position = Vec3::new(1.0, 2.0, 3.0);

        state.add_subscriber(Box::new(TestSubscriber {}));
    }

    app.start();
}

struct TestSubscriber {}

impl EventSubscriber for TestSubscriber {
    fn update(&mut self, state: &mut application::state::State) {
        println!("delta time: {}", state.delta_tick_time);
    }
}
