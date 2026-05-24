use std::collections::HashMap;
use bevy::prelude::*;
use crate::world::chunk::ChunkData;

// 存储世界数据
#[derive(Resource , Default)]
pub struct WorldStorage {
    /// 键是区块的 3D 坐标，值是区块的原生数据，用于存储世界的方块数据
    pub loaded_chunks: HashMap<IVec3, ChunkData>,
    /// 记录哪些区块已经在场景中生成了 Mesh 实体，存储区块对应的渲染实体
    pub chunk_entities: HashMap<IVec3, Entity>,
}