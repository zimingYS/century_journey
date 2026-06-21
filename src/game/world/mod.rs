pub mod chunk;
pub mod storage;
pub mod generation;
pub mod systems;
pub mod block_ops;
pub mod time;
pub mod save;
pub mod entity;

use bevy::prelude::*;
use crate::app::state::AppState;
use crate::shared::tag;
use crate::shared::tag::cache::CachedTagCache;
use crate::content::block::registry::BlockRegistry;
use crate::game::world::generation::noise::CachedBlockIds;

pub struct WorldPlugin;

impl Plugin for WorldPlugin{
    fn build(&self, app: &mut App){
        app
            .init_resource::<storage::WorldStorage>()
            .insert_resource(generation::WorldGenerator::new(12345))
            .insert_resource(time::TimeOfDay::default())
            .init_resource::<generation::climate::SeasonResource>()
            .init_resource::<systems::TerrainGenChannel>()
            .init_resource::<systems::MeshBuildChannel>()
            .init_resource::<systems::StructureGenChannel>()
            .init_resource::<systems::PlayerChunkCache>()
            .init_resource::<systems::CachedBlockInfo>()
            .add_plugins(save::SaveLoadPlugin)
            .add_plugins(entity::EntityPlugin)
            .add_systems(
                Update,
                systems::rebuild_block_info_snapshot
                    .before(systems::spawn_mesh_build_tasks)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(Update,(
                systems::manage_chunks_system,
                systems::spawn_terrain_gen_tasks,
                systems::receive_terrain_results,
                systems::generate_structures_system,
                systems::receive_structure_results,
                systems::spawn_mesh_build_tasks,
                systems::receive_mesh_results,
                systems::pickup::pickup_system,
                time::update_time_system,
            ).chain().run_if(in_state(AppState::InGame)))
        .add_systems(OnEnter(AppState::InGame), cache_block_ids_system);
    }
}

fn cache_block_ids_system(
    registry: Res<BlockRegistry>,
    cached_tag_cache: Option<Res<CachedTagCache>>,
    mut commands: Commands,
) {
    let block_ids = if let Some(ref cache) = cached_tag_cache {
        generation::noise::GenerationBlockIds::from_registry(&registry, &cache.0)
    } else {
        log::warn!("[世界] CachedTagCache 尚未初始化，使用空标签缓存");
        generation::noise::GenerationBlockIds::from_registry(
            &registry,
            &tag::cache::TagCache::default(),
        )
    };
    commands.insert_resource(CachedBlockIds(block_ids));
}