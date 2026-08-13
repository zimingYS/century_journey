//! 维护当前流送窗口内可生长树苗的稀疏候选索引。

use crate::content::block::event::BlockChangedEvent;
use crate::content::vegetation::registry::TreeSpeciesRegistry;
use crate::game::world::chunk::{ChunkData, ChunkState};
use crate::game::world::state::{ChunkRuntime, WorldState};
use crate::shared::voxel::CHUNK_SIZE;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// 当前会话内已完成扫描的区块和仍可能生长的树苗。
#[derive(Resource, Debug, Default)]
pub(super) struct VegetationRuntime {
    indexed_chunks: HashSet<IVec3>,
    candidates: HashMap<IVec3, u16>,
}

impl VegetationRuntime {
    /// 返回按世界坐标排序的候选快照，消除 HashMap 遍历顺序对模拟的影响。
    pub(super) fn sorted_candidates(&self) -> Vec<(IVec3, u16)> {
        let mut candidates = self
            .candidates
            .iter()
            .map(|(&position, &block_id)| (position, block_id))
            .collect::<Vec<_>>();
        candidates.sort_by_key(|(position, _)| (position.x, position.y, position.z));
        candidates
    }

    /// 从候选索引移除已变化或完成生长的方块。
    pub(super) fn remove_candidate(&mut self, position: IVec3) {
        self.candidates.remove(&position);
    }
}

/// 进入世界时清空上一会话的植被索引。
pub(super) fn reset_vegetation_runtime_system(mut runtime: ResMut<VegetationRuntime>) {
    *runtime = VegetationRuntime::default();
}

/// 根据统一方块变更消息增删树苗候选，无需依赖玩家放置的具体来源。
pub(super) fn track_growth_block_changes_system(
    mut changes: MessageReader<BlockChangedEvent>,
    species_registry: Res<TreeSpeciesRegistry>,
    mut runtime: ResMut<VegetationRuntime>,
) {
    for change in changes.read() {
        if species_registry
            .get_by_sapling_id(change.new_block_id)
            .is_some()
        {
            runtime
                .candidates
                .insert(change.world_pos, change.new_block_id);
        } else {
            runtime.candidates.remove(&change.world_pos);
        }
    }
}

/// 扫描刚完成结构阶段的已加载区块，恢复存档和重新流送区块中的树苗候选。
pub(super) fn index_loaded_growth_blocks_system(
    world_state: Res<WorldState>,
    chunk_runtime: Res<ChunkRuntime>,
    chunk_states: Query<&ChunkState>,
    species_registry: Res<TreeSpeciesRegistry>,
    mut runtime: ResMut<VegetationRuntime>,
) {
    let loaded_chunks = world_state
        .chunks()
        .map(|(position, _)| position)
        .collect::<HashSet<_>>();
    runtime
        .indexed_chunks
        .retain(|position| loaded_chunks.contains(position));
    runtime
        .candidates
        .retain(|position, _| loaded_chunks.contains(&world_to_chunk_position(*position)));

    let mut unindexed = world_state
        .chunks()
        .filter(|(position, _)| !runtime.indexed_chunks.contains(position))
        .filter(|(position, _)| chunk_is_ready(*position, &chunk_runtime, &chunk_states))
        .map(|(position, data)| (position, Arc::clone(data)))
        .collect::<Vec<(IVec3, Arc<ChunkData>)>>();
    unindexed.sort_by_key(|(position, _)| (position.x, position.y, position.z));

    for (chunk_position, chunk) in unindexed {
        index_chunk_saplings(
            chunk_position,
            &chunk,
            &species_registry,
            &mut runtime.candidates,
        );
        runtime.indexed_chunks.insert(chunk_position);
    }
}

/// 判断区块数据是否已经越过结构写入阶段且没有正在构建旧网格快照。
pub(super) fn chunk_is_ready(
    chunk_position: IVec3,
    chunk_runtime: &ChunkRuntime,
    chunk_states: &Query<&ChunkState>,
) -> bool {
    let Some(entity) = chunk_runtime.chunk_entity(chunk_position) else {
        return false;
    };
    chunk_states
        .get(entity)
        .is_ok_and(|state| state.has_completed_structure())
}

/// 把世界方块坐标转换为所属区块坐标。
pub(super) fn world_to_chunk_position(world_position: IVec3) -> IVec3 {
    IVec3::new(
        world_position.x.div_euclid(CHUNK_SIZE as i32),
        world_position.y.div_euclid(CHUNK_SIZE as i32),
        world_position.z.div_euclid(CHUNK_SIZE as i32),
    )
}

fn index_chunk_saplings(
    chunk_position: IVec3,
    chunk: &ChunkData,
    species_registry: &TreeSpeciesRegistry,
    candidates: &mut HashMap<IVec3, u16>,
) {
    let origin = chunk_position * CHUNK_SIZE as i32;
    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let block_id = chunk.get_voxel(x, y, z);
                if species_registry.get_by_sapling_id(block_id).is_none() {
                    continue;
                }
                let world_position = origin + IVec3::new(x as i32, y as i32, z as i32);
                candidates.insert(world_position, block_id);
            }
        }
    }
}
