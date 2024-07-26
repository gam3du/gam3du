use std::time::Instant;

use glam::{IVec3, Vec3};

use crate::{scene::DepthTexture, Tick};

use super::{
    camera::Camera, floor_renderer::FloorRenderer, projection::Projection,
    robot_renderer::RobotRenderer, tile::Tile, Animation, Orientation,
};

pub(crate) struct Renderer {
    pub(crate) robot_renderer: RobotRenderer,
    pub(crate) floor_renderer: FloorRenderer,

    state: RenderState,

}


impl RenderState {
    pub fn init(
        surface: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let robot = Robot::new();
        let floor = Floor::new();
        let tiles = floor.tiles;


        let state = RenderState {};

        Self {
            depth_map,
            robot_renderer,
            floor_renderer,
            state,
            camera,
            projection,
            animation_position: Vec3::default(),
            animation_angle: 0.0,
            position: IVec3::default(),
            orientation: Orientation::E,
            current_animation: None,
            tiles,
            tiles_tainted: true,
        }
    }

}
