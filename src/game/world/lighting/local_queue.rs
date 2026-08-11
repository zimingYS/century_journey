//! 管理局部光照任务的有序去重目标队列。

use std::collections::{HashSet, VecDeque};

use bevy::prelude::*;

use crate::game::world::state::WorldState;

/// 局部光照候选区块；玩家交互可把相关列提升到普通流送目标之前。
#[derive(Resource, Default)]
pub(super) struct LocalLightingQueue {
    ordered: VecDeque<IVec3>,
    contained: HashSet<IVec3>,
    pub(super) interaction_pending: bool,
}

impl LocalLightingQueue {
    /// 把普通流式更新目标追加到队尾，并按区块坐标去重。
    pub(super) fn enqueue(&mut self, position: IVec3) {
        if self.contained.insert(position) {
            self.ordered.push_back(position);
        }
    }

    /// 把玩家交互目标提升到队首，确保放置和破坏优先响应。
    pub(super) fn prioritize(&mut self, position: IVec3) {
        if self.contained.contains(&position) {
            self.ordered.retain(|queued| *queued != position);
        } else {
            self.contained.insert(position);
        }
        self.ordered.push_front(position);
    }

    /// 取出指定数量的水平区块列，并移除同列的重复纵向目标。
    pub(super) fn pop_columns(&mut self, limit: usize) -> Vec<(i32, i32)> {
        let mut columns = Vec::with_capacity(limit);
        let mut selected = HashSet::new();
        while columns.len() < limit {
            let Some(position) = self.ordered.pop_front() else {
                break;
            };
            self.contained.remove(&position);
            let column = (position.x, position.z);
            if selected.insert(column) {
                columns.push(column);
            }
        }
        self.ordered.retain(|position| {
            if selected.contains(&(position.x, position.z)) {
                self.contained.remove(position);
                false
            } else {
                true
            }
        });
        columns
    }

    /// 返回尚未调度的去重目标数量。
    pub(super) fn len(&self) -> usize {
        self.ordered.len()
    }

    /// 清空跨世界会话不得保留的排队状态。
    pub(super) fn clear(&mut self) {
        self.ordered.clear();
        self.contained.clear();
        self.interaction_pending = false;
    }

    /// 丢弃已经卸载的区块目标，避免后台任务为过期区域工作。
    pub(super) fn retain_loaded(&mut self, world: &WorldState) {
        self.ordered
            .retain(|position| world.contains_chunk(*position));
        self.contained = self.ordered.iter().copied().collect();
        if self.ordered.is_empty() {
            self.interaction_pending = false;
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/lighting/local_queue.rs"]
mod tests;
