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
use runtimes::api::Value;
use runtimes::{api::Identifier, message::MessageId};

use crate::tile::LineSegment;

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

    current_command_id: Option<MessageId>,
    completed_command_ids: Vec<MessageId>,
}

impl GameState {
    pub(crate) fn update(&mut self) {
        self.tick.0 += 1;
        if self.robot.update() {
            self.complete_command();
        }
    }

    pub(crate) fn process_command(
        &mut self,
        command_id: MessageId,
        command: &Identifier,
        _arguments: &[Value],
    ) -> Result<(), String> {
        match command.0.as_ref() {
            "draw forward" => {
                self.move_forward(command_id, true)?;
            }
            "move forward" => {
                self.move_forward(command_id, false)?;
            }
            "turn left" => {
                self.turn_left(command_id);
            }
            "turn right" => {
                self.turn_right(command_id);
            }
            "color black" => {
                self.color(command_id, Vec3::new(0.2, 0.2, 0.2));
            }
            "color red" => {
                self.color(command_id, Vec3::new(0.8, 0.2, 0.2));
            }
            "color green" => {
                self.color(command_id, Vec3::new(0.2, 0.8, 0.2));
            }
            "color yellow" => {
                self.color(command_id, Vec3::new(0.8, 0.8, 0.2));
            }
            "color blue" => {
                self.color(command_id, Vec3::new(0.2, 0.2, 0.8));
            }
            "color magenta" => {
                self.color(command_id, Vec3::new(0.8, 0.0, 0.8));
            }
            "color cyan" => {
                self.color(command_id, Vec3::new(0.2, 0.8, 0.8));
            }
            "color white" => {
                self.color(command_id, Vec3::new(0.8, 0.8, 0.8));
            }
            other => {
                return Err(format!("Unknown Command: {other}"));
            }
        };
        Ok(())
    }

    fn renew_command(&mut self, command_id: MessageId) {
        if let Some(completed_id) = self.current_command_id.replace(command_id) {
            self.completed_command_ids.push(completed_id);
        }
    }

    fn complete_command(&mut self) {
        if let Some(completed_id) = self.current_command_id.take() {
            self.completed_command_ids.push(completed_id);
        }
    }

    fn turn_right(&mut self, command_id: MessageId) {
        self.turn(command_id, -1);
    }

    fn turn_left(&mut self, command_id: MessageId) {
        self.turn(command_id, 1);
    }

    fn turn(&mut self, command_id: MessageId, step: i8) {
        self.robot.complete_animation();
        self.renew_command(command_id);
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
            duration: Duration::from_millis(200),
        });
    }

    fn color(&mut self, command_id: MessageId, color: Vec3) {
        self.robot.complete_animation();
        self.completed_command_ids.push(command_id);

        let start_pos = self.robot.position;
        let start_index = Floor::to_index(start_pos).unwrap();
        self.floor.tiles[start_index].set_color(color);
        self.floor.tainted = self.tick;
    }

    fn move_forward(&mut self, command_id: MessageId, draw: bool) -> Result<(), String> {
        self.robot.complete_animation();
        self.renew_command(command_id);
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
            duration: Duration::from_millis(500),
        });

        Ok(())
    }

    pub(crate) fn drain_completed_commands(&mut self) -> impl Iterator<Item = MessageId> + '_ {
        self.completed_command_ids.drain(..)
    }

    // pub(crate) fn is_idle(&mut self) -> bool {
    //     self.robot.is_idle()
    // }
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

    fn update(&mut self) -> bool {
        if let Some(animation) = self.current_animation.as_ref() {
            if animation.animate(&mut self.animation_position, &mut self.animation_angle) {
                self.current_animation.take();
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
