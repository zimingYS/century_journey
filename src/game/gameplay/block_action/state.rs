//! 保存持续破坏方块所需的目标、工具和累计进度。

use crate::shared::item_id::ItemId;
use bevy::math::IVec3;
use bevy::prelude::Resource;

/// 客户端可读取的当前方块破坏进度快照。
#[derive(Resource, Debug, Clone)]
pub struct BlockBreakProgress {
    pub visible: bool,
    pub world_pos: IVec3,
    pub block_id: u16,
    pub progress: f32,
}

impl Default for BlockBreakProgress {
    fn default() -> Self {
        Self {
            visible: false,
            world_pos: IVec3::ZERO,
            block_id: 0,
            progress: 0.0,
        }
    }
}

impl BlockBreakProgress {
    /// 隐藏进度并清除当前目标。
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// 发布指定目标的规范化破坏进度。
    pub fn set(&mut self, world_pos: IVec3, block_id: u16, progress: f32) {
        self.visible = true;
        self.world_pos = world_pos;
        self.block_id = block_id;
        self.progress = progress.clamp(0.0, 1.0);
    }
}

/// 固定步中用于累计同一方块和工具组合破坏时间的权威状态。
#[derive(Resource, Debug, Default, Clone)]
pub struct BlockBreakState {
    target: Option<BlockBreakTarget>,
    elapsed_seconds: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockBreakTarget {
    world_pos: IVec3,
    block_id: u16,
    tool_item: ItemId,
}

impl BlockBreakState {
    /// 清除当前破坏目标和累计时间。
    pub fn clear(&mut self) {
        self.target = None;
        self.elapsed_seconds = 0.0;
    }

    /// 累计当前目标的破坏时间；目标或工具变化时自动重新计时。
    pub fn tick(&mut self, world_pos: IVec3, block_id: u16, tool_item: &ItemId, delta: f32) -> f32 {
        let next_target = BlockBreakTarget {
            world_pos,
            block_id,
            tool_item: tool_item.clone(),
        };

        if self.target.as_ref() != Some(&next_target) {
            self.target = Some(next_target);
            self.elapsed_seconds = 0.0;
        }

        self.elapsed_seconds += delta.max(0.0);
        self.elapsed_seconds
    }
}
