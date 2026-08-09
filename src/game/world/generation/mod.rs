//! 组织确定性世界生成的地形、生物群系、结构和异步运行时。

pub mod biome;
pub mod block_ids;
pub mod cave;
pub mod generator;
pub mod ore;
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
