use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::item::stack::ItemStack;
use std::ops::Range;

/// 库存插入结果
#[derive(Debug, Clone)]
pub enum InventoryInsertResult {
    /// 全部插入成功，无剩余
    AllInserted,
    /// 部分插入，返回未能放入的剩余堆叠
    Partial(ItemStack),
    /// 库存已满，完全未能插入，返回原堆叠
    Full(ItemStack),
}

/// 尝试将物品堆叠插入容器
pub fn insert_into_container<C: InventoryContainer + ?Sized>(
    container: &mut C,
    stack: ItemStack,
) -> InventoryInsertResult {
    let slot_count = container.slot_count();
    insert_into_range(container, stack, 0..slot_count)
}

/// 仅向容器的指定槽位范围插入物品。
pub fn insert_into_range<C: InventoryContainer + ?Sized>(
    container: &mut C,
    mut stack: ItemStack,
    range: Range<usize>,
) -> InventoryInsertResult {
    if stack.is_empty() {
        return InventoryInsertResult::AllInserted;
    }

    // 尝试合并到已有同种堆叠
    for i in range.clone() {
        if stack.is_empty() {
            return InventoryInsertResult::AllInserted;
        }
        if let Some(slot_stack) = container.get_stack_mut(i)
            && slot_stack.can_merge(&stack)
        {
            stack.merge_from(slot_stack);
        }
    }

    if stack.is_empty() {
        return InventoryInsertResult::AllInserted;
    }

    // 放入第一个空槽位
    for i in range {
        let is_empty = container.get_stack(i).is_none_or(|s| s.is_empty());
        if is_empty {
            container.set_stack(i, stack);
            return InventoryInsertResult::AllInserted;
        }
    }

    // 容器已满
    InventoryInsertResult::Full(stack)
}

/// 尝试将物品插入玩家背包
pub fn insert_into_player(
    hotbar: &mut dyn InventoryContainer,
    backpack: &mut dyn InventoryContainer,
    stack: ItemStack,
) -> InventoryInsertResult {
    match insert_into_container(hotbar, stack) {
        result @ InventoryInsertResult::AllInserted => result,
        InventoryInsertResult::Partial(remaining) => insert_into_container(backpack, remaining),
        full @ InventoryInsertResult::Full(_) => {
            // 快捷栏已满，尝试背包
            let InventoryInsertResult::Full(stack) = full else {
                unreachable!()
            };
            insert_into_container(backpack, stack)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::inventory::container::InventoryContainer;
    use crate::game::inventory::container::survival::SurvivalInventory;
    use crate::shared::item_id::ItemId;

    #[test]
    fn range_insert_never_uses_equipment_slots() {
        let mut inventory = SurvivalInventory::default();
        for index in 0..SurvivalInventory::BACKPACK_SIZE {
            inventory.set_stack(
                index,
                ItemStack::new(
                    ItemId::item(format!("century_journey:full_{index}")),
                    ItemStack::MAX_STACK_SIZE,
                ),
            );
        }

        let incoming = ItemStack::single(ItemId::item("century_journey:overflow"));
        let result = insert_into_range(
            &mut inventory,
            incoming,
            0..SurvivalInventory::BACKPACK_SIZE,
        );

        assert!(matches!(result, InventoryInsertResult::Full(_)));
        assert!(
            inventory
                .get_stack(SurvivalInventory::equipment_index(0))
                .is_none()
        );
    }
}
