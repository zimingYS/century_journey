use crate::content::block::registry::BlockRegistry;
use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::game::simulation::{SimulationRng, SimulationSet};
use crate::game::world::generation::noise::CachedBlockIds;
use crate::game::world::{entity, generation, state, systems, time};
use crate::shared::states::AppState;
use bevy::app::{App, FixedUpdate, Plugin, Startup, Update};
use bevy::prelude::{Commands, IntoScheduleConfigs, OnEnter, Res, ResMut, in_state};

/// 组装世界基础资源、时间、生成、流送和实体子领域插件。
///
/// 本插件只负责世界领域的顶层装配；具体运行逻辑将逐步下沉到各子领域插件。
pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<state::WorldState>()
            .init_resource::<state::ChunkRuntime>()
            .init_resource::<crate::game::block::BlockBehaviorRegistry>()
            .add_systems(Startup, crate::game::block::init_behavior_registry_system)
            .insert_resource(generation::WorldGenerator::new(
                12345,
                crate::content::biome::BiomeRegistry::default(),
            ))
            .init_resource::<systems::TerrainGenChannel>()
            .init_resource::<systems::StructureGenChannel>()
            .init_resource::<systems::PlayerChunkCache>()
            .init_resource::<systems::WorldStreamingConfig>()
            .add_plugins((time::WorldTimePlugin, entity::EntityPlugin))
            .add_systems(
                Update,
                (
                    systems::manage_chunks_system,
                    systems::spawn_terrain_gen_tasks,
                    systems::receive_terrain_results,
                    systems::generate_structures_system,
                    systems::receive_structure_results,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                systems::pickup::pickup_system
                    .after(entity::dropped_item::dropped_item_tick_system)
                    .in_set(SimulationSet::Entities)
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
    world_generator: Res<generation::WorldGenerator>,
    mut simulation_rng: ResMut<SimulationRng>,
) {
    simulation_rng.set_world_seed(world_generator.seed as u64);
}

fn sync_world_biomes_system(
    registry: Res<crate::content::biome::BiomeRegistry>,
    mut world_generator: ResMut<generation::WorldGenerator>,
) {
    if registry.is_empty() {
        log::error!("[世界] 群系注册表为空，跳过世界生成器刷新");
        return;
    }
    world_generator.set_biome_registry(registry.clone());
}

fn cache_block_ids_system(
    registry: Res<BlockRegistry>,
    tag_registry: Option<Res<RuntimeTagRegistry>>,
    mut commands: Commands,
) {
    let block_ids = if let Some(ref tr) = tag_registry {
        generation::noise::GenerationBlockIds::from_registry(&registry, tr)
    } else {
        log::warn!("[世界] RuntimeTagRegistry 尚未初始化，使用空标签");
        generation::noise::GenerationBlockIds::from_registry(
            &registry,
            &RuntimeTagRegistry::default(),
        )
    };
    commands.insert_resource(CachedBlockIds(block_ids));
}
