use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub look_at: Vec3,
    pub up: Vec3,
}

impl Camera {
    #[must_use]
    pub fn new(eye: Vec3, center: Vec3) -> Self {
        Self {
            position: eye,
            look_at: center,
            up: Vec3::Z,
        }
    }

    #[must_use]
    pub fn matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.look_at, self.up)
    }
}
