pub mod hotbar;
pub mod creative;
pub mod player_inventory;
pub mod creative_inventory;
pub mod container;
pub mod interaction;
pub mod survival;

use crate::game::inventory::item::stack::ItemStack;

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
            Some(std::mem::replace(slot, ItemStack::empty()))
        } else {
            Some(std::mem::replace(slot, stack))
        }
    }
}