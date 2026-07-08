pub mod break_pipeline;
mod channel;
mod chunk_lifecycle;
mod greedy_mesh;
mod mesh_buffer;
mod mesh_lifecycle;
pub mod pickup;
mod streaming;

pub use channel::{
    BlockInfoSnapshot, CachedBlockInfo, MeshBuildChannel, MeshBuildInput, MeshBuildResult,
    PlayerChunkCache, StructureGenChannel, TerrainGenChannel, TerrainGenResult,
};
pub use chunk_lifecycle::{
    generate_structures_system, manage_chunks_system, receive_structure_results,
    receive_terrain_results, spawn_terrain_gen_tasks,
};
pub use greedy_mesh::build_greedy_mesh;
pub use mesh_buffer::{DIRECTIONS, MeshBufferData};
pub use mesh_lifecycle::{
    rebuild_block_info_snapshot, receive_mesh_results, spawn_mesh_build_tasks,
};
pub use streaming::WorldStreamingConfig;
