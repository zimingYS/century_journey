use crate::game::inventory::slot::SlotAction;
use crate::shared::item_id::ItemId;
use crate::shared::ui_types::SlotKind;
use bevy::prelude::*;

/// 槽位
#[derive(Component, Debug, Clone, Copy)]
pub struct InventorySlot {
    pub kind: SlotKind,
    pub index: usize,
}

/// 槽位图标子实体标记
#[derive(Component)]
pub struct SlotIcon;

/// 槽位数量文本子实体标记
#[derive(Component)]
pub struct SlotCountText;

/// 槽位的视觉状态缓存
#[derive(Component, Debug, Clone)]
pub struct SlotVisual {
    pub item: ItemId,
    pub count: u32,
}

impl Default for SlotVisual {
    fn default() -> Self {
        Self {
            item: ItemId::air(),
            count: 0,
        }
    }
}

/// 分类标签按钮
#[derive(Component, Debug, Clone, Copy)]
pub struct CategoryTab {
    pub category_index: usize,
}

/// 搜索框标记
#[derive(Component)]
pub struct CreativeSearchInput;

/// 槽位点击事件
#[derive(Message, Debug)]
pub struct SlotInteractionEvent {
    pub kind: SlotKind,
    pub index: usize,
    pub action: SlotAction,
}

/// 分类切换事件
#[derive(Message, Debug)]
pub struct CategoryClickedEvent {
    pub category_index: usize,
}
