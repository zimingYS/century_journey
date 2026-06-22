use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::biome::BiomeRegistry;
use crate::game::world::generation::climate::{ClimateConfig, ClimateSampler};
use crate::game::world::generation::context::ChunkGenContext;
use crate::game::world::generation::noise::{GenerationBlockIds, NoiseSampler};
use crate::game::world::generation::structure::StructureGenerator;
use crate::game::world::generation::terrain::TerrainGenerator;
use crate::game::world::storage::WorldStorage;
use bevy::prelude::IVec3;

/// 生成管线阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationStage {
    /// 采样气候 + 选择群系
    ClimateAndBiome,
    /// 生成基础地形
    Terrain,
    /// 放置结构（树/矿脉）
    Structure,
    /// 后处理（洞穴/侵蚀）
    PostProcess,
    /// 完成
    Done,
}

/// 生成管线 — 组合所有生成阶段
#[derive(Clone)]
pub struct GenerationPipeline {
    pub noise_sampler: NoiseSampler,
    pub climate_sampler: ClimateSampler,
    pub biome_registry: BiomeRegistry,
    pub seed: u32,
}

impl GenerationPipeline {
    pub fn new(seed: u32) -> Self {
        Self {
            noise_sampler: NoiseSampler::new(seed),
            climate_sampler: ClimateSampler::new(seed, ClimateConfig::default()),
            biome_registry: {
                let mut reg = BiomeRegistry::default();
                reg.register_builtin_biomes();
                reg
            },
            seed,
        }
    }

    /// 生成区块地形
    pub fn generate_chunk(&self, chunk_pos: IVec3, block_ids: GenerationBlockIds) -> ChunkData {
        // 采样气候和群系
        let ctx = TerrainGenerator::sample_context(
            &self.noise_sampler,
            &self.climate_sampler,
            self.climate_sampler.current_season,
            &self.biome_registry,
            chunk_pos,
        );

        // 生成地形
        TerrainGenerator::generate_terrain(&ctx, &block_ids, &self.biome_registry)
    }

    /// 根据种子、气候、当前季节与生物群系注册表重建世界生成
    pub fn rebuild_from_seed(
        seed: u32,
        climate_config: ClimateConfig,
        current_season: crate::game::world::generation::climate::Season,
        biome_registry: BiomeRegistry,
    ) -> Self {
        let mut pipeline = Self::new(seed);
        pipeline.climate_sampler = ClimateSampler::new(seed, climate_config);
        pipeline.climate_sampler.current_season = current_season;
        pipeline.biome_registry = biome_registry;
        pipeline
    }

    /// 分阶段生成：允许逐帧执行，减少卡顿
    pub fn generate_chunk_staged(
        &self,
        chunk_pos: IVec3,
        block_ids: GenerationBlockIds,
        from_stage: GenerationStage,
        context: Option<ChunkGenContext>,
        chunk_data: Option<ChunkData>,
        world_storage: &mut WorldStorage,
    ) -> (Option<ChunkData>, ChunkGenContext, GenerationStage) {
        match from_stage {
            GenerationStage::ClimateAndBiome => {
                let ctx = TerrainGenerator::sample_context(
                    &self.noise_sampler,
                    &self.climate_sampler,
                    self.climate_sampler.current_season,
                    &self.biome_registry,
                    chunk_pos,
                );
                (None, ctx, GenerationStage::Terrain)
            }
            GenerationStage::Terrain => {
                let ctx = context.expect("Terrain stage requires context");
                let data =
                    TerrainGenerator::generate_terrain(&ctx, &block_ids, &self.biome_registry);
                (Some(data), ctx, GenerationStage::Structure)
            }
            GenerationStage::Structure => {
                let data = chunk_data.expect("Structure stage requires chunk_data");
                let ctx = context.expect("Structure stage requires context");
                (Some(data), ctx, GenerationStage::PostProcess)
            }
            GenerationStage::PostProcess => {
                // 后处理暂为空操作
                let data = chunk_data.expect("PostProcess stage requires chunk_data");
                let ctx = context.expect("PostProcess stage requires context");
                (Some(data), ctx, GenerationStage::Done)
            }
            GenerationStage::Done => {
                let data = chunk_data.expect("Done stage requires chunk_data");
                let ctx = context.expect("Done stage requires context");
                (Some(data), ctx, GenerationStage::Done)
            }
        }
    }

    /// 更新季节（由昼夜循环系统调用）
    pub fn update_season(&mut self, season: crate::game::world::generation::climate::Season) {
        self.climate_sampler.current_season = season;
    }
}
