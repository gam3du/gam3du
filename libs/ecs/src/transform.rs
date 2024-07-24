use glam::Vec3;

use crate::component::Component;
use crate::state::State;

pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Default for TransformComponent {
    fn default() -> Self {
        TransformComponent {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Component for TransformComponent {
    fn start(&mut self, _state: &mut State) {
        //println!("{}", self.position);
    }

    fn update(&mut self, state: &mut State) {
        println!(
            "TestComponent::update | FPS: {} | TPS: {}",
            state.measured_frames_per_second, state.measured_ticks_per_second
        );
    }

    fn render(&mut self, _state: &mut State) {
        println!("TestComponent::render");
    }

    fn stop(&mut self, _state: &mut State) {
        println!("TestComponent::stop");
    }
}
