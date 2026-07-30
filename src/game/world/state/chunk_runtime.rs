//! 记录已加载区块、脏标记和异步任务等会话期运行状态。

use crate::game::world::generation::terrain::context::ChunkGenContext;
use bevy::math::IVec3;
use bevy::prelude::{Entity, Resource};
use std::collections::HashMap;

/// 区块实体和生成上下文等不进入存档的会话期索引。
#[derive(Resource, Debug, Default)]
pub struct ChunkRuntime {
    /// 区块坐标到当前会话 ECS 实体的可重建映射。
    chunk_entities: HashMap<IVec3, Entity>,
    /// 地形阶段交给结构阶段使用的临时生成上下文。
    gen_contexts: HashMap<IVec3, ChunkGenContext>,
}

impl ChunkRuntime {
    /// 查询指定区块对应的 ECS 实体。
    pub fn chunk_entity(&self, position: IVec3) -> Option<Entity> {
        self.chunk_entities.get(&position).copied()
    }

    /// 判断指定区块是否已有对应的 ECS 实体。
    pub fn contains_chunk_entity(&self, position: IVec3) -> bool {
        self.chunk_entities.contains_key(&position)
    }

    /// 登记区块与 ECS 实体的运行时映射。
    pub fn register_chunk_entity(&mut self, position: IVec3, entity: Entity) {
        self.chunk_entities.insert(position, entity);
    }

    /// 移除区块与 ECS 实体的运行时映射。
    pub fn remove_chunk_entity(&mut self, position: IVec3) -> Option<Entity> {
        self.chunk_entities.remove(&position)
    }
    /// 缓存地形生成阶段产出的区块上下文，供结构生成阶段复用。
    pub fn cache_generation_context(&mut self, position: IVec3, context: ChunkGenContext) {
        self.gen_contexts.insert(position, context);
    }

    /// 查询指定区块的生成上下文。
    pub fn generation_context(&self, position: IVec3) -> Option<&ChunkGenContext> {
        self.gen_contexts.get(&position)
    }

    /// 在结构生成结束后清除不再需要的生成上下文。
    pub fn remove_generation_context(&mut self, position: IVec3) -> Option<ChunkGenContext> {
        self.gen_contexts.remove(&position)
    }
}
