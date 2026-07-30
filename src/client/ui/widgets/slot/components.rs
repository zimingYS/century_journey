//! 定义槽位 UI 使用的标记组件和本地交互消息。

use crate::game::inventory::slot::SlotKind;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;

/// 槽位
#[derive(Component, Debug, Clone, Copy)]
pub struct InventorySlot {
    /// 槽位对应的逻辑区域。
    pub kind: SlotKind,
    /// 槽位在该区域内的局部索引。
    pub index: usize,
}

/// 槽位图标子实体标记
#[derive(Component)]
pub struct SlotIcon;

/// 槽位数量文本子实体标记
#[derive(Component)]
pub struct SlotCountText;

/// 空装备/饰品槽中的短占位标记。
#[derive(Component)]
pub struct SlotPlaceholder;

/// 绑定逻辑槽位的耐久条根节点。
#[derive(Component, Debug, Clone, Copy)]
pub struct SlotDurabilityBar {
    /// 槽位对应的逻辑区域。
    pub kind: SlotKind,
    /// 槽位在该区域内的局部索引。
    pub index: usize,
}

/// 耐久条中表示剩余比例的填充节点。
#[derive(Component)]
pub struct SlotDurabilityFill;

/// 槽位的视觉状态缓存
#[derive(Component, Debug, Clone, Default)]
pub struct SlotVisual {
    pub item: ItemId,
    pub count: u32,
}

/// 分类标签按钮
#[derive(Component, Debug, Clone, Copy)]
pub struct CategoryTab {
    pub category_index: usize,
}

/// 搜索框标记
#[derive(Component)]
pub struct CreativeSearchInput;

pub use crate::game::inventory::events::SlotInteractionEvent;

/// 分类切换事件
#[derive(Message, Debug)]
pub struct CategoryClickedEvent {
    pub category_index: usize,
}
