mod animation;
mod floor;

use std::{
    f32::consts::TAU,
    ops::{AddAssign, SubAssign},
    time::{Duration, Instant},
};

use animation::RobotAnimation;
use floor::Floor;
use glam::{IVec3, Vec3};
use log::debug;

use crate::{api::EngineApi, events::EventRegistries, tile::LineSegment};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub(crate) struct Tick(pub(crate) u64);

/// Contains every information about the current state of the game.
/// This is what needs to be stored/loaded if the game need to be suspended.
#[derive(Default)]
pub struct GameState {
    /// ever increasing counter representing the number of game loop iterations so far
    pub(crate) tick: Tick,
    /// current state of the robot
    pub(crate) robot: Robot,
    /// current state of the floor
    pub(crate) floor: Floor,

    pub(crate) event_registries: EventRegistries,
}

impl GameState {
    pub(crate) fn update(&mut self) {
        self.tick.0 += 1;
        self.robot.update(&mut self.event_registries);
    }

    fn turn(&mut self, step: i8, duration: Duration) {
        self.robot.complete_animation();
        #[expect(clippy::cast_sign_loss, reason = "TODO make this less cumbersome")]
        if step < 0 {
            self.robot.orientation -= -step as u8;
        } else {
            self.robot.orientation += step as u8;
        }
        self.robot.current_animation = Some(RobotAnimation::Rotate {
            start: self.robot.animation_angle,
            end: self.robot.orientation.angle(),
            start_time: Instant::now(),
            duration,
        });
    }

    pub fn _color_rgb(&mut self, color: Vec3) {
        self.robot.complete_animation();

        let start_pos = self.robot.position;
        let start_index = Floor::to_index(start_pos).unwrap();
        self.floor.tiles[start_index].set_color(color);
        self.floor.tainted = self.tick;
    }

    fn _move_forward(&mut self, draw: bool, duration: Duration) -> Result<(), String> {
        self.robot.complete_animation();
        let segment = LineSegment::from(self.robot.orientation);

        let offset = self.robot.orientation.as_ivec3();

        let start_pos = self.robot.position;
        let end_pos = start_pos + offset;

        if draw {
            let start_index = Floor::to_index(start_pos)?;

            self.floor.tiles[start_index].line_pattern |= segment;

            // draw adjacent diagonal corners
            if offset.x != 0 && offset.y != 0 {
                let index0 = Floor::to_index(start_pos + IVec3::new(offset.x, 0, 0))?;
                self.floor.tiles[index0].line_pattern |= segment.get_x_corner().unwrap();
                let index1 = Floor::to_index(start_pos + IVec3::new(0, offset.y, 0))?;
                self.floor.tiles[index1].line_pattern |= -segment.get_x_corner().unwrap();
            }

            let end_index = Floor::to_index(end_pos)?;

            self.floor.tiles[end_index].line_pattern |= -segment;
            self.floor.tainted = self.tick;
        }

        self.robot.position = end_pos;

        self.robot.current_animation = Some(RobotAnimation::Move {
            start: self.robot.animation_position,
            end: self.robot.position.as_vec3() + Vec3::new(0.5, 0.5, 0.0),
            start_time: Instant::now(),
            duration,
        });

        Ok(())
    }

    // #[must_use]
    // pub(crate) fn is_idle(&mut self) -> bool {
    //     self.robot.is_idle()
    // }
}

impl EngineApi for GameState {
    fn move_forward(&mut self, duration: u64) -> Result<(), String> {
        self._move_forward(false, Duration::from_millis(duration))
    }

    fn draw_forward(&mut self, duration: u64) -> Result<(), String> {
        self._move_forward(true, Duration::from_millis(duration))
    }

    fn turn_left(&mut self, duration: u64) {
        self.turn(1, Duration::from_millis(duration));
    }

    fn turn_right(&mut self, duration: u64) {
        self.turn(-1, Duration::from_millis(duration));
    }

    fn color_rgb(&mut self, red: f32, green: f32, blue: f32) {
        self._color_rgb(Vec3::new(red, green, blue));
    }
}

pub(crate) struct Robot {
    pub(crate) animation_position: Vec3,
    pub(crate) animation_angle: f32,
    position: IVec3,
    orientation: Orientation,
    current_animation: Option<RobotAnimation>,
}

impl Robot {
    // #[must_use]
    // pub(crate) fn is_idle(&self) -> bool {
    //     self.current_animation.is_none()
    // }

    fn complete_animation(&mut self) {
        if let Some(current_animation) = self.current_animation.take() {
            debug!("short-circuiting running animation");
            current_animation.complete(&mut self.animation_position, &mut self.animation_angle);
        } else {
            debug!("no existing animation to short-circuit");
        }
    }

    fn update(&mut self, event_registries: &mut EventRegistries) -> bool {
        if let Some(animation) = self.current_animation.as_ref() {
            if animation.animate(&mut self.animation_position, &mut self.animation_angle) {
                self.current_animation.take();
                event_registries.robot_stopped.notify();
                return true;
            }
        };
        false
    }
}

impl Default for Robot {
    fn default() -> Self {
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
}

// TODO W.I.P.
#[expect(
    clippy::min_ident_chars,
    reason = "their meaning is clear from the context"
)]
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
