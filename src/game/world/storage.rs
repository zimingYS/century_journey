use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::context::ChunkGenContext;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Legacy compatibility storage.
///
/// New code should use `WorldState`, `ChunkRuntime`, and `ClientPresentation`.
/// This type remains temporarily for save/structure adapters during migration.
#[derive(Resource, Default)]
pub struct WorldStorage {
    /// 键是区块的 3D 坐标，值是区块的原生数据，用于存储世界的方块数据
    pub loaded_chunks: HashMap<IVec3, Arc<ChunkData>>,
    /// 记录哪些区块已经在场景中生成了 Mesh 实体，存储区块对应的渲染实体
    pub chunk_entities: HashMap<IVec3, Entity>,
    /// 每个区块最后修改时间
    pub chunk_modified_times: HashMap<IVec3, f64>,
    /// 待处理的方块写入队列
    pub pending_writes: PendingVoxelWrites,
    /// 缓存区块的生成上下文，供结构生成复用，避免重复噪声采样
    pub gen_contexts: HashMap<IVec3, ChunkGenContext>,
}

/// 待处理的方块写入队列
#[derive(Default, Debug)]
pub struct PendingVoxelWrites {
    /// 该区块内所有待修改的方块列表
    pub writes: HashMap<IVec3, Vec<PendingVoxel>>,
}

/// 单个待处理的方块
#[derive(Debug)]
pub struct PendingVoxel {
    pub local_x: usize,
    pub local_y: usize,
    pub local_z: usize,
    pub block_id: u16,
}
