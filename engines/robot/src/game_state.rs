mod animation;
mod floor;
mod orientation;
mod robot;

use crate::{api::EngineApi, events::EventRegistries, tile::LineSegment};
use animation::RobotAnimation;
use floor::Floor;
use glam::{IVec2, UVec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};
pub(crate) use orientation::Orientation;
pub(crate) use robot::Robot;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub(crate) struct Tick(pub(crate) u64);

/// Contains every information about the current state of the game.
/// This is what needs to be stored/loaded if the game need to be suspended.
// #[derive(Default)]
pub struct GameState {
    /// ever increasing counter representing the number of game loop iterations so far
    pub(crate) tick: Tick,
    /// current state of the robot
    pub(crate) robot: Robot,
    /// current state of the floor
    pub floor: Floor,

    pub(crate) event_registries: EventRegistries,
}

impl GameState {
    pub(crate) fn bogus() -> Self {
        Self {
            tick: Tick::default(),
            robot: Robot::default(),
            floor: Floor::new((0, 0)),
            event_registries: EventRegistries::default(),
        }
    }

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

    pub fn _set_height(&mut self, height: f32) {
        self.robot.complete_animation();

        let start_pos = self.robot.position;
        let start_index = self.floor.to_index(start_pos.xy()).unwrap();
        self.floor.tiles[start_index].pos[2] = height;
        self.floor.tainted = self.tick;
        self.robot.animation_position[2] = height;
    }

    pub fn _paint_tile(&mut self) {
        self.robot.complete_animation();

        let start_pos = self.robot.position;
        let start_index = self.floor.to_index(start_pos.xy()).unwrap();
        self.floor.tiles[start_index].set_color(self.robot.color);
        self.floor.tainted = self.tick;
    }

    fn _move_forward(&mut self, draw: bool, duration: Duration) -> Result<(), String> {
        self.robot.complete_animation();

        let segment = LineSegment::from(self.robot.orientation);

        let offset = self.robot.orientation.as_ivec2();

        let start_pos = self.robot.position.xy();
        let start_index = self.floor.to_index(start_pos)?;
        let end_pos = start_pos + offset;
        let end_index = self.floor.to_index(end_pos)?;

        if draw {
            self.floor.tiles[start_index].line_pattern |= segment;

            // draw adjacent diagonal corners
            if offset.x != 0 && offset.y != 0 {
                let index0 = self.floor.to_index(start_pos + IVec2::new(offset.x, 0))?;
                self.floor.tiles[index0].line_pattern |= segment.get_x_corner().unwrap();
                let index1 = self.floor.to_index(start_pos + IVec2::new(0, offset.y))?;
                self.floor.tiles[index1].line_pattern |= -segment.get_x_corner().unwrap();
            }

            self.floor.tiles[end_index].line_pattern |= -segment;
            self.floor.tainted = self.tick;
        }

        self.robot.position.x = end_pos.x;
        self.robot.position.y = end_pos.y;
        let animation_start = self.robot.animation_position;
        let animation_end = self.floor.tiles[end_index].center_pos();
        let animation_via = (
            animation_start.xy().midpoint(animation_end.xy()),
            animation_start.z.max(animation_end.z),
        )
            .into();
        self.robot.current_animation = Some(RobotAnimation::Move {
            start: animation_start,
            via: animation_via,
            end: animation_end,
            start_time: Instant::now(),
            duration,
        });

        Ok(())
    }

    pub fn new(floor_size: impl Into<UVec2>) -> Self {
        // let robot = Robot {
        //     position: UVec3::from((floor_size / 2, 0)).as_ivec3(),
        //     ..Robot::default()
        // };

        Self {
            tick: Tick::default(),
            robot: Robot::default(),
            floor: Floor::new(floor_size.into()),
            event_registries: EventRegistries::default(),
        }
    }

    // #[must_use]
    // pub(crate) fn is_idle(&mut self) -> bool {
    //     self.robot.is_idle()
    // }
}

impl EngineApi for GameState {
    fn set_height(&mut self, height: f32) {
        self._set_height(height);
    }

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
