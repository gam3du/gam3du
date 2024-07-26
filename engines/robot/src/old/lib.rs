mod camera;
mod floor;
mod floor_renderer;
mod projection;
mod renderer;
mod robot_renderer;
pub(crate) mod scene;
mod tile;

use camera::Camera;
use floor_renderer::FloorRenderer;
use projection::Projection;
use renderer::{RenderState, Renderer};
use robot_renderer::RobotRenderer;
use scene::DepthTexture;
pub use scene::Scene;

use std::{
    f32::consts::{PI, TAU},
    ops::{AddAssign, SubAssign},
    time::Duration,
};

use bytemuck::{Pod, Zeroable};
use floor::Floor;
use glam::{FloatExt, IVec3, Vec2, Vec3, Vec4};
use log::error;
use std::time::Instant;
use tile::{LineSegment, Tile};

use bindings::api::Identifier;




#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct Vertex {
    pos: Vec4,
    tex_coord: Vec2,
    _padding: Vec2,
}

fn vertex(position: Vec3, texture_coord: Vec2) -> Vertex {
    Vertex {
        pos: Vec4::new(position.x, position.y, position.z, 1.0),
        tex_coord: texture_coord,
        _padding: Vec2::default(),
    }
}

#[derive(Debug)]
pub struct Command {
    pub name: Identifier,
}
