use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::cursor::CursorData;
use crate::game::inventory::item::stack::ItemStack;
use std::ops::Range;

// ═══════════════════════════════════════════════════════════════════════════════
// 核心交互函数 — 纯数据操作，无 UI 依赖
// ═══════════════════════════════════════════════════════════════════════════════

/// 左键点击槽位
///
/// 实现 Minecraft 标准行为：
/// - 光标空 + 槽有物 → 拿起全部
/// - 光标有物 + 槽空 → 放下全部
/// - 光标有物 + 槽有同种 → 合并（超出留在光标）
/// - 光标有物 + 槽有不同 → 交换
pub fn left_click_slot<C: InventoryContainer>(
    container: &mut C,
    index: usize,
    cursor: &mut CursorData,
) {
    let slot_has = container.get_stack(index).is_some_and(|s| !s.is_empty());
    let cursor_has = cursor.has_item();

    match (cursor_has, slot_has) {
        (false, true) => {
            if let Some(stack) = container.replace_stack(index, ItemStack::empty()) {
                cursor.set_stack(stack);
            }
        }
        (true, false) => {
            if let Some(stack) = cursor.take_stack() {
                container.set_stack(index, stack);
            }
        }
        (true, true) => {
            let Some(slot_stack) = container.get_stack(index) else {
                return;
            };
            let Some(cursor_stack) = cursor.stack() else {
                return;
            };
            let is_same = cursor_stack.item == slot_stack.item;

            if is_same {
                if let Some(slot_stack) = container.get_stack_mut(index)
                    && let Some(cursor_stack) = cursor.stack_mut()
                {
                    slot_stack.merge_from(cursor_stack);
                }

                // 如果光标空，清除光标
                if cursor.stack().is_none_or(|s| s.is_empty()) {
                    cursor.clear();
                }
            } else {
                if let Some(slot_stack) = container.replace_stack(index, ItemStack::empty()) {
                    let cursor_stack = cursor.take_stack().unwrap_or_default();
                    cursor.set_stack(slot_stack);
                    container.set_stack(index, cursor_stack);
                }
            }
        }
        (false, false) => {}
    }
}

/// 右键点击槽位
///
/// 实现 Minecraft 标准行为：
/// - 光标空 + 槽有物 → 拿走一半（奇数向上取整）
/// - 光标有物 + 槽空 → 放入 1 个
/// - 光标有物 + 槽有同种且未满 → 放入 1 个
/// - 不同物品 → 无操作
pub fn right_click_slot<C: InventoryContainer>(
    container: &mut C,
    index: usize,
    cursor: &mut CursorData,
) {
    let slot_has = container.get_stack(index).is_some_and(|s| !s.is_empty());
    let cursor_has = cursor.has_item();

    match (cursor_has, slot_has) {
        (false, true) => {
            let Some(stack) = container.get_stack(index) else {
                return;
            };
            let total = stack.count;
            let half = total.div_ceil(2);
            let remaining = total - half;

            if remaining == 0 {
                if let Some(stack) = container.replace_stack(index, ItemStack::empty()) {
                    cursor.set_stack(stack);
                }
            } else {
                if let Some(slot_stack) = container.get_stack_mut(index) {
                    slot_stack.count = remaining;
                }

                let Some(stack) = container.get_stack(index) else {
                    return;
                };
                let cursor_stack = ItemStack::new(stack.item.clone(), half);
                cursor.set_stack(cursor_stack);
            }
        }
        (true, false) => {
            let Some(cursor_stack) = cursor.stack() else {
                return;
            };
            let cursor_count = cursor_stack.count;
            let take = 1.min(cursor_count);

            let mut new_cursor = cursor_stack.clone();
            new_cursor.count = cursor_count - take;

            let mut new_slot = cursor_stack.clone();
            new_slot.count = take;

            if new_cursor.count == 0 {
                cursor.take_stack();
            } else {
                cursor.set_stack(new_cursor);
            }
            container.set_stack(index, new_slot);
        }
        (true, true) => {
            let Some(slot) = container.get_stack(index) else {
                return;
            };
            let Some(cursor_item) = cursor.stack() else {
                return;
            };
            let is_same = cursor_item.item == slot.item;

            if is_same && slot.count < ItemStack::MAX_STACK_SIZE {
                if let Some(slot_stack) = container.get_stack_mut(index) {
                    slot_stack.count += 1;
                }
                if let Some(cursor_stack) = cursor.stack_mut() {
                    cursor_stack.count -= 1;
                    if cursor_stack.count == 0 {
                        cursor.take_stack();
                    }
                }
            }
        }
        (false, false) => {}
    }
}

/// Shift + 点击槽位（快速转移）
///
/// 在 source 和 dest 容器间转移物品：
/// 1. 优先合并到 dest 中已有的同种堆叠
/// 2. 再寻找 dest 中第一个空槽位放入
pub fn shift_click<C1: InventoryContainer, C2: InventoryContainer>(
    source: &mut C1,
    dest: &mut C2,
    index: usize,
) -> bool {
    let slot_count = dest.slot_count();
    shift_click_into_range(source, dest, index, 0..slot_count)
}

