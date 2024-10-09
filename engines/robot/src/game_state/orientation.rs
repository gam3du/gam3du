use std::{
    f32::consts::TAU,
    ops::{AddAssign, SubAssign},
};

use glam::IVec2;

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
    pub(crate) fn as_ivec2(self) -> IVec2 {
        match self {
            Orientation::E => IVec2::new(1, 0),
            Orientation::NE => IVec2::new(1, 1),
            Orientation::N => IVec2::new(0, 1),
            Orientation::NW => IVec2::new(-1, 1),
            Orientation::W => IVec2::new(-1, 0),
            Orientation::SW => IVec2::new(-1, -1),
            Orientation::S => IVec2::new(0, -1),
            Orientation::SE => IVec2::new(1, -1),
        }
    }

    pub(crate) fn angle(self) -> f32 {
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
