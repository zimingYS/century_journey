//! 负责把权威区块和掉落实体转换为客户端可见网格。

mod channel;
pub mod dropped_item;
mod greedy_mesh;
mod mesh_buffer;
mod mesh_lifecycle;

pub use channel::{
    BlockInfoSnapshot, CachedBlockInfo, MeshBuildChannel, MeshBuildInput, MeshBuildResult,
};
pub use greedy_mesh::build_greedy_mesh;
pub use mesh_buffer::{DIRECTIONS, MeshBufferData};
pub use mesh_lifecycle::{
    rebuild_block_info_snapshot, receive_mesh_results, spawn_mesh_build_tasks,
};
