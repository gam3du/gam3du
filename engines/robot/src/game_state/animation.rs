use glam::{FloatExt, Vec3};
use std::{
    f32::consts::{PI, TAU},
    time::{Duration, Instant},
};

pub(crate) enum RobotAnimation {
    Move {
        start: Vec3,
        end: Vec3,
        start_time: Instant,
        duration: Duration,
    },
    Rotate {
        start: f32,
        end: f32,
        start_time: Instant,
        duration: Duration,
    },
}

impl RobotAnimation {
    fn progress(&self) -> f32 {
        match *self {
            RobotAnimation::Move {
                start_time,
                duration,
                ..
            }
            | RobotAnimation::Rotate {
                start_time,
                duration,
                ..
            } => start_time.elapsed().as_secs_f32() / duration.as_secs_f32(),
        }
    }

    pub(crate) fn animate(&self, position: &mut Vec3, orientation: &mut f32) -> bool {
        let progress = self.progress();
        let animation_complete = progress >= 1.0;
        self.animate_progress(progress, position, orientation);
        animation_complete
    }

    pub(crate) fn complete(&self, position: &mut Vec3, orientation: &mut f32) {
        self.animate_progress(1.0, position, orientation);
    }

    fn animate_progress(&self, progress: f32, position: &mut Vec3, orientation: &mut f32) {
        let progress = progress.clamp(0.0, 1.0);
        match *self {
            RobotAnimation::Move { start, end, .. } => {
                *position = start.lerp(end, progress);
            }
            RobotAnimation::Rotate { start, end, .. } => {
                *orientation = if (start - end).abs() <= PI {
                    start.lerp(end, progress)
                } else if start < end {
                    (start + TAU).lerp(end, progress)
                } else {
                    (start - TAU).lerp(end, progress)
                };
            }
        }
    }
}
