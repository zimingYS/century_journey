//! 编排环境采样、地形塑造、生物群系分类和结构放置的生成流水线。

use crate::content::biome::registry::BiomeRegistry;
use crate::content::ore_vein::registry::RuntimeOreVein;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::block_ids::GenerationBlockIds;
use crate::game::world::generation::cave::{DEFAULT_CAVE_PROFILE, apply_caves};
use crate::game::world::generation::ore;
use crate::game::world::generation::terrain::climate::{ClimateConfig, ClimateSampler};
use crate::game::world::generation::terrain::context::ChunkGenContext;
use crate::game::world::generation::terrain::generator::{TerrainGenerator, TerrainSurfaceSample};
use crate::game::world::generation::terrain::noise::NoiseSampler;
use bevy::prelude::{IVec3, Resource};
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

/// 已解析为运行时方块 ID 的只读地表采样结果。
///
/// 该快照只用于 Client 远景网格，保留基础生成所需的地表/次表/石头/水方块语义，
/// 不进入 `ChunkData` 或存档；同一管线快照在任意线程上的结果都相同。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedTerrainSurfaceSample {
    /// 基础地表方块所在的世界 Y 坐标。
    pub ground_height: i32,
    /// 地表或海面可见顶面的世界 Y 坐标。
    pub visible_surface_height: i32,
    /// 当前列是否由海平面水体覆盖。
    pub is_water_surface: bool,
    /// 可见地表方块运行时 ID。
    pub surface_block: u16,
    /// 次表方块运行时 ID。
    pub subsurface_block: u16,
    /// 深层代表方块运行时 ID。
    pub stone_block: u16,
    /// 水体方块运行时 ID。
    pub water_block: u16,
}

/// Game 层提供的确定性远景采样服务。
///
/// 服务内部持有不可变的生成管线和方块 ID 快照；Client 只能调用
/// `sample_surface`，不能读取噪声、群系或生成器内部状态。它可安全克隆到后台任务。
#[derive(Resource, Clone)]
pub struct TerrainSurfaceSampler {
    pipeline: GenerationPipeline,
    block_ids: GenerationBlockIds,
}

impl TerrainSurfaceSampler {
    /// 创建尚未绑定内容注册表的采样服务占位资源。
    ///
    /// 世界进入游戏并完成内容刷新后必须由 `from_generation_state` 原子替换；占位资源
    /// 的 `is_ready` 始终为 false，Client 不会据此派发网格任务。
    pub(crate) fn pending(pipeline: GenerationPipeline, block_ids: GenerationBlockIds) -> Self {
        Self {
            pipeline,
            block_ids,
        }
    }

    /// 从当前世界生成管线和方块缓存创建就绪的只读服务。
    pub(crate) fn from_generation_state(
        pipeline: GenerationPipeline,
        block_ids: GenerationBlockIds,
    ) -> Self {
        Self {
            pipeline,
            block_ids,
        }
    }

    /// 判断地形与方块映射是否都已就绪。
    pub fn is_ready(&self) -> bool {
        !self.pipeline.biome_registry.is_empty()
            && self.block_ids.stone != self.block_ids.air
            && self.block_ids.water != self.block_ids.air
    }

    /// 返回当前采样服务绑定的基础生成版本。
    pub fn generation_version(&self) -> u32 {
        self.pipeline.generation_version
    }

    /// 采样任意世界列，并返回可直接交给表现层网格的运行时方块 ID。
    pub fn sample_surface(&self, world_x: i32, world_z: i32) -> ResolvedTerrainSurfaceSample {
        let sample = self.pipeline.sample_surface(world_x, world_z);
        ResolvedTerrainSurfaceSample {
            ground_height: sample.ground_height,
            visible_surface_height: sample.visible_surface_height,
            is_water_surface: sample.is_water_surface,
            // 必须复用真实区块生成的标识解析路径，不能在 LOD 侧另建映射规则。
            surface_block: self.block_ids.resolve_block_id(&sample.surface_block),
            subsurface_block: self.block_ids.resolve_block_id(&sample.subsurface_block),
            stone_block: self.block_ids.stone,
            water_block: self.block_ids.water,
        }
    }
    /// 采样任意世界列的基础温湿度（不含运行期季节），供客户端生物群系着色使用。
    ///
    /// 生物群系色由稳定的基础气候决定，季节作为独立乘子叠加，避免群系边界随季节漂移。
    pub fn sample_climate(&self, world_x: i32, world_z: i32) -> (f64, f64) {
        (
            self.pipeline.climate_sampler.sample_temperature(world_x, world_z),
            self.pipeline.climate_sampler.sample_humidity(world_x, world_z),
        )
    }
}

/// 只读、可跨线程克隆的基础生成管线。
///
/// 区块生命周期由 `ChunkState` 驱动；这里不再维护一套没有执行能力的阶段枚举。
#[derive(Clone)]
pub struct GenerationPipeline {
    pub noise_sampler: Arc<NoiseSampler>,
    pub climate_sampler: Arc<ClimateSampler>,
    pub biome_registry: Arc<BiomeRegistry>,
    pub ore_veins: Arc<Vec<RuntimeOreVein>>,
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
            ore_veins: Arc::new(Vec::new()),
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

    /// 采样任意世界列的确定性基础地表。
    ///
    /// 该接口只提供远景等纯表现所需的地形轮廓；洞穴、矿石、结构和玩家修改不在
    /// 其中，调用方不得据此推断权威体素或可交互状态。
    pub fn sample_surface(&self, world_x: i32, world_z: i32) -> TerrainSurfaceSample {
        TerrainGenerator::sample_surface(
            &self.noise_sampler,
            &self.climate_sampler,
            &self.biome_registry,
            self.key(IVec3::ZERO),
            world_x,
            world_z,
        )
    }

    /// 生成不含跨区块结构的基础区块及其可复用上下文。
    pub fn generate_base_chunk(
        &self,
        chunk_pos: IVec3,
        block_ids: &GenerationBlockIds,
    ) -> (ChunkData, ChunkGenContext) {
        let context = self.sample_context(chunk_pos);
        let mut data =
            TerrainGenerator::generate_terrain(&context, block_ids, &self.biome_registry);
        apply_caves(
            &mut data,
            &context,
            &self.noise_sampler,
            block_ids.stone,
            &DEFAULT_CAVE_PROFILE,
        );
        ore::apply_ores(
            &mut data,
            chunk_pos,
            &self.noise_sampler,
            block_ids.stone,
            &self.ore_veins,
        );
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

    /// 替换后续生成任务使用的矿脉注册表快照。
    pub fn replace_ore_veins(&mut self, veins: Vec<RuntimeOreVein>) {
        self.ore_veins = Arc::new(veins);
    }
}
