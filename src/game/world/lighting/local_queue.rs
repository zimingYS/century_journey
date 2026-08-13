//! 管理局部光照任务的有序去重目标队列。

use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::*;

use crate::game::world::state::WorldState;

/// 固定步等待达到该值后强制选择目标，避免交互或邻区流送让普通区块永久饥饿。
pub(super) const LOCAL_LIGHTING_STARVATION_TICKS: u16 = 40;
/// 玩家编辑最后一次入队后等待的固定步数；短窗口用于合并连续挖掘，不能参与普通流送等待。
pub(super) const LOCAL_LIGHTING_EDIT_MERGE_TICKS: u8 = 2;

/// 一次取出的水平列及其最长等待时间。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LightingColumnTarget {
    /// 水平区块列坐标。
    pub column: (i32, i32),
    /// 该列任一目标已经等待的固定步数。
    pub waited_ticks: u16,
    /// 该列是否包含玩家交互提升的目标。
    pub priority: bool,
    /// 该列是否必须重新计算天空光；普通交互只重建方块光。
    pub sky_dirty: bool,
}

impl LightingColumnTarget {
    /// 是否已经达到防饥饿强制调度阈值。
    pub(super) fn is_starved(self) -> bool {
        self.waited_ticks >= LOCAL_LIGHTING_STARVATION_TICKS
    }
}

/// 局部光照候选区块；玩家交互可把相关列提升到普通流送目标之前。
#[derive(Resource, Default)]
pub(super) struct LocalLightingQueue {
    ordered: VecDeque<IVec3>,
    contained: HashSet<IVec3>,
    waited_ticks: HashMap<IVec3, u16>,
    priority_targets: HashSet<IVec3>,
    sky_dirty_targets: HashSet<IVec3>,
    edit_merge_ticks: u8,
}

impl LocalLightingQueue {
    /// 把普通流式更新目标追加到队尾，并按区块坐标去重。
    pub(super) fn enqueue(&mut self, position: IVec3) {
        self.enqueue_with_sky(position, true);
    }

    /// 追加已知不需要天光重建的普通目标（例如方块光边界网格邻居）。
    pub(super) fn enqueue_with_sky(&mut self, position: IVec3, sky_dirty: bool) {
        if self.contained.insert(position) {
            self.ordered.push_back(position);
            self.waited_ticks.insert(position, 0);
        }
        if sky_dirty {
            self.sky_dirty_targets.insert(position);
        }
    }

    /// 重新排队并保留累计等待时间，邻域暂缺不能重置防饥饿计时。
    pub(super) fn requeue(
        &mut self,
        position: IVec3,
        waited_ticks: u16,
        priority: bool,
        sky_dirty: bool,
    ) {
        if priority {
            self.priority_targets.insert(position);
        }
        if sky_dirty {
            self.sky_dirty_targets.insert(position);
        }
        if self.contained.insert(position) {
            if priority {
                self.ordered.push_front(position);
            } else {
                self.ordered.push_back(position);
            }
            self.waited_ticks.insert(position, waited_ticks);
        } else {
            let waited = self.waited_ticks.entry(position).or_default();
            *waited = (*waited).max(waited_ticks);
        }
    }

    /// 把玩家交互目标提升到队首，确保放置和破坏优先响应。
    pub(super) fn prioritize_edit(&mut self, position: IVec3, sky_dirty: bool) {
        if self.contained.contains(&position) {
            self.ordered.retain(|queued| *queued != position);
        } else {
            self.contained.insert(position);
            self.waited_ticks.insert(position, 0);
        }
        self.priority_targets.insert(position);
        if sky_dirty {
            self.sky_dirty_targets.insert(position);
        }
        self.ordered.push_front(position);
    }

    /// 从最后一次方块编辑重新开始短合并窗口。
    pub(super) fn restart_edit_merge_window(&mut self) {
        self.edit_merge_ticks = LOCAL_LIGHTING_EDIT_MERGE_TICKS;
    }

