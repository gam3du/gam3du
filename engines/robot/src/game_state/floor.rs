use super::Tick;
use crate::tile::{tile, LinePattern, Tile};
use glam::IVec3;

pub(crate) struct Floor {
    pub(crate) tiles: Vec<Tile>,
    pub(crate) tainted: Tick,
}

impl Floor {
    fn create_tiles() -> Vec<Tile> {
        let mut vertex_data = Vec::new();
        for y in -5_i16..5 {
            let bottom = f32::from(y);
            for x in -5_i16..5 {
                let left = f32::from(x);
                let line_pattern = 0; //thread_rng.gen();
                vertex_data.push(tile([left, bottom, 0.0], LinePattern(line_pattern)));
            }
        }

        vertex_data
    }

    pub(crate) fn to_index(pos: IVec3) -> Result<usize, String> {
        let x =
            usize::try_from(pos.x + 5).map_err(|_err| "robot left the plane at -x".to_owned())?;
        let y =
            usize::try_from(pos.y + 5).map_err(|_err| "robot left the plane at -y".to_owned())?;
        if x >= 10 {
            return Err("robot left the plane at +x".into());
        }
        if y >= 10 {
            return Err("robot left the plane at +y".into());
        }

        Ok(x + 10 * y)
    }
}

impl Default for Floor {
    fn default() -> Self {
        let tiles = Self::create_tiles();

        Self {
            tiles,
            tainted: Tick::default(),
        }
    }
}
