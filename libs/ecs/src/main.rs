#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    // clippy::missing_panics_doc,
    clippy::print_stdout,
    clippy::unwrap_used,
    // clippy::expect_used,
    clippy::indexing_slicing,
    // clippy::panic,
    reason = "TODO remove before release"
)]

use glam::Vec3;
use lib_ecs::event_subscriber::EventSubscriber;
use lib_ecs::transform::TransformComponent;
use lib_ecs::Application;

fn main() {
    println!("Hello, world!");

    let mut app = Application::default();

    {
        let state_arc = app.get_state_arc();

        let mut state = state_arc.write().unwrap();

        state.create_entity("Test".to_owned());

        let entity = state.get_entity("Test").unwrap();

        entity.add_component(TransformComponent::default());

        let mut components = entity.get_components::<TransformComponent>();
        let component = &mut *components[0];

        component.position = Vec3::new(1.0, 2.0, 3.0);

        state.add_subscriber(Box::new(TestSubscriber {}));
    }

    app.start();
}

struct TestSubscriber;

impl EventSubscriber for TestSubscriber {
    fn update(&mut self, state: &mut lib_ecs::state::State) {
        println!("delta time: {}", state.delta_tick_time);
    }
}
