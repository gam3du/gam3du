use glam::{IVec3, Vec3};
use std::time::Instant;

use bindings::api::Identifier;

use crate::{renderer::RenderState, tile::Tile, Orientation};

use super::{
    camera::Camera, floor::Floor, floor_renderer::FloorRenderer, projection::Projection,
    robot_renderer::RobotRenderer, Robot,
};


pub struct Scene {
    depth_map: DepthTexture,
    start_time: Instant,
    // projection: Projection,
    // camera: Camera,
    robot: Robot,
    // floor: Floor,
    state: RenderState,
    robot_renderer: RobotRenderer,
    floor_renderer: FloorRenderer,
}

impl Scene {


    pub fn process_command(&mut self, command: &Identifier) {
        self.robot.process_command(command, &mut self.state);
    }
}

