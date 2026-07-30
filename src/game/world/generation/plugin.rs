//! 注册世界生成通道与任务系统，并声明生成阶段的调度顺序。

use super::{
    StructureGenChannel, TerrainGenChannel, generate_structures_system, receive_structure_results,
    receive_terrain_results, spawn_terrain_gen_tasks,
};
use crate::content::biome::BiomeRegistry;
use crate::content::block::registry::BlockRegistry;
use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::game::simulation::SimulationRng;
use crate::game::world::generation::block_ids::{CachedBlockIds, GenerationBlockIds};
use crate::game::world::generation::generator::WorldGenerator;
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
                (sync_world_biomes_system, cache_block_ids_system)
                    .chain()
                    .after(crate::content::tag::plugin::init_tag_registry_system)
                    .in_set(ContentReloadSet::Consumers)
                    .run_if(content_reload_requested),
            );
    }
}

fn sync_simulation_rng_seed_system(
    world_generator: Res<WorldGenerator>,
    mut simulation_rng: ResMut<SimulationRng>,
) {
    simulation_rng.set_world_seed(world_generator.seed as u64);
}

fn sync_world_biomes_system(
    registry: Res<BiomeRegistry>,
    mut world_generator: ResMut<WorldGenerator>,
) {
    if registry.is_empty() {
        log::error!("[世界] 生物群系注册表为空，跳过世界生成器刷新");
        return;
    }

    world_generator.set_biome_registry(registry.clone());
}

fn cache_block_ids_system(
    registry: Res<BlockRegistry>,
    tag_registry: Option<Res<RuntimeTagRegistry>>,
    mut commands: Commands,
) {
    let block_ids = if let Some(tag_registry) = tag_registry {
        GenerationBlockIds::from_registry(&registry, &tag_registry)
    } else {
        log::warn!("[世界] RuntimeTagRegistry 尚未初始化，使用空标签");
        GenerationBlockIds::from_registry(&registry, &RuntimeTagRegistry::default())
    };

    commands.insert_resource(CachedBlockIds(block_ids));
}
