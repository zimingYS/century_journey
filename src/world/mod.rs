pub mod chunk;
pub mod storage;
pub mod generation;
pub mod systems;
pub mod sky;
pub mod time;

use bevy::prelude::*;
use crate::core::state::app_state::AppState;

pub struct WorldPlugin;

impl Plugin for WorldPlugin{
    fn build(&self, app: &mut App){
        app
            .init_resource::<storage::WorldStorage>()
            .insert_resource(generation::WorldGenerator::new(12345))
            .insert_resource(time::TimeOfDay::default())
            .add_plugins(sky::SkyPlugin)
            .add_systems(Update,(
                systems::manage_chunks_system,
                systems::generate_chunk_data_system,
                systems::build_chunk_mesh_system,
                time::update_time_system,
            ).run_if(in_state(AppState::InGame)));
    }
}