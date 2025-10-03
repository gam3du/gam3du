use glam::{IVec3, Vec3};
use tracing::debug;

use crate::events::EventRegistries;

use super::{animation::RobotAnimation, orientation::Orientation};

pub(crate) struct Robot {
    pub(crate) animation_position: Vec3,
    pub(crate) animation_angle: f32,
    pub(crate) position: IVec3,
    pub(crate) color: Vec3,
    pub(crate) orientation: Orientation,
    pub(crate) current_animation: Option<RobotAnimation>,
}

impl Robot {
    // #[must_use]
    // pub(crate) fn is_idle(&self) -> bool {
    //     self.current_animation.is_none()
    // }

    pub(crate) fn complete_animation(&mut self, event_registries: &mut EventRegistries) {
        if let Some(animation) = self.current_animation.take() {
            debug!("short-circuiting running animation");
            animation.complete(&mut self.animation_position, &mut self.animation_angle);
            debug!("notifying `robot_stopped` listeners");
            event_registries.robot_stopped.notify();
        } else {
            debug!("no existing animation to short-circuit");
        }
    }

    pub(crate) fn update(&mut self, event_registries: &mut EventRegistries) -> bool {
        if let Some(animation) = self.current_animation.as_ref()
            && animation.animate(&mut self.animation_position, &mut self.animation_angle)
        {
            self.current_animation.take();
            debug!("notifying `robot_stopped` listeners");
            event_registries.robot_stopped.notify();
            return true;
        }
        false
    }
}

impl Default for Robot {
    fn default() -> Self {
        let orientation = Orientation::default();
        let position = IVec3::new(0, 0, 0);

        Self {
            color: Vec3::new(0.3, 0.3, 0.3),
            animation_position: position.as_vec3() + Vec3::new(0.5, 0.5, 0.0),
            current_animation: None,
            animation_angle: orientation.angle(),
            orientation,
            position,
        }
    }
}
