//! 对外封装按种子与版本生成区块的稳定世界生成器。

use crate::content::biome::BiomeRegistry;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::block_ids::GenerationBlockIds;
use crate::game::world::generation::pipeline::{CURRENT_GENERATION_VERSION, GenerationPipeline};
use bevy::math::IVec3;
use bevy::prelude::Resource;

/// 当前世界会话使用的生成种子、算法版本和只读生成管线。
#[derive(Resource)]
pub struct WorldGenerator {
    pub seed: u32,
    pub generation_version: u32,
    pub pipeline: GenerationPipeline,
}

impl WorldGenerator {
    /// 使用当前生成算法版本创建世界生成器。
    pub fn new(seed: u32, biome_registry: BiomeRegistry) -> Self {
        Self::with_generation_version(seed, CURRENT_GENERATION_VERSION, biome_registry)
    }

    /// 使用指定兼容版本创建世界生成器，供旧存档继续生成区块。
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

    /// 生成指定坐标的完整区块体素数据。
    pub fn generate_chunk_data(
        &self,
        chunk_pos: IVec3,
        block_ids: GenerationBlockIds,
    ) -> ChunkData {
        self.pipeline.generate_chunk(chunk_pos, block_ids)
    }

    /// 在内容重载后替换生成管线使用的生物群系注册表。
    pub fn set_biome_registry(&mut self, biome_registry: BiomeRegistry) {
        self.pipeline.replace_biome_registry(biome_registry);
    }
}
