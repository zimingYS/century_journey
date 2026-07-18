use crate::content::biome::registry::BiomeRegistry;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::climate::{ClimateConfig, ClimateSampler};
use crate::game::world::generation::context::ChunkGenContext;
use crate::game::world::generation::noise::{GenerationBlockIds, NoiseSampler};
use crate::game::world::generation::terrain::TerrainGenerator;
use bevy::prelude::IVec3;
use std::sync::Arc;

/// 当前基础地形算法版本。它只在明确修改基础体素生成规则时递增。
pub const CURRENT_GENERATION_VERSION: u32 = 2;
/// 旧存档在引入显式生成版本前使用的基础地形规则。
pub const LEGACY_GENERATION_VERSION: u32 = 1;

/// 一次基础区块生成的完整可变输入键。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BaseGenerationKey {
    pub seed: u32,
    pub chunk_pos: IVec3,
    pub generation_version: u32,
}

/// 只读、可跨线程克隆的基础生成管线。
///
/// 区块生命周期由 `ChunkState` 驱动；这里不再维护一套没有执行能力的阶段枚举。
#[derive(Clone)]
pub struct GenerationPipeline {
    pub noise_sampler: Arc<NoiseSampler>,
    pub climate_sampler: Arc<ClimateSampler>,
    pub biome_registry: Arc<BiomeRegistry>,
    pub seed: u32,
    pub generation_version: u32,
}

impl GenerationPipeline {
    pub fn new(seed: u32, biome_registry: BiomeRegistry) -> Self {
        Self::with_generation_version(seed, CURRENT_GENERATION_VERSION, biome_registry)
    }

    pub fn with_generation_version(
        seed: u32,
        generation_version: u32,
        biome_registry: BiomeRegistry,
    ) -> Self {
        assert!(
            (LEGACY_GENERATION_VERSION..=CURRENT_GENERATION_VERSION).contains(&generation_version),
            "unsupported generation version {generation_version}"
        );
        Self {
            noise_sampler: Arc::new(NoiseSampler::new(seed)),
            climate_sampler: Arc::new(ClimateSampler::new(seed, ClimateConfig::default())),
            biome_registry: Arc::new(biome_registry),
            seed,
            generation_version,
        }
    }

    pub fn key(&self, chunk_pos: IVec3) -> BaseGenerationKey {
        BaseGenerationKey {
            seed: self.seed,
            chunk_pos,
            generation_version: self.generation_version,
        }
    }

    pub fn sample_context(&self, chunk_pos: IVec3) -> ChunkGenContext {
        TerrainGenerator::sample_context(
            &self.noise_sampler,
            &self.climate_sampler,
            &self.biome_registry,
            self.key(chunk_pos),
        )
    }

    pub fn generate_base_chunk(
        &self,
        chunk_pos: IVec3,
        block_ids: &GenerationBlockIds,
    ) -> (ChunkData, ChunkGenContext) {
        let context = self.sample_context(chunk_pos);
        let data = TerrainGenerator::generate_terrain(&context, block_ids, &self.biome_registry);
        (data, context)
    }

    pub fn generate_chunk(&self, chunk_pos: IVec3, block_ids: GenerationBlockIds) -> ChunkData {
        self.generate_base_chunk(chunk_pos, &block_ids).0
    }

    pub fn replace_biome_registry(&mut self, biome_registry: BiomeRegistry) {
        self.biome_registry = Arc::new(biome_registry);
    }
}
