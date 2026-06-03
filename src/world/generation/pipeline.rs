use bevy::prelude::IVec3;
use crate::world::chunk::ChunkData;
use crate::world::generation::biome::BiomeRegistry;
use crate::world::generation::climate::{ClimateConfig, ClimateSampler};
use crate::world::generation::context::ChunkGenContext;
use crate::world::generation::noise::{GenerationBlockIds, NoiseSampler};
use crate::world::generation::structure::StructureGenerator;
use crate::world::generation::terrain::TerrainGenerator;
use crate::world::storage::WorldStorage;

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

    /// 一键生成：执行完整管线
    pub fn generate_chunk(
        &self,
        chunk_pos: IVec3,
        block_ids: GenerationBlockIds,
        world_storage: &mut WorldStorage,
    ) -> ChunkData {
        // 采样上下文（气候 + 群系）
        let ctx = TerrainGenerator::sample_context(
            &self.noise_sampler,
            &self.climate_sampler,
            &self.biome_registry,
            chunk_pos,
        );

        // 生成地形
        let mut chunk_data = TerrainGenerator::generate_terrain(
            &ctx,
            &block_ids,
            &self.biome_registry,
        );

        // 放置结构
        StructureGenerator::generate_structures(
            &mut chunk_data,
            &ctx,
            &block_ids,
            &self.biome_registry,
            self.seed,
            world_storage,
        );

        // 后处理（洞穴等，后续实现）
        // self.post_process(&mut chunk_data, &ctx, chunk_pos);

        chunk_data
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
                    &self.biome_registry,
                    chunk_pos,
                );
                (None, ctx, GenerationStage::Terrain)
            }
            GenerationStage::Terrain => {
                let ctx = context.expect("Terrain stage requires context");
                let data = TerrainGenerator::generate_terrain(
                    &ctx, &block_ids, &self.biome_registry,
                );
                (Some(data), ctx, GenerationStage::Structure)
            }
            GenerationStage::Structure => {
                let mut data = chunk_data.expect("Structure stage requires chunk_data");
                let ctx = context.expect("Structure stage requires context");
                StructureGenerator::generate_structures(
                    &mut data, &ctx, &block_ids, &self.biome_registry, self.seed, world_storage,
                );
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
    pub fn update_season(&mut self, season: crate::world::generation::climate::Season) {
        self.climate_sampler.current_season = season;
    }
}