    /// 推进编辑合并窗口；返回 `true` 表示本固定步仍应等待更多编辑。
    pub(super) fn wait_for_edit_merge(&mut self) -> bool {
        // 交互目标必须在本固定步争取任务槽；普通流送仍可利用短窗口合并重复目标。
        if self.has_priority_target() {
            self.edit_merge_ticks = 0;
            return false;
        }
        if self.edit_merge_ticks == 0 {
            return false;
        }
        self.edit_merge_ticks -= 1;
        true
    }

    /// 推进排队年龄；队列有硬上限，因此固定步线性更新的成本可控。
    pub(super) fn age(&mut self) {
        for position in &self.ordered {
            let waited = self.waited_ticks.entry(*position).or_default();
            *waited = waited.saturating_add(1);
        }
    }

    /// 判断是否有目标需要绕过普通任务积压和完整邻域等待。
    pub(super) fn has_starved_target(&self) -> bool {
        self.waited_ticks
            .values()
            .any(|waited| *waited >= LOCAL_LIGHTING_STARVATION_TICKS)
    }

    /// 当前是否还有任何交互提升目标。
    pub(super) fn has_priority_target(&self) -> bool {
        self.priority_targets
            .iter()
            .any(|position| self.contained.contains(position))
    }

    /// 取出指定数量的水平区块列，并移除同列的重复纵向目标。
    pub(super) fn pop_columns(&mut self, limit: usize) -> Vec<LightingColumnTarget> {
        let mut columns = Vec::with_capacity(limit);
        let mut selected = HashMap::<(i32, i32), usize>::new();
        while columns.len() < limit {
            let starved_index = self
                .ordered
                .iter()
                .enumerate()
                .filter(|(_, position)| {
                    self.waited_ticks
                        .get(position)
                        .is_some_and(|waited| *waited >= LOCAL_LIGHTING_STARVATION_TICKS)
                })
                .max_by_key(|(_, position)| self.waited_ticks.get(position).copied().unwrap_or(0))
                .map(|(index, _)| index);
            let position = starved_index
                .and_then(|index| self.ordered.remove(index))
                .or_else(|| self.ordered.pop_front());
            let Some(position) = position else {
                break;
            };
            let priority = self.priority_targets.contains(&position);
            let sky_dirty = self.sky_dirty_targets.contains(&position);
            self.contained.remove(&position);
            self.priority_targets.remove(&position);
            self.sky_dirty_targets.remove(&position);
            let waited_ticks = self.waited_ticks.remove(&position).unwrap_or(0);
            let column = (position.x, position.z);
            if let std::collections::hash_map::Entry::Vacant(entry) = selected.entry(column) {
                entry.insert(columns.len());
                columns.push(LightingColumnTarget {
                    column,
                    waited_ticks,
                    priority,
                    sky_dirty,
                });
            } else if let Some(index) = selected.get(&column).copied() {
                columns[index].waited_ticks = columns[index].waited_ticks.max(waited_ticks);
                columns[index].priority |= priority;
                columns[index].sky_dirty |= sky_dirty;
            }
        }
        self.ordered.retain(|position| {
            let column = (position.x, position.z);
            if let Some(index) = selected.get(&column).copied() {
                columns[index].priority |= self.priority_targets.contains(position);
                columns[index].sky_dirty |= self.sky_dirty_targets.contains(position);
                columns[index].waited_ticks = columns[index]
                    .waited_ticks
                    .max(self.waited_ticks.remove(position).unwrap_or(0));
                self.contained.remove(position);
                self.priority_targets.remove(position);
                self.sky_dirty_targets.remove(position);
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
        self.waited_ticks.clear();
        self.priority_targets.clear();
        self.sky_dirty_targets.clear();
        self.edit_merge_ticks = 0;
    }

    /// 丢弃已经卸载的区块目标，避免后台任务为过期区域工作。
    pub(super) fn retain_loaded(&mut self, world: &WorldState) {
        self.ordered
            .retain(|position| world.contains_chunk(*position));
        self.contained = self.ordered.iter().copied().collect();
        self.waited_ticks
            .retain(|position, _| self.contained.contains(position));
        self.priority_targets
            .retain(|position| self.contained.contains(position));
        self.sky_dirty_targets
            .retain(|position| self.contained.contains(position));
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/lighting/local_queue.rs"]
mod tests;
