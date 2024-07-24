use super::tile::{tile, LinePattern, Tile};

pub(super) struct Floor {
    pub(super) tiles: Vec<Tile>,
    pub(super) tainted: bool,
}

impl Floor {
    // `time` will be moved to global scope anyway
    #[allow(clippy::similar_names)]
    #[must_use]
    pub(super) fn new() -> Self {
        let tiles = Self::create_tiles();

        Self {
            tiles,
            tainted: false,
        }
    }

    fn tile_count(&self) -> u32 {
        u32::try_from(self.tiles.len()).unwrap()
    }

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
}
