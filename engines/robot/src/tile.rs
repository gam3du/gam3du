#![expect(
    clippy::allow_attributes_without_reason,
    reason = "false positives for Pod/Zeroable macros"
)]
use std::ops;

use bytemuck::{Pod, Zeroable};

use crate::game_state::Orientation;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub(super) struct Tile {
    pub(super) pos: [f32; 4],
    pub(super) line_pattern: LinePattern,
}

pub(super) fn tile(pos: [f32; 3], line_pattern: LinePattern) -> Tile {
    Tile {
        pos: [pos[0], pos[1], pos[2], 1.0],
        line_pattern,
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, Default)]
pub(super) struct LinePattern(pub(super) u32);

impl ops::BitOrAssign<LineSegment> for LinePattern {
    fn bitor_assign(&mut self, rhs: LineSegment) {
        self.0 |= 1 << rhs as u32;
    }
}

#[derive(Clone, Copy)]
#[expect(
    clippy::min_ident_chars,
    reason = "their meaning is clear from the context"
)]
pub(super) enum LineSegment {
    /// positive x
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
    /// +x, +y
    NECorner = 9,
    /// -x +y
    NWCorner = 11,
    /// -x -y
    SWCorner = 13,
    /// +x -y
    SECorner = 15,
}

impl From<Orientation> for LineSegment {
    fn from(value: Orientation) -> Self {
        match value {
            Orientation::E => Self::E,
            Orientation::NE => Self::NE,
            Orientation::N => Self::N,
            Orientation::NW => Self::NW,
            Orientation::W => Self::W,
            Orientation::SW => Self::SW,
            Orientation::S => Self::S,
            Orientation::SE => Self::SE,
        }
    }
}

impl LineSegment {
    pub(crate) fn get_x_corner(self) -> Option<LineSegment> {
        match self {
            Self::NE => Some(Self::NWCorner),
            Self::NW => Some(Self::NECorner),
            Self::SW => Some(Self::SECorner),
            Self::SE => Some(Self::SWCorner),
            _ => None,
        }
    }
}

impl ops::Neg for LineSegment {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::E => Self::W,
            Self::NE => Self::SW,
            Self::N => Self::S,
            Self::NW => Self::SE,
            Self::W => Self::E,
            Self::SW => Self::NE,
            Self::S => Self::N,
            Self::SE => Self::NW,
            Self::NECorner => Self::SWCorner,
            Self::NWCorner => Self::SECorner,
            Self::SWCorner => Self::NECorner,
            Self::SECorner => Self::NWCorner,
        }
    }
}
