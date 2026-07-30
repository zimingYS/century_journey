//! 提供与具体界面无关的库存插入算法。

use std::ops::Range;

use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::item::stack::ItemStack;

/// 库存插入结果。
#[derive(Debug, Clone)]
pub enum InventoryInsertResult {
    /// 全部插入成功，无剩余。
    AllInserted,
    /// 部分插入，返回未能放入的剩余堆叠。
    Partial(ItemStack),
    /// 库存已满，完全未能插入，返回原堆叠。
    Full(ItemStack),
}

/// 尝试将物品堆叠插入容器。
pub fn insert_into_container<C: InventoryContainer + ?Sized>(
    container: &mut C,
    stack: ItemStack,
) -> InventoryInsertResult {
    let slot_count = container.slot_count();
    insert_into_range(container, stack, 0..slot_count)
}

/// 仅向容器的指定槽位范围插入物品。
///
/// 先填充范围内已有的同种堆叠，再使用空槽位。返回值中的剩余堆叠始终由调用方继续处理。
pub fn insert_into_range<C: InventoryContainer + ?Sized>(
    container: &mut C,
    mut stack: ItemStack,
    range: Range<usize>,
) -> InventoryInsertResult {
    if stack.is_empty() {
        return InventoryInsertResult::AllInserted;
    }

    let original_count = stack.count;

    // 先把输入堆叠合并到已有槽位，避免反向搬空容器中的物品。
    for index in range.clone() {
        if stack.is_empty() {
            return InventoryInsertResult::AllInserted;
        }
        if let Some(slot_stack) = container.get_stack_mut(index)
            && slot_stack.is_same_item(&stack)
            && !slot_stack.is_full()
        {
            slot_stack.merge_from(&mut stack);
        }
    }

    // 每个合法物品堆叠不超过上限，因此剩余部分可以整体放入一个空槽位。
    for index in range {
        if container.get_stack(index).is_none_or(ItemStack::is_empty) {
            container.set_stack(index, stack);
            return InventoryInsertResult::AllInserted;
        }
    }

    if stack.count < original_count {
        InventoryInsertResult::Partial(stack)
    } else {
        InventoryInsertResult::Full(stack)
    }
}

/// 尝试将物品依次插入玩家快捷栏和主背包。
pub fn insert_into_player(
    hotbar: &mut dyn InventoryContainer,
    backpack: &mut dyn InventoryContainer,
    stack: ItemStack,
) -> InventoryInsertResult {
    match insert_into_container(hotbar, stack) {
        InventoryInsertResult::AllInserted => InventoryInsertResult::AllInserted,
        InventoryInsertResult::Partial(remaining) => {
            match insert_into_container(backpack, remaining) {
                InventoryInsertResult::AllInserted => InventoryInsertResult::AllInserted,
                InventoryInsertResult::Partial(remaining)
                | InventoryInsertResult::Full(remaining) => {
                    InventoryInsertResult::Partial(remaining)
                }
            }
        }
        InventoryInsertResult::Full(remaining) => insert_into_container(backpack, remaining),
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/inventory/interaction/transfer.rs"]
mod tests;
