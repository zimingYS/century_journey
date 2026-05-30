pub mod noise;

use bevy::prelude::*;
use crate::world::chunk::ChunkData;
use crate::world::generation::noise::{GenerationBlockIds, TerrainGenerator};

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

    pub fn generate_chunk_data(&self, chunk_pos: IVec3, block_ids: GenerationBlockIds) -> ChunkData {
        self.generator.generate_chunk(chunk_pos, block_ids)
    }
}