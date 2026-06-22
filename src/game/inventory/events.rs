use crate::game::inventory::item::id::ItemId;
use crate::game::inventory::item::stack::ItemStack;
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
