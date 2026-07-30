//! 编排环境采样、地形塑造、生物群系分类和结构放置的生成流水线。

use crate::content::biome::registry::BiomeRegistry;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::block_ids::GenerationBlockIds;
use crate::game::world::generation::terrain::climate::{ClimateConfig, ClimateSampler};
use crate::game::world::generation::terrain::context::ChunkGenContext;
use crate::game::world::generation::terrain::generator::TerrainGenerator;
use crate::game::world::generation::terrain::noise::NoiseSampler;
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
    /// 使用当前算法版本构建生成管线。
    pub fn new(seed: u32, biome_registry: BiomeRegistry) -> Self {
        Self::with_generation_version(seed, CURRENT_GENERATION_VERSION, biome_registry)
    }

    /// 使用显式兼容版本构建生成管线，并拒绝未知版本。
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

    /// 返回指定区块的完整基础生成键。
    pub fn key(&self, chunk_pos: IVec3) -> BaseGenerationKey {
        BaseGenerationKey {
            seed: self.seed,
            chunk_pos,
            generation_version: self.generation_version,
        }
    }

    /// 采样区块每个地表列所需的地形与气候上下文。
    pub fn sample_context(&self, chunk_pos: IVec3) -> ChunkGenContext {
        TerrainGenerator::sample_context(
            &self.noise_sampler,
            &self.climate_sampler,
            &self.biome_registry,
            self.key(chunk_pos),
        )
    }

    /// 生成不含跨区块结构的基础区块及其可复用上下文。
    pub fn generate_base_chunk(
        &self,
        chunk_pos: IVec3,
        block_ids: &GenerationBlockIds,
    ) -> (ChunkData, ChunkGenContext) {
        let context = self.sample_context(chunk_pos);
        let data = TerrainGenerator::generate_terrain(&context, block_ids, &self.biome_registry);
        (data, context)
    }

    /// 生成指定坐标的基础区块体素数据。
    pub fn generate_chunk(&self, chunk_pos: IVec3, block_ids: GenerationBlockIds) -> ChunkData {
        self.generate_base_chunk(chunk_pos, &block_ids).0
    }

    /// 替换后续生成任务使用的生物群系注册表快照。
    pub fn replace_biome_registry(&mut self, biome_registry: BiomeRegistry) {
        self.biome_registry = Arc::new(biome_registry);
    }
}
