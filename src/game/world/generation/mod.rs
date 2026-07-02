pub mod climate;
pub mod context;
pub mod noise;
pub mod pipeline;
pub mod structure;
pub mod terrain;

use crate::content::biome::definition::BiomeRegistry;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::climate::ClimateSampler;
use crate::game::world::generation::noise::{GenerationBlockIds, NoiseSampler};
use crate::game::world::generation::pipeline::GenerationPipeline;
use bevy::prelude::*;
use std::sync::Arc;

#[derive(Resource)]
pub struct WorldGenerator {
    pub seed: u32,
    pub pipeline: GenerationPipeline,
    pub shared_noise: Arc<NoiseSampler>,
    pub shared_climate: Arc<ClimateSampler>,
    pub shared_biome: Arc<BiomeRegistry>,
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        let pipeline = GenerationPipeline::new(seed);

        Self {
            seed,
            shared_noise: Arc::new(pipeline.noise_sampler.clone()),
            shared_climate: Arc::new(pipeline.climate_sampler.clone()),
            shared_biome: Arc::new(pipeline.biome_registry.clone()),
            pipeline,
        }
    }

    /// 生成区块数据
    pub fn generate_chunk_data(
        &self,
        chunk_pos: IVec3,
        block_ids: GenerationBlockIds,
    ) -> ChunkData {
        self.pipeline.generate_chunk(chunk_pos, block_ids)
    }

    /// 更新季节
    pub fn update_season(&mut self, season: climate::Season) {
        self.pipeline.update_season(season);
    }
}
