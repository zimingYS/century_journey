use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::item::stack::ItemStack;
use std::ops::Range;

/// 搴撳瓨鎻掑叆缁撴灉
#[derive(Debug, Clone)]
pub enum InventoryInsertResult {
    /// 鍏ㄩ儴鎻掑叆鎴愬姛锛屾棤鍓╀綑
    AllInserted,
    /// 閮ㄥ垎鎻掑叆锛岃繑鍥炴湭鑳芥斁鍏ョ殑鍓╀綑鍫嗗彔
    Partial(ItemStack),
    /// 搴撳瓨宸叉弧锛屽畬鍏ㄦ湭鑳芥彃鍏ワ紝杩斿洖鍘熷爢鍙?
    Full(ItemStack),
}

/// 灏濊瘯灏嗙墿鍝佸爢鍙犳彃鍏ュ鍣?
pub fn insert_into_container<C: InventoryContainer + ?Sized>(
    container: &mut C,
    stack: ItemStack,
) -> InventoryInsertResult {
    let slot_count = container.slot_count();
    insert_into_range(container, stack, 0..slot_count)
}

/// 浠呭悜瀹瑰櫒鐨勬寚瀹氭Ы浣嶈寖鍥存彃鍏ョ墿鍝併€?
pub fn insert_into_range<C: InventoryContainer + ?Sized>(
    container: &mut C,
    mut stack: ItemStack,
    range: Range<usize>,
) -> InventoryInsertResult {
    if stack.is_empty() {
        return InventoryInsertResult::AllInserted;
    }

    // 灏濊瘯鍚堝苟鍒板凡鏈夊悓绉嶅爢鍙?
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

    // 鏀惧叆绗竴涓┖妲戒綅
    for i in range {
        let is_empty = container.get_stack(i).is_none_or(|s| s.is_empty());
        if is_empty {
            container.set_stack(i, stack);
            return InventoryInsertResult::AllInserted;
        }
    }

    // 瀹瑰櫒宸叉弧
    InventoryInsertResult::Full(stack)
}

/// 灏濊瘯灏嗙墿鍝佹彃鍏ョ帺瀹惰儗鍖?
pub fn insert_into_player(
    hotbar: &mut dyn InventoryContainer,
    backpack: &mut dyn InventoryContainer,
    stack: ItemStack,
) -> InventoryInsertResult {
    match insert_into_container(hotbar, stack) {
        result @ InventoryInsertResult::AllInserted => result,
        InventoryInsertResult::Partial(remaining) => insert_into_container(backpack, remaining),
        full @ InventoryInsertResult::Full(_) => {
            // 蹇嵎鏍忓凡婊★紝灏濊瘯鑳屽寘
            let InventoryInsertResult::Full(stack) = full else {
                unreachable!()
            };
            insert_into_container(backpack, stack)
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/inventory/transfer.rs"]
mod tests;
