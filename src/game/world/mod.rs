pub mod block_ops;
pub mod chunk;
pub mod entity;
pub mod generation;
pub mod save;
pub mod storage;
pub mod systems;
pub mod time;

use crate::content::block::registry::BlockRegistry;
use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::game::world::generation::noise::CachedBlockIds;
use crate::shared::states::app_state::AppState;
use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<storage::WorldStorage>()
            .init_resource::<crate::game::block::BlockBehaviorRegistry>()
            .add_systems(Startup, crate::game::block::init_behavior_registry_system)
            .insert_resource(generation::WorldGenerator::new(
                12345,
                crate::content::biome::BiomeRegistry::default(),
            ))
            .insert_resource(time::TimeOfDay::default())
            .init_resource::<generation::climate::SeasonResource>()
            .init_resource::<systems::TerrainGenChannel>()
            .init_resource::<systems::StructureGenChannel>()
            .init_resource::<systems::PlayerChunkCache>()
            .init_resource::<systems::WorldStreamingConfig>()
            .add_plugins(save::SaveLoadPlugin)
            .add_plugins(entity::EntityPlugin)
            .add_systems(
                Update,
                (
                    systems::manage_chunks_system,
                    systems::spawn_terrain_gen_tasks,
                    systems::receive_terrain_results,
                    systems::generate_structures_system,
                    systems::receive_structure_results,
                    systems::pickup::pickup_system,
                    time::update_time_system,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
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
