use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::slot::SlotAction;
use crate::shared::item_id::ItemId;
use crate::shared::ui_types::SlotKind;
use bevy::prelude::*;

/// 物品被拾取到鼠标
#[derive(Message)]
pub struct ItemPickedEvent {
    pub item: ItemId,
}

/// 物品被放置到快捷栏
#[derive(Message)]
pub struct ItemPlacedToHotbarEvent {
    pub hotbar_index: usize,
    pub item: ItemId,
}

/// Q 丢弃事件
#[derive(Message, Debug, Clone)]
pub struct DropItemEvent {
    pub stack: ItemStack,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct SlotInteractionEvent {
    pub kind: SlotKind,
    pub index: usize,
    pub action: SlotAction,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryCommand {
    CompactBackpack,
    SortBackpack,
}

/// 只描述物品栏操作结果，供客户端表现层播放提示，不参与物品规则判定。
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryFeedbackEvent {
    Full,
}
