mod camera;
mod floor;
mod floor_renderer;
mod projection;
mod renderer;
mod robot_renderer;
pub(crate) mod scene;
mod tile;

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
use tile::LineSegment;

use bindings::api::Identifier;

pub struct Robot {
    animation_position: Vec3,
    animation_angle: f32,
    position: IVec3,
    orientation: Orientation,
    current_animation: Option<Animation>,
}

impl Robot {
    #[must_use]
    pub fn new() -> Self {
        let orientation = Orientation::default();
        let position = IVec3::new(0, 0, 0);

        Self {
            animation_position: position.as_vec3() + Vec3::new(0.5, 0.5, 0.0),
            current_animation: None,
            animation_angle: orientation.angle(),
            orientation,
            position,
        }
    }

    pub fn is_idle(&self) -> bool {
        self.current_animation.is_none()
    }

    pub fn process_command(&mut self, command: &Identifier, floor: &mut Floor) {
        if let Some(current_animation) = self.current_animation.take() {
            current_animation.complete(&mut self.animation_position, &mut self.animation_angle);
        }

        self.current_animation = match command.0.as_str() {
            "MoveForward" => {
                let segment = LineSegment::from(self.orientation);

                // TODO make this a safe function
                #[allow(clippy::cast_sign_loss)]
                let start_index = (self.position.y * 10 + self.position.x + 55) as usize;
                floor.tiles[start_index].line_pattern |= segment;

                let offset = self.orientation.as_ivec3();
                if offset.x != 0 && offset.y != 0 {
                    // TODO make this a safe function
                    #[allow(clippy::cast_sign_loss)]
                    let index0 =
                        (self.position.y * 10 + (self.position.x + offset.x) + 55) as usize;
                    floor.tiles[index0].line_pattern |= segment.get_x_corner().unwrap();

                    // TODO make this a safe function
                    #[allow(clippy::cast_sign_loss)]
                    let index1 =
                        ((self.position.y + offset.y) * 10 + self.position.x + 55) as usize;
                    floor.tiles[index1].line_pattern |= -segment.get_x_corner().unwrap();
                }

                self.position += offset;

                // TODO make this a safe function
                #[allow(clippy::cast_sign_loss)]
                let end_index = (self.position.y * 10 + self.position.x + 55) as usize;
                floor.tiles[end_index].line_pattern |= -segment;
                floor.tainted = true;

                Some(Animation::Move {
                    start: self.animation_position,
                    end: self.position.as_vec3() + Vec3::new(0.5, 0.5, 0.0),
                    start_time: Instant::now(),
                    duration: Duration::from_millis(1_000),
                })
            }
            "TurnLeft" => {
                self.orientation += 1;
                Some(Animation::Rotate {
                    start: self.animation_angle,
                    end: self.orientation.angle(),
                    start_time: Instant::now(),
                    duration: Duration::from_millis(1_000),
                })
            }
            "TurnRight" => {
                self.orientation -= 1;
                Some(Animation::Rotate {
                    start: self.animation_angle,
                    end: self.orientation.angle(),
                    start_time: Instant::now(),
                    duration: Duration::from_millis(1_000),
                })
            }
            other => {
                error!("Unknown Command: {other}");
                None
            }
        };
    }
}

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

enum Animation {
    Move {
        start: Vec3,
        end: Vec3,
        start_time: Instant,
        duration: Duration,
    },
    Rotate {
        start: f32,
        end: f32,
        start_time: Instant,
        duration: Duration,
    },
}

impl Animation {
    fn progress(&self) -> f32 {
        match *self {
            Animation::Move {
                start_time,
                duration,
                ..
            }
            | Animation::Rotate {
                start_time,
                duration,
                ..
            } => start_time.elapsed().as_secs_f32() / duration.as_secs_f32(),
        }
    }

    fn animate(&self, position: &mut Vec3, orientation: &mut f32) -> bool {
        let progress = self.progress();
        let animation_complete = progress >= 1.0;
        self.animate_progress(progress, position, orientation);
        animation_complete
    }

    fn complete(&self, position: &mut Vec3, orientation: &mut f32) {
        self.animate_progress(1.0, position, orientation);
    }

    fn animate_progress(&self, progress: f32, position: &mut Vec3, orientation: &mut f32) {
        let progress = progress.clamp(0.0, 1.0);
        match *self {
            Animation::Move { start, end, .. } => {
                *position = start.lerp(end, progress);
            }
            Animation::Rotate { start, end, .. } => {
                *orientation = if (start - end).abs() <= PI {
                    start.lerp(end, progress)
                } else if start < end {
                    (start + TAU).lerp(end, progress)
                } else {
                    (start - TAU).lerp(end, progress)
                };
            }
        }
    }
}

// TODO W.I.P.
// their meaning is clear from the context
#[allow(clippy::min_ident_chars)]
#[derive(Clone, Copy, Default)]
#[repr(u8)]
pub enum Orientation {
    /// positive x
    #[default]
    E = 0,
    /// +x, +y
    NE = 1,
    /// positive y
    N = 2,
    /// -x +y
    NW = 3,
    /// negative x
    W = 4,
    /// -x -y
    SW = 5,
    /// negative y
    S = 6,
    /// +x -y
    SE = 7,
}

impl Orientation {
    fn as_ivec3(self) -> IVec3 {
        match self {
            Orientation::E => IVec3::new(1, 0, 0),
            Orientation::NE => IVec3::new(1, 1, 0),
            Orientation::N => IVec3::new(0, 1, 0),
            Orientation::NW => IVec3::new(-1, 1, 0),
            Orientation::W => IVec3::new(-1, 0, 0),
            Orientation::SW => IVec3::new(-1, -1, 0),
            Orientation::S => IVec3::new(0, -1, 0),
            Orientation::SE => IVec3::new(1, -1, 0),
        }
    }

    fn angle(self) -> f32 {
        f32::from(self as u8) / 8.0 * TAU
    }
}

impl From<u8> for Orientation {
    fn from(value: u8) -> Self {
        match value & 0x07 {
            0 => Self::E,
            1 => Self::NE,
            2 => Self::N,
            3 => Self::NW,
            4 => Self::W,
            5 => Self::SW,
            6 => Self::S,
            7 => Self::SE,
            _ => unreachable!(),
        }
    }
}

impl AddAssign<u8> for Orientation {
    fn add_assign(&mut self, rhs: u8) {
        *self = (*self as u8).wrapping_add(rhs).into();
    }
}

impl SubAssign<u8> for Orientation {
    fn sub_assign(&mut self, rhs: u8) {
        *self = (*self as u8).wrapping_sub(rhs).into();
    }
}
