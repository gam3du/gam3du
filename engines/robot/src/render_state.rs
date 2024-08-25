use std::time::Instant;

use glam::Vec3;

use crate::{
    camera::Camera,
    game_state::{GameState, Tick},
    tile::Tile,
};

const CAMERA_POS: Vec3 = Vec3::new(-2.0, -3.0, 2.0);

/// Contains every game information that is required to render the scene.
pub struct RenderState {
    /// Time stamp of the creation.
    /// Used to animate visual effects that run independently of the game loop.
    /// The current time can be retrieved by calling `start_time.elapsed()`
    /// (e.g. water, particles, ... even while the game is paused.)
    pub(crate) start_time: Instant,

    pub(crate) camera: Camera,
    // pub projection: Projection,
    /// current position of the robot
    pub(crate) animation_position: Vec3,
    /// current orientation of the robot
    pub(crate) animation_angle: f32,
    // pub(crate) position: IVec3,
    // pub(crate) orientation: Orientation,
    // pub(crate) current_animation: Option<Animation>,
    // pub(super) tiles: Vec<Tile>,
    // pub(super) tiles_tainted: bool,
    /// Tick of when we copied the `tiles` from [`GameState::floor`].
    pub(crate) tiles_tick: Tick,
    /// Our local copy of the game loop's `floor.tiles` field
    pub(crate) tiles: Vec<Tile>,
    // pub robot_renderer: RobotRenderer,
    // pub floor_renderer: FloorRenderer,
}

impl RenderState {
    #[must_use]
    pub fn new(game_state: &GameState) -> Self {
        let camera = Camera::new(CAMERA_POS, Vec3::ZERO);

        Self {
            start_time: Instant::now(),
            camera,
            animation_position: game_state.robot.animation_position,
            animation_angle: game_state.robot.animation_angle,
            tiles_tick: game_state.tick,
            tiles: game_state.floor.tiles.clone(),
        }
    }

    pub fn update(&mut self, game_state: &GameState) {
        let time = self.start_time.elapsed();

        let (dy, dx) = (time.as_secs_f32() * 0.1).sin_cos();
        self.camera.position = CAMERA_POS + Vec3::new(dx * 0.3, -dy * 0.3, 0.0);

        self.animation_position = game_state.robot.animation_position;
        self.animation_angle = game_state.robot.animation_angle;
        if game_state.floor.tainted > self.tiles_tick {
            self.tiles_tick = game_state.floor.tainted;
            self.tiles.clone_from(&game_state.floor.tiles);
        }
    }
}
