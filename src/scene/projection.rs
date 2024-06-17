use std::ops::Range;

use glam::Mat4;

pub(super) enum Projection {
    Perspective {
        surface_width: u32,
        surface_height: u32,
        fov: f32,
        z_range: Range<f32>,
    },
}

impl Projection {
    #[must_use]
    pub(super) fn new_perspective(
        (surface_width, surface_height): (u32, u32),
        fov: f32,
        z_range: Range<f32>,
    ) -> Self {
        Self::Perspective {
            surface_width,
            surface_height,
            fov,
            z_range,
        }
    }

    fn surface_width(&self) -> u32 {
        match *self {
            Projection::Perspective { surface_width, .. } => surface_width,
        }
    }

    fn surface_height(&self) -> u32 {
        match *self {
            Projection::Perspective { surface_height, .. } => surface_height,
        }
    }

    fn near(&self) -> f32 {
        match *self {
            Projection::Perspective { ref z_range, .. } => z_range.start,
        }
    }

    fn far(&self) -> f32 {
        match *self {
            Projection::Perspective { ref z_range, .. } => z_range.end,
        }
    }

    fn fov(&self) -> f32 {
        match *self {
            Projection::Perspective { fov, .. } => fov,
        }
    }

    fn aspect_ratio(&self) -> f32 {
        self.surface_width() as f32 / self.surface_height() as f32
    }

    #[must_use]
    pub(super) fn matrix(&self) -> Mat4 {
        glam::Mat4::perspective_rh(self.fov(), self.aspect_ratio(), self.near(), self.far())
    }

    pub(crate) fn set_surface_dimensions(
        &mut self,
        (new_surface_width, new_surface_height): (u32, u32),
    ) {
        match *self {
            Projection::Perspective {
                ref mut surface_width,
                ref mut surface_height,
                ..
            } => {
                *surface_width = new_surface_width;
                *surface_height = new_surface_height;
            }
        }
    }
}
