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

// /// 方块tick事件
// #[derive(Message)]
// pub struct BlockTickEvent {
//     /// 方块世界坐标
//     pub world_pos: IVec3,
//     /// 方块运行时ID
//     pub block_id: u16,
// }