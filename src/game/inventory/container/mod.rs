pub mod creative;
pub mod hotbar;
pub mod survival;

use crate::game::inventory::item::stack::ItemStack;
use crate::shared::ui_types::ContainerKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerLayout {
    pub columns: usize,
    pub rows: usize,
}

impl ContainerLayout {
    pub const fn new(columns: usize, rows: usize) -> Self {
        Self { columns, rows }
    }

    pub const fn slot_count(self) -> usize {
        self.columns * self.rows
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerSlotRole {
    Storage,
    Input,
    Output,
    Fuel,
}

/// 统一容器接口
/// 所有容器（Hotbar、Backpack、Chest、Furnace 等）均实现此 trait。
/// 只定义数据访问，不包含 UI 或交互逻辑。
pub trait InventoryContainer {
    /// 容器总槽位数
    fn slot_count(&self) -> usize;

    /// 获取指定槽位的物品堆叠只读引用
    fn get_stack(&self, index: usize) -> Option<&ItemStack>;

    /// 获取指定槽位的物品堆叠可变引用
    fn get_stack_mut(&mut self, index: usize) -> Option<&mut ItemStack>;

    /// 无条件设置指定槽位（空槽位也生效）
    ///
    /// 必须由实现者覆盖，因为 `get_stack_mut` 对空槽位返回 None。
    fn set_stack(&mut self, index: usize, stack: ItemStack);

    /// 将指定槽位替换为新堆叠，返回旧堆叠
    ///
    /// 仅当槽位已有物品时生效；空槽位请使用 `set_stack`。
    fn replace_stack(&mut self, index: usize, stack: ItemStack) -> Option<ItemStack> {
        let slot = self.get_stack_mut(index)?;
        if stack.is_empty() {
            Some(std::mem::take(slot))
        } else {
            Some(std::mem::replace(slot, stack))
        }
    }
}

/// 带布局和槽位语义的通用容器接口。
///
/// 背包等旧容器继续只需实现 `InventoryContainer`；工作台以及后续箱子、熔炉
/// 使用此接口向交互和 UI 层公开统一的容器元数据。
pub trait GameContainer: InventoryContainer {
    fn kind(&self) -> ContainerKind;

    fn layout(&self) -> ContainerLayout;

    fn slot_role(&self, _index: usize) -> ContainerSlotRole {
        ContainerSlotRole::Storage
    }

    fn can_insert(&self, index: usize, _stack: &ItemStack) -> bool {
        index < self.slot_count() && self.slot_role(index) != ContainerSlotRole::Output
    }

    fn can_extract(&self, index: usize) -> bool {
        index < self.slot_count()
    }
}
