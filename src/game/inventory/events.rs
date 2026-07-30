//! 定义库存领域接收的命令和对外发布的物品操作消息。

use crate::game::inventory::container::world::ContainerId;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::slot::SlotKind;
use crate::game::player::identity::PlayerId;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;

/// 物品被拾取到鼠标
#[derive(Message)]
pub struct ItemPickedEvent {
    /// 被拾取的物品 ID。
    pub item: ItemId,
}

/// 物品被放置到快捷栏
#[derive(Message)]
pub struct ItemPlacedToHotbarEvent {
    /// 接收物品的快捷栏索引。
    pub hotbar_index: usize,
    /// 被放入快捷栏的物品 ID。
    pub item: ItemId,
}

/// Q 丢弃事件
#[derive(Message, Debug, Clone)]
pub struct DropItemEvent {
    /// 发起丢弃的稳定玩家 ID。
    pub player_id: PlayerId,
    /// 要生成到世界中的物品堆。
    pub stack: ItemStack,
}

#[derive(Message, Debug, Clone, Copy)]
/// 客户端提交的单次槽位交互意图。
pub struct SlotInteractionEvent {
    /// 发起操作的稳定玩家 ID。
    pub player_id: PlayerId,
    /// 世界容器 ID；玩家自身槽位为空。
    pub container_id: Option<ContainerId>,
    /// 被操作槽位所属的逻辑区域。
    pub kind: SlotKind,
    /// 槽位在对应逻辑区域内的索引。
    pub index: usize,
    /// 需要由权威规则解释的操作类型。
    pub action: SlotAction,
}

/// 客户端提交给权威物品栏的操作意图。
///
/// Client 只负责采集输入；Game 在固定步中校验玩家状态并修改物品栏。
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryCommand {
    /// 打开当前玩家的物品栏。
    Open,
    /// 关闭物品栏，并安全归还光标上的物品。
    Close,
    /// 在打开和关闭状态之间切换。
    Toggle,
    /// 压缩生存背包中的同类堆叠。
    CompactBackpack,
    /// 按物品 ID 对生存背包排序。
    SortBackpack,
}
/// 只描述物品栏操作结果，供客户端表现层播放提示，不参与物品规则判定。
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryFeedbackEvent {
    /// 操作因库存没有可用容量而失败。
    Full,
}
