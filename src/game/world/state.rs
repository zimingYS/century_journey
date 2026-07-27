//! World state ownership boundaries.
//!
//! `WorldState` is authoritative simulation data. `ChunkRuntime` is disposable
//! streaming/task state. Client mesh entities live in the client module.

use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::context::ChunkGenContext;
use crate::game::world::pending_writes::PendingVoxelWrites;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Resource, Debug, Default)]
pub struct WorldState {
    loaded_chunks: HashMap<IVec3, Arc<ChunkData>>,
    pub chunk_modified_times: HashMap<IVec3, f64>,
    pub pending_writes: PendingVoxelWrites,
    pub block_entities: HashMap<IVec3, Entity>,
}

impl WorldState {
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
}

#[derive(Resource, Debug, Default)]
pub struct ChunkRuntime {
    pub chunk_entities: HashMap<IVec3, Entity>,
    pub gen_contexts: HashMap<IVec3, ChunkGenContext>,
    pub last_chunk_pos: Option<IVec3>,
    pub expected_chunks: HashSet<IVec3>,
    pub terrain_tasks_in_flight: usize,
    pub structure_tasks_in_flight: usize,
    pub mesh_tasks_in_flight: usize,
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
