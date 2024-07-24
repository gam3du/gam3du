use std::time::Instant;

use glam::{IVec3, Vec3};

use super::{
    camera::Camera, floor_renderer::FloorRenderer, projection::Projection,
    robot_renderer::RobotRenderer, tile::Tile, Animation, Orientation,
};

pub(super) fn elapsed_as_vec(start_time: Instant) -> [u32; 2] {
    let elapsed = start_time.elapsed();
    let seconds = u32::try_from(elapsed.as_secs()).unwrap();
    let subsec_nanos = u64::from(elapsed.subsec_nanos());
    // map range of nanoseconds to value range of u32 with rounding
    let subseconds = ((subsec_nanos << u32::BITS) + 500_000_000) / 1_000_000_000;

    [seconds, u32::try_from(subseconds).unwrap()]
}

pub(crate) struct RendererState {
    pub(crate) camera: Camera,
    pub(crate) projection: Projection,
    pub(crate) animation_position: Vec3,
    pub(crate) animation_angle: f32,
    pub(crate) position: IVec3,
    pub(crate) orientation: Orientation,
    pub(crate) current_animation: Option<Animation>,
    pub(super) tiles: Vec<Tile>,
    pub(super) tiles_tainted: bool,
}

pub(crate) struct Renderer {
    pub(crate) robot_renderer: RobotRenderer,
    pub(crate) floor_renderer: FloorRenderer,
}

impl Renderer {
    pub(crate) fn render<'pipeline>(
        &'pipeline mut self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'pipeline>,
        start_time: Instant,
        state: &RendererState,
    ) {
        self.floor_renderer
            .render(queue, render_pass, state, start_time);
        self.robot_renderer
            .render(queue, render_pass, state, start_time);

        state.tiles_tainted = false;
    }
}
