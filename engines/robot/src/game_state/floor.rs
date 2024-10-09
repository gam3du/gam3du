use super::Tick;
use crate::tile::{tile, LinePattern, Tile};
use glam::{IVec2, UVec2};
use log::trace;
use rand::{thread_rng, Rng};

pub struct Floor {
    pub(crate) tiles: Vec<Tile>,
    pub(crate) origin: IVec2,
    pub(crate) size: UVec2,
    pub(crate) tainted: Tick,
}

impl Floor {
    pub(crate) fn new(size: impl Into<UVec2>) -> Self {
        let size = size.into();
        let mut tiles = Vec::new();
        let pos = -size.as_vec2() / 2.0;
        for y in 0..size.y {
            for x in 0..size.x {
                let xy = pos + UVec2::new(x, y).as_vec2();
                let line_pattern = 0; //thread_rng.gen();
                let height = thread_rng().r#gen::<f32>() * 0.2;
                tiles.push(tile((xy, height), LinePattern(line_pattern)));
            }
        }

        Self {
            tiles,
            size,
            origin: size.as_ivec2() / 2,
            tainted: Tick::default(),
        }
    }

    pub(crate) fn to_index(&self, pos: impl Into<IVec2>) -> Result<usize, String> {
        let pos = pos.into() + self.origin;
        trace!("to_index({pos})");
        let x = usize::try_from(pos.x).map_err(|_err| "robot left the plane at -x".to_owned())?;
        let y = usize::try_from(pos.y).map_err(|_err| "robot left the plane at -y".to_owned())?;
        let size_x = self.size.x as usize;
        let size_y = self.size.y as usize;

        if x >= size_x {
            return Err("robot left the plane at +x".into());
        }
        if y >= size_y {
            return Err("robot left the plane at +y".into());
        }

        Ok(x + size_x * y)
    }
}
