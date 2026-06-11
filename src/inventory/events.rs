use bevy::prelude::*;
use crate::inventory::item::id::ItemId;

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