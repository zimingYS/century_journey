//! 注册世界生成通道与任务系统，并声明生成阶段的调度顺序。

use super::sync::{
    cache_block_ids_system, sync_simulation_rng_seed_system, sync_terrain_surface_sampler_system,
    sync_world_biomes_system, sync_world_ores_system,
};
use super::{
    StructureGenChannel, TerrainGenChannel, generate_structures_system, receive_structure_results,
    receive_terrain_results, spawn_terrain_gen_tasks,
};
use crate::content::biome::BiomeRegistry;
use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::game::world::generation::block_ids::{CachedBlockIds, GenerationBlockIds};
use crate::game::world::generation::generator::WorldGenerator;
use crate::game::world::generation::pipeline::{GenerationPipeline, TerrainSurfaceSampler};
use crate::game::world::generation::terrain::climate::{ClimateConfig, ClimateSampler};
use crate::game::world::streaming::manage_chunks_system;
use crate::shared::states::AppState;
use bevy::prelude::*;

/// 组装地形和结构生成的资源、任务通道及结果回收系统。
///
/// 区块实体必须先由流送系统创建，因此生成任务在 Update 中显式排在
/// `manage_chunks_system` 之后。地形与结构阶段继续使用链式顺序。
pub(in crate::game::world) struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldGenerator::new(12345, BiomeRegistry::default()))
            .insert_resource(ClimateSampler::new(12345, ClimateConfig::default()))
            .init_resource::<CachedBlockIds>()
            .insert_resource(TerrainSurfaceSampler::pending(
                GenerationPipeline::new(12345, BiomeRegistry::default()),
                GenerationBlockIds::default(),
            ))
            .init_resource::<TerrainGenChannel>()
            .init_resource::<StructureGenChannel>()
            .add_systems(
                Update,
                (
                    spawn_terrain_gen_tasks,
                    receive_terrain_results,
                    generate_structures_system,
                    receive_structure_results,
                )
                    .chain()
                    .after(manage_chunks_system)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnEnter(AppState::InGame), sync_simulation_rng_seed_system)
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    sync_world_biomes_system,
                    cache_block_ids_system,
                    sync_world_ores_system,
                    sync_terrain_surface_sampler_system,
                )
                    .chain()
                    .after(crate::content::tag::init_tag_registry_system)
                    .in_set(ContentReloadSet::Consumers)
                    .run_if(content_reload_requested),
            );
    }
}
