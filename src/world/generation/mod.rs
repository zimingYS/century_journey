pub mod noise;

use bevy::prelude::*;
use crate::world::chunk::ChunkData;
use crate::world::generation::noise::TerrainGenerator;

#[derive(Resource)]
pub struct WorldGenerator {
    pub seed: u32,
    generator: TerrainGenerator,
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            generator: TerrainGenerator::new(seed),
        }
    }

    pub fn generate_chunk_data(&self, chunk_pos: IVec3) -> ChunkData {
        self.generator.generate_chunk(chunk_pos)
    }
}