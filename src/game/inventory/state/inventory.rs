//! 聚合单个玩家的快捷栏、生存背包、创造目录和交互会话状态。

use crate::game::inventory::container::creative::CreativeData;
use crate::game::inventory::container::hotbar::HotbarData;
use crate::game::inventory::container::survival::SurvivalInventory;
use crate::game::inventory::state::cursor::CursorData;
use crate::game::inventory::state::recent::RecentItems;
use bevy::prelude::*;

use crate::game::player::identity::LocalPlayer;

/// 统一的物品栏状态资源
#[derive(Component, Debug, Default)]
pub struct InventoryState {
    /// 快捷栏
    pub hotbar: HotbarData,
    /// 创造模式数据
    pub creative: CreativeData,
    /// 生存模式背包
    pub survival: SurvivalInventory,
    /// 拖拽期间由光标持有的物品
    pub cursor: CursorData,
    /// 最近使用
    pub recent: RecentItems,
    /// 任意物品栏界面是否打开
    pub opened: bool,
}

/// 查询本地玩家只读库存的单实体系统参数。
pub type LocalInventory<'w, 's> = Single<'w, 's, &'static InventoryState, With<LocalPlayer>>;
/// 查询本地玩家可变库存的单实体系统参数。
pub type LocalInventoryMut<'w, 's> = Single<'w, 's, &'static mut InventoryState, With<LocalPlayer>>;

impl InventoryState {
    /// 切换物品栏打开状态
    pub fn toggle(&mut self) {
        self.opened = !self.opened;
    }

    /// 添加最近使用物品堆叠
    pub fn add_recent_stack(&mut self, stack: crate::game::inventory::item::stack::ItemStack) {
        self.recent.push_stack(stack);
    }
}
