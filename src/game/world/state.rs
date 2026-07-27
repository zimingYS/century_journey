//! World state ownership boundaries.
//!
//! `WorldState` is authoritative simulation data. `ChunkRuntime` is disposable
//! streaming/task state. Client mesh entities live in the client module.

use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::context::ChunkGenContext;
use crate::game::world::pending_writes::{PendingVoxel, PendingVoxelWrites};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Resource, Debug, Default)]
pub struct WorldState {
    loaded_chunks: HashMap<IVec3, Arc<ChunkData>>,
    chunk_modified_times: HashMap<IVec3, f64>,
    pending_writes: PendingVoxelWrites,
}

impl WorldState {
    //# ---loaded_chunks
    /// 读取指定坐标已加载区块的不可变快照
    pub fn chunk(&self, position: IVec3) -> Option<&Arc<ChunkData>> {
        self.loaded_chunks.get(&position)
    }

    /// 取得可替换的区块快照
    pub fn chunk_mut(&mut self, position: IVec3) -> Option<&mut Arc<ChunkData>> {
        self.loaded_chunks.get_mut(&position)
    }

    /// 判断指定区块是否已加载
    pub fn contains_chunk(&self, position: IVec3) -> bool {
        self.loaded_chunks.contains_key(&position)
    }

    /// 写入或替换一个已加载区块的权威快照
    pub fn insert_chunk(&mut self, position: IVec3, data: Arc<ChunkData>) {
        self.loaded_chunks.insert(position, data);
    }

    /// 移除区块并返回其最后快照，供卸载保存流程使用
    pub fn remove_chunk(&mut self, position: IVec3) -> Option<Arc<ChunkData>> {
        self.loaded_chunks.remove(&position)
    }

    /// 返回当前已加载区块数量，供客户端统计使用
    pub fn loaded_chunk_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    /// 遍历所有已加载区块的权威快照，供全量存档等批处理流程读取
    pub fn chunks(&self) -> impl Iterator<Item = (IVec3, &Arc<ChunkData>)> {
        self.loaded_chunks
            .iter()
            .map(|(position, data)| (*position, data))
    }
    //# ---

    //# ---chunk_modified_times
    /// 标记区块发生修改，并记录该修改对应的时间
    pub fn mark_chunk_modified(&mut self, position: IVec3, modified_time: f64) {
        self.chunk_modified_times.insert(position, modified_time);
    }

    /// 查询区块最近一次修改时间
    pub fn chunk_modified_time(&self, position: IVec3) -> Option<f64> {
        self.chunk_modified_times.get(&position).copied()
    }

    /// 清理指定区块的脏区块标记
    pub fn clear_chunk_modified(&mut self, position: IVec3) {
        self.chunk_modified_times.remove(&position);
    }

    /// 取走全部脏区块标记
    pub fn take_modified_chunks(&mut self) -> Vec<(IVec3, f64)> {
        std::mem::take(&mut self.chunk_modified_times)
            .into_iter()
            .collect()
    }
    //# ---

    //# ---pending_writes
    /// 合并结构生成任务产生的跨区块延迟写入。
    pub fn queue_pending_writes(&mut self, position: IVec3, writes: Vec<PendingVoxel>) {
        if writes.is_empty() {
            return;
        }

        self.pending_writes
            .writes
            .entry(position)
            .or_default()
            .extend(writes);
    }

    /// 取走指定区块等待应用的全部写入
    pub fn take_pending_writes(&mut self, position: IVec3) -> Option<Vec<PendingVoxel>> {
        self.pending_writes.writes.remove(&position)
    }
    //# ---
}

#[derive(Resource, Debug, Default)]
pub struct ChunkRuntime {
    chunk_entities: HashMap<IVec3, Entity>,
    gen_contexts: HashMap<IVec3, ChunkGenContext>,
}

impl ChunkRuntime {
    //# ---chunk_entities
    /// 查询制定区块对应的ECS实体
    pub fn chunk_entity(&self, position: IVec3) -> Option<Entity> {
        self.chunk_entities.get(&position).copied()
    }

    /// 判断指定区块是否已有对应的ECS实体
    pub fn contains_chunk_entity(&self, position: IVec3) -> bool {
        self.chunk_entities.contains_key(&position)
    }

    /// 登记区块与ECS实体的运行时映射
    pub fn register_chunk_entity(&mut self, position: IVec3, entity: Entity) {
        self.chunk_entities.insert(position, entity);
    }

    /// 移除区块与ECS实体的运行时映射
    pub fn remove_chunk_entity(&mut self, position: IVec3) -> Option<Entity> {
        self.chunk_entities.remove(&position)
    }
    //# ---

    //# ---gen_contexts
    /// 缓存地形生成阶段产出的区块上下文，供结构生成阶段复用
    pub fn cache_generation_context(&mut self, position: IVec3, context: ChunkGenContext) {
        self.gen_contexts.insert(position, context);
    }

    /// 查询指定区块的生成上下文
    pub fn generation_context(&self, position: IVec3) -> Option<&ChunkGenContext> {
        self.gen_contexts.get(&position)
    }

    /// 在结构生成结束后清除不再需要的生成上下文
    pub fn remove_generation_context(&mut self, position: IVec3) -> Option<ChunkGenContext> {
        self.gen_contexts.remove(&position)
    }
    //# ---
}

pub struct HeadlessWorldPlugin;

impl Plugin for HeadlessWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldState>()
            .init_resource::<ChunkRuntime>()
            .init_resource::<crate::game::block::BlockBehaviorRegistry>()
            .insert_resource(Time::<Fixed>::from_hz(
                crate::game::world::time::SIMULATION_TICKS_PER_SECOND as f64,
            ))
            .add_plugins(crate::game::simulation::SimulationPlugin)
            .add_plugins(crate::game::gameplay::GameplayPlugin)
            .add_plugins(crate::game::inventory::plugin::InventoryPlugin)
            .add_plugins(crate::game::player::plugin::GamePlayerPlugin);
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/game/world/state.rs"]
mod tests;
