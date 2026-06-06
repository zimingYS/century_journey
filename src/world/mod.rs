pub mod chunk;
pub mod storage;
pub mod generation;
pub mod systems;
pub mod sky;
pub mod time;
pub mod save;

use bevy::prelude::*;
use crate::core::state::app_state::AppState;
use crate::voxel::registry::BlockRegistry;
use crate::world::generation::noise::CachedBlockIds;

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
            .add_plugins(sky::SkyPlugin)
            .add_plugins(save::SaveLoadPlugin)

            .add_systems(Update,(
                systems::manage_chunks_system,
                systems::spawn_terrain_gen_tasks,
                systems::receive_terrain_results,
                systems::generate_structures_system,
                systems::receive_structure_results,
                systems::spawn_mesh_build_tasks,
                systems::receive_mesh_results,
                time::update_time_system,
            ).chain().run_if(in_state(AppState::InGame)))
        .add_systems(OnEnter(AppState::InGame), cache_block_ids_system);
    }
}

fn cache_block_ids_system(
    registry: Res<BlockRegistry>,
    mut commands: Commands,
) {
    commands.insert_resource(CachedBlockIds(
        generation::noise::GenerationBlockIds::from_registry(&registry)
    ));
}