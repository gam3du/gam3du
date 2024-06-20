#![warn(clippy::all, clippy::pedantic)]
// TODO re-enable this later and review all occurrences
#![allow(clippy::cast_precision_loss)]

// TODO enable hand-picked clippy lints from the `restriction` group

//mod application_state;
mod framework;
mod logging;
mod python;
mod scene;

use std::{
    sync::{atomic::AtomicU16, mpsc::channel},
    thread,
};

use application::event_subscriber::EventSubscriber;
use glam::Vec3;
use transform::TransformComponent;

mod application;
mod transform;

use logging::init_logger;
use python::python_runner;

pub(crate) static ROTATION: AtomicU16 = AtomicU16::new(0);

fn main() {
    //ecs_test();

    init_logger();

    let (command_sender, command_receiver) = channel();

    let source_path = "python/test.py";
    let python_tread = thread::spawn(move || python_runner(&source_path, command_sender));

    pollster::block_on(framework::start("demo scene".into(), command_receiver));
    // FIXME on Windows the window will still be unresponsively lingering until the control was given back to the OS (maybe a bug in `winit`)

    python_tread.join().unwrap();
}

struct TestSubscriber {}

impl EventSubscriber for TestSubscriber {
    fn update(&mut self, state: &mut application::state::State) {
        println!("delta time: {}", state.delta_tick_time);
    }
}

fn ecs_test() {
    println!("Hello, world!");

    let mut app = application::Application::new();

    {
        let state_arc = app.get_state_arc();

        let mut state = state_arc.write().unwrap();

        state.create_entity("Test".to_string());

        let entity = state.get_entity("Test").unwrap();

        entity.add_component(TransformComponent::new());

        let mut components = entity.get_components::<TransformComponent>();
        let component = components.get_mut(0).unwrap();

        component.position = Vec3::new(1.0, 2.0, 3.0);

        state.add_subscriber(Box::new(TestSubscriber {}));
    }

    app.start();
}
