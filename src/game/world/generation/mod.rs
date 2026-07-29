pub mod biome;
pub mod block_ids;
pub mod generator;
pub mod pipeline;
mod plugin;
mod runtime;
pub mod structure;
pub mod terrain;

pub(in crate::game::world) use plugin::WorldGenerationPlugin;
pub(crate) use runtime::{
    StructureGenChannel, TerrainGenChannel, generate_structures_system, receive_structure_results,
    receive_terrain_results, spawn_terrain_gen_tasks,
};
