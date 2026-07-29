use crate::content::biome::BiomeRegistry;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::block_ids::GenerationBlockIds;
use crate::game::world::generation::pipeline::{CURRENT_GENERATION_VERSION, GenerationPipeline};
use bevy::math::IVec3;
use bevy::prelude::Resource;

#[derive(Resource)]
pub struct WorldGenerator {
    pub seed: u32,
    pub generation_version: u32,
    pub pipeline: GenerationPipeline,
}

impl WorldGenerator {
    pub fn new(seed: u32, biome_registry: BiomeRegistry) -> Self {
        Self::with_generation_version(seed, CURRENT_GENERATION_VERSION, biome_registry)
    }

    pub fn with_generation_version(
        seed: u32,
        generation_version: u32,
        biome_registry: BiomeRegistry,
    ) -> Self {
        let pipeline =
            GenerationPipeline::with_generation_version(seed, generation_version, biome_registry);
        Self {
            seed,
            generation_version,
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

    pub fn set_biome_registry(&mut self, biome_registry: BiomeRegistry) {
        self.pipeline.replace_biome_registry(biome_registry);
    }
}
