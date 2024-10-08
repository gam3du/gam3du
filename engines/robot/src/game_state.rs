mod animation;
mod floor;
mod orientation;
mod robot;

use crate::{api::EngineApi, events::EventRegistries, tile::LineSegment};
use animation::RobotAnimation;
use floor::Floor;
use glam::{IVec3, Vec3};
pub(crate) use orientation::Orientation;
pub(crate) use robot::Robot;
use std::time::{Duration, Instant};

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

    fn _turn(&mut self, steps_ccw: i8, duration: Duration) {
        self.robot.complete_animation();
        #[expect(clippy::cast_sign_loss, reason = "TODO make this less cumbersome")]
        if steps_ccw < 0 {
            self.robot.orientation -= -steps_ccw as u8;
        } else {
            self.robot.orientation += steps_ccw as u8;
        }
        self.robot.current_animation = Some(RobotAnimation::Rotate {
            start: self.robot.animation_angle,
            end: self.robot.orientation.angle(),
            start_time: Instant::now(),
            duration,
        });
    }

    pub fn _robot_color_rgb(&mut self, color: Vec3) {
        self.robot.color = color;
    }

    pub fn _paint_tile(&mut self) {
        self.robot.complete_animation();

        let start_pos = self.robot.position;
        let start_index = Floor::to_index(start_pos).unwrap();
        self.floor.tiles[start_index].set_color(self.robot.color);
        self.floor.tainted = self.tick;
    }

    fn _move_forward(&mut self, draw: bool, duration: Duration) -> Result<(), String> {
        self.robot.complete_animation();
        let segment = LineSegment::from(self.robot.orientation);

        let offset = self.robot.orientation.as_ivec3();

        let start_pos = self.robot.position;
        let end_pos = start_pos + offset;
        let end_index = Floor::to_index(end_pos)?;

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
    fn move_forward(&mut self, draw: bool, duration: u64) -> bool {
        self._move_forward(draw, Duration::from_millis(duration))
            .is_ok()
    }

    fn turn(&mut self, steps_ccw: i8, duration: u64) {
        self._turn(steps_ccw, Duration::from_millis(duration));
    }

    fn robot_color_rgb(&mut self, red: f32, green: f32, blue: f32) {
        self._robot_color_rgb(Vec3::new(red, green, blue));
    }

    fn paint_tile(&mut self) {
        self._paint_tile();
    }
}
