use bevy::prelude::*;
use crate::inventory::container::creative::CreativeData;
use crate::inventory::container::hotbar::HotbarData;
use crate::inventory::cursor::CursorData;
use crate::inventory::recent::RecentItems;

/// 统一的物品栏状态资源
#[derive(Resource, Debug)]
pub struct InventoryState {
    /// 快捷栏
    pub hotbar: HotbarData,
    /// 创造模式数据
    pub creative: CreativeData,
    /// 鼠标悬浮物品
    pub cursor: CursorData,
    /// 最近使用
    pub recent: RecentItems,
    /// 任意物品栏界面是否打开
    pub opened: bool,
}

impl Default for InventoryState {
    fn default() -> Self {
        Self {
            hotbar: HotbarData::default(),
            creative: CreativeData::default(),
            cursor: CursorData::default(),
            recent: RecentItems::default(),
            opened: false,
        }
    }
}

impl InventoryState {
    /// 切换物品栏打开状态
    pub fn toggle(&mut self) {
        self.opened = !self.opened;
    }

    /// 添加最近使用物品
    pub fn add_recent(&mut self, item: crate::inventory::item::id::ItemId) {
        self.recent.push(item);
    }
}