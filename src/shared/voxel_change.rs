//! 世界方块变更命令的纯数据类型（跨层共享）。
//!
//! 定义变更命令与保序缓冲。应用逻辑（apply）在 Game 层，
//! 此处仅存放 Content/Game/Client 共享的数据结构。

use bevy::prelude::*;

/// 世界方块变更的来源（Provenance 记录 + 生态规则消费）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeSource {
    Player,
    WorldGen,
    Vegetation,
    Weather,
    Hydrology,
    Ecology,
    Fire,
}

/// 一次待应用的世界方块变更（推入 buffer，由 apply 统一应用）。
#[derive(Debug, Clone, Copy)]
pub struct VoxelChange {
    pub pos: IVec3,
    pub block_id: u16,
    pub source: ChangeSource,
}

/// 保序的变更命令缓冲：提交顺序 = 应用顺序（确定性）。
#[derive(Resource, Default)]
pub struct VoxelChangeBuffer(pub Vec<VoxelChange>);
