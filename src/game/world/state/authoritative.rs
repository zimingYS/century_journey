//! 维护可持久化的权威区块与树木实例，并提供统一快照入口。

use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::structure::pending_writes::{PendingVoxel, PendingVoxelWrites};
use crate::game::world::vegetation::{TreeInstance, TreeInstanceStore};
use bevy::math::IVec3;
use bevy::prelude::Resource;
use std::collections::HashMap;
use std::sync::Arc;

/// 区块卸载时一次性移交给存档领域的权威数据。
#[derive(Debug)]
pub(in crate::game) struct WorldChunkSnapshot {
    /// 区块的不可变体素快照。
    pub data: Arc<ChunkData>,
    /// 根坐标属于该区块的有序树木实例。
    pub tree_instances: Vec<TreeInstance>,
}

/// 当前世界会话的权威区块快照、树木实例、修改时间和生成期延迟写入。
#[derive(Resource, Debug, Default)]
pub struct WorldState {
    /// 当前流送窗口内已加载的权威区块快照。
    loaded_chunks: HashMap<IVec3, Arc<ChunkData>>,
    /// 已加载体素快照的单调修订号，供后台系统用 O(1) 判断全窗口输入是否变化。
    snapshot_revision: u64,
    /// 区块最后修改的 Unix 时间秒数，供增量保存判断使用。
    chunk_modified_times: HashMap<IVec3, f64>,
    /// 结构越过区块边界时等待目标区块加载的确定性延迟写入。
    pending_writes: PendingVoxelWrites,
    /// 按树根所在区块唯一持有的逻辑树实例；不会复制到树冠跨入的区块。
    tree_instances: TreeInstanceStore,
}

impl WorldState {
    /// 读取指定坐标已加载区块的不可变快照。
    pub fn chunk(&self, position: IVec3) -> Option<&Arc<ChunkData>> {
        self.loaded_chunks.get(&position)
    }

    /// 取得可替换的区块快照。
    pub fn chunk_mut(&mut self, position: IVec3) -> Option<&mut Arc<ChunkData>> {
        let chunk = self.loaded_chunks.get_mut(&position)?;
        self.snapshot_revision = self.snapshot_revision.wrapping_add(1);
        Some(chunk)
    }

    /// 判断指定区块是否已加载。
    pub fn contains_chunk(&self, position: IVec3) -> bool {
        self.loaded_chunks.contains_key(&position)
    }

    /// 写入或替换一个已加载区块的权威快照。
    pub fn insert_chunk(&mut self, position: IVec3, data: Arc<ChunkData>) {
        self.loaded_chunks.insert(position, data);
        self.snapshot_revision = self.snapshot_revision.wrapping_add(1);
    }

    /// 恢复磁盘区块及其根区块树实例；校验失败时不替换原有数据。
    pub(in crate::game) fn insert_restored_chunk(
        &mut self,
        position: IVec3,
        data: Arc<ChunkData>,
        tree_instances: Vec<TreeInstance>,
    ) -> Result<(), String> {
        self.tree_instances
            .replace_chunk(position, tree_instances)?;
        self.loaded_chunks.insert(position, data);
        self.snapshot_revision = self.snapshot_revision.wrapping_add(1);
        Ok(())
    }

    /// 移除区块并同时返回体素与根区块树实例，供卸载保存流程使用。
    pub(in crate::game) fn remove_chunk(&mut self, position: IVec3) -> Option<WorldChunkSnapshot> {
        let data = self.loaded_chunks.remove(&position)?;
        self.snapshot_revision = self.snapshot_revision.wrapping_add(1);
        let tree_instances = self.tree_instances.take_chunk(position);
        Some(WorldChunkSnapshot {
            data,
            tree_instances,
        })
    }

    /// 返回当前已加载区块数量，供客户端统计使用。
    pub fn loaded_chunk_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    /// 返回已加载体素快照的当前修订号。
    pub fn snapshot_revision(&self) -> u64 {
        self.snapshot_revision
    }

    /// 遍历所有已加载区块的权威快照，供全量存档等批处理流程读取。
    pub fn chunks(&self) -> impl Iterator<Item = (IVec3, &Arc<ChunkData>)> {
        self.loaded_chunks
            .iter()
            .map(|(position, data)| (*position, data))
    }
    /// 克隆指定已加载区块的体素与树实例快照，供异步存档取得独立所有权。
    pub(in crate::game) fn chunk_snapshot(&self, position: IVec3) -> Option<WorldChunkSnapshot> {
        let data = Arc::clone(self.loaded_chunks.get(&position)?);
        Some(WorldChunkSnapshot {
            data,
            tree_instances: self.tree_instances.snapshot_chunk(position),
        })
    }

    /// 插入新树实例；树根区块未加载或根坐标重复时拒绝写入。
    pub(in crate::game::world) fn insert_tree_instance(
        &mut self,
        instance: TreeInstance,
    ) -> Result<(), String> {
        if !self.contains_chunk(instance.owner_chunk()) {
            return Err(format!("树根 {:?} 所属区块尚未加载", instance.root()));
        }
        self.tree_instances.insert(instance)
    }

    /// 返回指定树根坐标的逻辑实例。
    pub(in crate::game) fn tree_instance(&self, root: IVec3) -> Option<&TreeInstance> {
        self.tree_instances.get(root)
    }

    /// 返回指定树根的可变实例，仅供世界生命周期在体素提交后更新元数据。
    pub(in crate::game::world) fn tree_instance_mut(
        &mut self,
        root: IVec3,
    ) -> Option<&mut TreeInstance> {
        self.tree_instances.get_mut(root)
    }

    /// 返回当前已加载世界中到达生命周期结算时间的有序树根。
    pub(in crate::game::world) fn due_tree_roots(&self, game_minute: u64) -> Vec<IVec3> {
        self.tree_instances.due_roots(game_minute)
    }

    /// 删除指定树根坐标的逻辑实例。
    pub(in crate::game::world) fn remove_tree_instance(
        &mut self,
        root: IVec3,
    ) -> Option<TreeInstance> {
        self.tree_instances.remove(root)
    }

    /// 标记区块发生修改，并记录该修改对应的时间。
    pub fn mark_chunk_modified(&mut self, position: IVec3, modified_time: f64) {
        self.chunk_modified_times.insert(position, modified_time);
    }

    /// 查询区块最近一次修改时间。
    pub fn chunk_modified_time(&self, position: IVec3) -> Option<f64> {
        self.chunk_modified_times.get(&position).copied()
    }

    /// 清理指定区块的脏区块标记。
    pub fn clear_chunk_modified(&mut self, position: IVec3) {
        self.chunk_modified_times.remove(&position);
    }

    /// 取走全部脏区块标记。
    pub fn take_modified_chunks(&mut self) -> Vec<(IVec3, f64)> {
        std::mem::take(&mut self.chunk_modified_times)
            .into_iter()
            .collect()
    }
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

    /// 取走指定区块等待应用的全部写入。
    pub fn take_pending_writes(&mut self, position: IVec3) -> Option<Vec<PendingVoxel>> {
        self.pending_writes.writes.remove(&position)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/state/authoritative.rs"]
mod tests;
