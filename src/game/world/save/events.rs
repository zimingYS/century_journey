use bevy::prelude::*;

/// 脏数据来源枚举
#[derive(Debug, Clone, Copy, Message)]
pub enum SaveDirtySource {
    /// 背包变化
    Inventory,
    /// 玩家位置变化
    Position,
    /// 游戏模式变化
    GameMode,
    /// 统计数据变化 (预留)
    Stats,
    /// 区块变化 (预留)
    Chunk,
}

/// 脏数据通知事件
#[derive(Debug, Clone, Message)]
pub struct SaveDirtyEvent {
    pub source: SaveDirtySource,
}
