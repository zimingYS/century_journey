pub mod chunk;
pub mod storage;
pub mod generation;
pub mod systems;


use bevy::prelude::*;
use crate::ui::hud::hotbar::{handle_hotbar_switch_system, spawn_hotbar_ui_system, update_hotbar_ui_system};
use crate::world::generation::WorldGenerator;
use crate::world::storage::WorldStorage;
use crate::world::systems::{build_chunk_mesh_system, generate_chunk_data_system, manage_chunks_system};

pub struct WorldPlugin;

impl Plugin for WorldPlugin{
    fn build(&self, app: &mut App){
        app
            .init_resource::<WorldStorage>()
            .insert_resource(WorldGenerator::new(12345))
            .add_systems(Update,(
                manage_chunks_system,
                generate_chunk_data_system,
                build_chunk_mesh_system
            ));
    }
}