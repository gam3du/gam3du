mod animation;
mod floor;
mod orientation;
mod robot;

use crate::{api::EngineApi, events::EventRegistries, tile::LineSegment};
use animation::RobotAnimation;
use floor::Floor;
use glam::{IVec2, UVec2, Vec3, Vec3Swizzles};
pub(crate) use orientation::Orientation;
pub(crate) use robot::Robot;
use std::sync::{Arc, RwLock};
use web_time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub(crate) struct Tick(pub(crate) u64);

pub type SharedGameState = Arc<RwLock<Box<GameState>>>;

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

    #[must_use]
    pub fn into_shared(self) -> SharedGameState {
        Arc::new(RwLock::new(Box::new(self)))
    }

    pub(crate) fn update(&mut self) {
        self.tick.0 += 1;
        self.robot.update(&mut self.event_registries);
    }

    fn turn_(&mut self, steps_ccw: i8, duration: Duration) {
        self.robot.complete_animation(&mut self.event_registries);
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

    pub fn robot_color_rgb_(&mut self, color: Vec3) {
        self.robot.color = color;
    }

    pub fn set_height_(&mut self, height: f32) {
        self.robot.complete_animation(&mut self.event_registries);

        let start_pos = self.robot.position;
        let start_index = self.floor.to_index(start_pos.xy()).unwrap();
        self.floor.tiles[start_index].pos[2] = height;
        self.floor.tainted = self.tick;
        self.robot.animation_position[2] = height;
    }

    pub fn paint_tile_(&mut self) {
        self.robot.complete_animation(&mut self.event_registries);

        let start_pos = self.robot.position;
        let start_index = self.floor.to_index(start_pos.xy()).unwrap();
        self.floor.tiles[start_index].set_color(self.robot.color);
        self.floor.tainted = self.tick;
    }

    fn move_forward_(&mut self, draw: bool, duration: Duration) -> Result<(), String> {
        self.robot.complete_animation(&mut self.event_registries);

        let segment = LineSegment::from(self.robot.orientation);

        let offset = self.robot.orientation.as_ivec2();

        let start_pos = self.robot.position.xy();
        let start_index = self.floor.to_index(start_pos)?;
        let end_pos = start_pos + offset;
        let end_index = self.floor.to_index(end_pos)?;

        let animation_start = self.robot.animation_position;
        let animation_end = self.floor.tiles[end_index].center_pos();
        let animation_via = (
            animation_start.xy().midpoint(animation_end.xy()),
            animation_start.z.max(animation_end.z),
        )
            .into();

        if (animation_start.z - animation_end.z).abs() >= 0.5 {
            return Err("too high".into());
        }

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
        self.robot.current_animation = Some(RobotAnimation::Move {
            start: animation_start,
            via: animation_via,
            end: animation_end,
            start_time: Instant::now(),
            duration,
        });

        Ok(())
    }

    fn jump_(&mut self, duration: Duration) -> Result<(), String> {
        self.robot.complete_animation(&mut self.event_registries);

        let offset = self.robot.orientation.as_ivec2();

        let start_pos = self.robot.position.xy();
        let end_pos = start_pos + offset;
        let end_index = self.floor.to_index(end_pos)?;

        let animation_start = self.robot.animation_position;
        let animation_end = self.floor.tiles[end_index].center_pos();

        if (animation_start.z - animation_end.z).abs() >= 1.0 {
            return Err("too high".into());
        }

        self.robot.position.x = end_pos.x;
        self.robot.position.y = end_pos.y;
        self.robot.current_animation = Some(RobotAnimation::Jump {
            start: animation_start,
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
        self.set_height_(height);
    }

    fn move_forward(&mut self, draw: bool, duration: u64) -> bool {
        self.move_forward_(draw, Duration::from_millis(duration))
            .is_ok()
    }

    fn jump(&mut self, duration: u64) -> bool {
        self.jump_(Duration::from_millis(duration)).is_ok()
    }

    fn turn(&mut self, steps_ccw: i8, duration: u64) {
        self.turn_(steps_ccw, Duration::from_millis(duration));
    }

    fn robot_color_rgb(&mut self, red: f32, green: f32, blue: f32) {
        self.robot_color_rgb_(Vec3::new(red, green, blue));
    }

    fn paint_tile(&mut self) {
        self.paint_tile_();
    }
}