/// Shift 点击并把物品限制在目标容器的指定槽位范围内。
pub fn shift_click_into_range<C1: InventoryContainer, C2: InventoryContainer>(
    source: &mut C1,
    dest: &mut C2,
    index: usize,
    range: Range<usize>,
) -> bool {
    let Some(source_stack) = source.get_stack(index) else {
        return false;
    };
    if source_stack.is_empty() {
        return false;
    }

    let mut remaining = source_stack.clone();

    // 第一步：优先合并到已有同类堆叠。
    for i in range.clone() {
        if remaining.is_empty() {
            break;
        }
        if let Some(dest_stack) = dest.get_stack_mut(i)
            && dest_stack.is_same_item(&remaining)
        {
            dest_stack.merge_from(&mut remaining);
        }
    }

    // 第二步：将剩余物品放入第一个空槽位。
    if !remaining.is_empty() {
        for i in range {
            if dest.get_stack(i).is_none_or(|s| s.is_empty()) {
                dest.set_stack(i, remaining.clone());
                remaining = ItemStack::empty();
                break;
            }
        }
    }

    if remaining.is_empty() {
        source.replace_stack(index, ItemStack::empty());
        true
    } else {
        let moved_count = source_stack.count - remaining.count;
        if moved_count > 0 {
            let mut updated = source_stack.clone();
            updated.count = remaining.count;
            source.set_stack(index, updated);
            true
        } else {
            false
        }
    }
}

/// 将来源槽位中的一个物品移动到目标范围，供滚轮快速转移使用。
pub fn move_one_into_range<C1: InventoryContainer, C2: InventoryContainer>(
    source: &mut C1,
    dest: &mut C2,
    source_index: usize,
    dest_range: Range<usize>,
) -> bool {
    let Some(source_stack) = source.get_stack(source_index).cloned() else {
        return false;
    };
    if source_stack.is_empty() {
        return false;
    }
    let mut one =
        ItemStack::with_instance(source_stack.item.clone(), 1, source_stack.instance.clone());
    if !insert_one_into_range(dest, &mut one, dest_range) {
        return false;
    }

    let emptied = source
        .get_stack_mut(source_index)
        .and_then(|stack| stack.take(1).map(|_| stack.is_empty()))
        .unwrap_or(false);
    if emptied {
        source.set_stack(source_index, ItemStack::empty());
    }
    true
}

/// 从来源范围寻找与目标槽位相同的物品，并补入一个。
pub fn pull_one_matching<C1: InventoryContainer, C2: InventoryContainer>(
    dest: &mut C1,
    source: &mut C2,
    dest_index: usize,
    source_range: Range<usize>,
) -> bool {
    let Some(target) = dest.get_stack(dest_index).cloned() else {
        return false;
    };
    if target.is_empty() || target.is_full() {
        return false;
    }
    let source_index = source_range.into_iter().find(|index| {
        source
            .get_stack(*index)
            .is_some_and(|stack| stack.is_same_item(&target) && !stack.is_empty())
    });
    source_index.is_some_and(|index| {
        move_one_into_range(
            source,
            dest,
            index,
            dest_index..dest_index.saturating_add(1),
        )
    })
}

fn insert_one_into_range<C: InventoryContainer>(
    dest: &mut C,
    one: &mut ItemStack,
    range: Range<usize>,
) -> bool {
    for index in range.clone() {
        if let Some(existing) = dest.get_stack_mut(index)
            && existing.is_same_item(one)
            && !existing.is_full()
        {
            existing.merge_from(one);
            return one.is_empty();
        }
    }
    for index in range {
        if dest.get_stack(index).is_none_or(ItemStack::is_empty) {
            dest.set_stack(index, std::mem::take(one));
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::inventory::container::survival::SurvivalInventory;
    use crate::game::inventory::item::stack::ItemInstanceData;
    use crate::shared::item_id::ItemId;

    #[test]
    fn wheel_transfer_moves_one_and_preserves_instance_data() {
        let item = ItemId::item("century_journey:test_tool");
        let mut source = SurvivalInventory::default();
        let mut dest = SurvivalInventory::default();
        source.set_stack(
            0,
            ItemStack::with_instance(
                item.clone(),
                2,
                ItemInstanceData {
                    durability: Some(7),
                },
            ),
        );

        assert!(move_one_into_range(&mut source, &mut dest, 0, 0..1));
        assert_eq!(source.get_stack(0).unwrap().count, 1);
        assert_eq!(dest.get_stack(0).unwrap().count, 1);
        assert_eq!(dest.get_stack(0).unwrap().durability(), Some(7));
    }

    #[test]
    fn wheel_pull_only_uses_matching_stacks() {
        let item = ItemId::item("century_journey:planks");
        let mut dest = SurvivalInventory::default();
        let mut source = SurvivalInventory::default();
        dest.set_stack(0, ItemStack::single(item.clone()));
        source.set_stack(0, ItemStack::new(item, 3));

        assert!(pull_one_matching(&mut dest, &mut source, 0, 0..1));
        assert_eq!(dest.get_stack(0).unwrap().count, 2);
        assert_eq!(source.get_stack(0).unwrap().count, 2);
    }
}
