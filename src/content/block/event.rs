//! 定义方块领域可跨系统传递的内容事件。

use bevy::prelude::*;

/// 方块被破坏事件
#[derive(Message)]
pub struct BlockBreakEvent {
    /// 被破坏方块的世界坐标
    pub world_pos: IVec3,
    /// 被破坏方块的运行时ID
    pub block_id: u16,
    /// 破坏者实体
    pub breaker: Option<Entity>,
}

/// 方块被放置事件
#[derive(Message)]
pub struct BlockPlaceEvent {
    /// 被放置方块的世界坐标
    pub world_pos: IVec3,
    /// 被放置方块的运行时ID
    pub block_id: u16,
    /// 放置面法线
    pub face_normal: IVec3,
    /// 放置者实体
    pub placer: Option<Entity>,
}

/// 方块被右键交互事件
#[derive(Message)]
pub struct BlockInteractEvent {
    /// 被交互方块的世界坐标
    pub world_pos: IVec3,
    /// 被交互方块的运行时ID
    pub block_id: u16,
    /// 交互面法线
    pub face_normal: IVec3,
    /// 交互者实体
    pub interactor: Option<Entity>,
}

/// 方块状态变更事件
#[derive(Message)]
pub struct BlockStateChangeEvent {
    /// 方块世界坐标
    pub world_pos: IVec3,
    /// 方块运行时ID
    pub block_id: u16,
    /// 旧状态索引
    pub old_state: u16,
    /// 新状态索引
    pub new_state: u16,
}
/// 方块写入成功后的权威状态变更。
///
/// 该消息只描述已提交的旧、新运行时 ID；世界规则据此刷新区块表现并通知相邻方块，
/// 不应由客户端表现层直接伪造。
#[derive(Message, Debug, Clone, Copy)]
pub struct BlockChangedEvent {
    /// 被写入方块的世界坐标。
    pub world_pos: IVec3,
    /// 写入前的运行时方块 ID。
    pub old_block_id: u16,
    /// 写入后的运行时方块 ID。
    pub new_block_id: u16,
}

/// 由相邻方块变更引发的邻居通知。
///
/// `neighbor_pos` 是需要重新评估的方块，而 `changed_pos` 是刚完成写入的方块；
/// 两者分离可支持支撑、连接和红石类规则，而无需在写入处硬编码具体玩法。
#[derive(Message, Debug, Clone, Copy)]
pub struct BlockNeighborChangedEvent {
    /// 需要响应邻居变化的方块坐标。
    pub neighbor_pos: IVec3,
    /// 已发生写入的相邻方块坐标。
    pub changed_pos: IVec3,
    /// 相邻方块写入前的运行时 ID。
    pub old_block_id: u16,
    /// 相邻方块写入后的运行时 ID。
    pub new_block_id: u16,
}
