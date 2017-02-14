use game::*;

#[derive(Clone, Copy, Debug, RustcEncodable, RustcDecodable)]
pub struct LevelSwitch {
    pub terrain_type: TerrainType,
}

impl LevelSwitch {
    pub fn new(terrain_type: TerrainType) -> Self {
        LevelSwitch {
            terrain_type: terrain_type,
        }
    }
}
