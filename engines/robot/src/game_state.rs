use std::{
    f32::consts::{PI, TAU},
    ops::{AddAssign, SubAssign},
    time::{Duration, Instant},
};

use bindings::api::Identifier;
use glam::{FloatExt, IVec3, Vec3};
use log::error;

use crate::tile::{tile, LinePattern, LineSegment, Tile};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub(crate) struct Tick(pub(crate) u64);

/// Contains every information about the current state of the game.
/// This is what needs to be stored/loaded if the game need to be suspended.
pub(crate) struct GameState {
    /// ever increasing counter representing the number of game loop iterations so far
    pub(crate) tick: Tick,
    /// current state of the robot
    pub(crate) robot: Robot,
    /// current state of the floor
    pub(crate) floor: Floor,
}

impl GameState {
    pub(crate) fn update(&mut self) {
        self.tick.0 += 1;
        self.robot.update();
    }

    pub(crate) fn process_command(&mut self, command: &Identifier) {
        self.robot.current_animation = match command.0.as_str() {
            "move forward" => {
                self.robot.complete_animation();
                let segment = LineSegment::from(self.robot.orientation);

                // TODO make this a safe function
                #[allow(clippy::cast_sign_loss)]
                let start_index =
                    (self.robot.position.y * 10 + self.robot.position.x + 55) as usize;
                self.floor.tiles[start_index].line_pattern |= segment;

                let offset = self.robot.orientation.as_ivec3();
                if offset.x != 0 && offset.y != 0 {
                    // TODO make this a safe function
                    #[allow(clippy::cast_sign_loss)]
                    let index0 = (self.robot.position.y * 10
                        + (self.robot.position.x + offset.x)
                        + 55) as usize;
                    self.floor.tiles[index0].line_pattern |= segment.get_x_corner().unwrap();

                    // TODO make this a safe function
                    #[allow(clippy::cast_sign_loss)]
                    let index1 = ((self.robot.position.y + offset.y) * 10
                        + self.robot.position.x
                        + 55) as usize;
                    self.floor.tiles[index1].line_pattern |= -segment.get_x_corner().unwrap();
                }

                self.robot.position += offset;

                // TODO make this a safe function
                #[allow(clippy::cast_sign_loss)]
                let end_index = (self.robot.position.y * 10 + self.robot.position.x + 55) as usize;
                self.floor.tiles[end_index].line_pattern |= -segment;
                self.floor.tainted = self.tick;

                Some(Animation::Move {
                    start: self.robot.animation_position,
                    end: self.robot.position.as_vec3() + Vec3::new(0.5, 0.5, 0.0),
                    start_time: Instant::now(),
                    duration: Duration::from_millis(1_000),
                })
            }
            "turn left" => {
                self.robot.complete_animation();
                self.robot.orientation += 1;
                Some(Animation::Rotate {
                    start: self.robot.animation_angle,
                    end: self.robot.orientation.angle(),
                    start_time: Instant::now(),
                    duration: Duration::from_millis(1_000),
                })
            }
            "turn right" => {
                self.robot.complete_animation();
                self.robot.orientation -= 1;
                Some(Animation::Rotate {
                    start: self.robot.animation_angle,
                    end: self.robot.orientation.angle(),
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

    pub(crate) fn is_idle(&mut self) -> bool {
        self.robot.is_idle()
    }
}

pub(crate) struct Robot {
    pub(crate) animation_position: Vec3,
    pub(crate) animation_angle: f32,
    position: IVec3,
    orientation: Orientation,
    current_animation: Option<Animation>,
}

impl Robot {
    #[must_use]
    pub(crate) fn new() -> Self {
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

    #[must_use]
    pub(crate) fn is_idle(&self) -> bool {
        self.current_animation.is_none()
    }

    fn complete_animation(&mut self) {
        if let Some(current_animation) = self.current_animation.take() {
            current_animation.complete(&mut self.animation_position, &mut self.animation_angle);
        }
    }

    fn update(&mut self) {
        if let Some(animation) = self.current_animation.as_ref() {
            if animation.animate(&mut self.animation_position, &mut self.animation_angle) {
                self.current_animation.take();
            }
        };
    }
}

pub(super) struct Floor {
    pub(super) tiles: Vec<Tile>,
    pub(super) tainted: Tick,
}

impl Floor {
    // `time` will be moved to global scope anyway
    #[allow(clippy::similar_names)]
    #[must_use]
    pub(super) fn new() -> Self {
        let tiles = Self::create_tiles();

        Self {
            tiles,
            tainted: Tick::default(),
        }
    }

    fn tile_count(&self) -> u32 {
        u32::try_from(self.tiles.len()).unwrap()
    }

    fn create_tiles() -> Vec<Tile> {
        let mut vertex_data = Vec::new();
        for y in -5_i16..5 {
            let bottom = f32::from(y);
            for x in -5_i16..5 {
                let left = f32::from(x);
                let line_pattern = 0; //thread_rng.gen();
                vertex_data.push(tile([left, bottom, 0.0], LinePattern(line_pattern)));
            }
        }

        vertex_data
    }
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
pub(crate) enum Orientation {
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
