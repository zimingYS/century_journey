//! 世界方块变更来源的稀疏记录层。
//!
//! 记录每个被主动变更方块的最近变更来源（`ChangeSource`），供后续生态规则、
//! 天气演变与玩家方块保护等系统消费。会话期态，不进存档；区块卸载时按区块清理。

use crate::shared::voxel::CHUNK_SIZE;
use crate::shared::voxel_change::ChangeSource;
use bevy::math::IVec3;
use bevy::prelude::Resource;
use std::collections::HashMap;

/// 世界坐标到最近变更来源的稀疏映射。
///
/// 仅在方块被主动变更（区别于世界生成）时写入；`source_of` 提供只读查询。
/// 区块卸载时调用 [`VoxelProvenance::remove_chunk`] 清理，避免随探索无限增长。
#[derive(Resource, Debug, Default)]
pub struct VoxelProvenance {
    entries: HashMap<IVec3, ChangeSource>,
}

impl VoxelProvenance {
    /// 记录指定世界坐标的最近变更来源（覆盖旧值）。
    pub fn record(&mut self, position: IVec3, source: ChangeSource) {
        self.entries.insert(position, source);
    }

    /// 查询指定世界坐标的最近变更来源；从未被主动变更返回 `None`。
    pub fn source_of(&self, position: IVec3) -> Option<ChangeSource> {
        self.entries.get(&position).copied()
    }

    /// 移除单个坐标的来源记录并返回旧值。
    pub fn remove(&mut self, position: IVec3) -> Option<ChangeSource> {
        self.entries.remove(&position)
    }

    /// 清理指定区块内所有坐标的来源记录（区块卸载时调用）。
    pub fn remove_chunk(&mut self, chunk_pos: IVec3) {
        self.entries.retain(|position, _| {
            let owner = IVec3::new(
                position.x.div_euclid(CHUNK_SIZE as i32),
                position.y.div_euclid(CHUNK_SIZE as i32),
                position.z.div_euclid(CHUNK_SIZE as i32),
            );
            owner != chunk_pos
        });
    }

    /// 当前记录的条目总数。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否存在任何来源记录。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/voxel_change/provenance.rs"]
mod tests;
