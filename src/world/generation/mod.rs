pub mod noise;
pub mod context;
pub mod climate;
pub mod biome;
pub mod terrain;
pub mod structure;
pub mod pipeline;

use bevy::prelude::*;
use crate::world::chunk::ChunkData;
use crate::world::generation::noise::GenerationBlockIds;
use crate::world::generation::pipeline::GenerationPipeline;

#[derive(Resource)]
pub struct WorldGenerator {
    pub seed: u32,
    pipeline: GenerationPipeline,
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            pipeline: GenerationPipeline::new(seed),
        }
    }

    /// 生成区块数据
    pub fn generate_chunk_data(&self, chunk_pos: IVec3, block_ids: GenerationBlockIds) -> ChunkData {
        self.pipeline.generate_chunk(chunk_pos, block_ids)
    }

    /// 更新季节
    pub fn update_season(&mut self, season: climate::Season) {
        self.pipeline.update_season(season);
    }
}