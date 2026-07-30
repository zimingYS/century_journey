//! 定义合成台会话打开等跨系统领域消息。

use crate::game::inventory::container::world::ContainerId;
use crate::game::player::identity::PlayerId;
use bevy::prelude::*;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
/// 玩家成功打开一个世界合成台后发布的领域消息。
pub struct CraftingStationOpened {
    /// 打开合成台的稳定玩家 ID。
    pub player_id: PlayerId,
    /// 本次会话绑定的世界容器 ID。
    pub container_id: ContainerId,
    /// 合成台方块的世界坐标。
    pub position: IVec3,
}
