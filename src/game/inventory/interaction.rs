use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::cursor::CursorData;
use crate::game::inventory::item::stack::ItemStack;

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
            let slot_item = container.get_stack(index).unwrap().item.clone();
            let cursor_item = cursor.stack().unwrap().item.clone();
            let is_same = cursor_item == slot_item;

            if is_same {
                let slot_stack = container.get_stack_mut(index).unwrap();
                slot_stack.merge_from(cursor.stack_mut().unwrap());

                // 如果槽位满或光标空，清除光标；否则保留剩余
                if cursor.stack().map_or(true, |s| s.is_empty()) {
                    cursor.clear();
                }
            } else {
                let slot_stack = container.replace_stack(index, ItemStack::empty()).unwrap();
                let cursor_stack = cursor.take_stack().unwrap();
                cursor.set_stack(slot_stack);
                container.set_stack(index, cursor_stack);
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
            let total = container.get_stack(index).unwrap().count;
            let half = (total + 1) / 2;
            let remaining = total - half;

            if remaining == 0 {
                if let Some(stack) = container.replace_stack(index, ItemStack::empty()) {
                    cursor.set_stack(stack);
                }
            } else {
                container.get_stack_mut(index).unwrap().count = remaining;

                let cursor_stack =
                    ItemStack::new(container.get_stack(index).unwrap().item.clone(), half);
                cursor.set_stack(cursor_stack);
            }
        }
        (true, false) => {
            let cursor_count = cursor.stack().unwrap().count;
            let take = 1.min(cursor_count);

            let mut new_cursor = cursor.stack().unwrap().clone();
            new_cursor.count = cursor_count - take;

            let mut new_slot = cursor.stack().unwrap().clone();
            new_slot.count = take;

            if new_cursor.count == 0 {
                cursor.take_stack();
            } else {
                cursor.set_stack(new_cursor);
            }
            container.set_stack(index, new_slot);
        }
        (true, true) => {
            let slot_item = container.get_stack(index).unwrap().item.clone();
            let cursor_item = cursor.stack().unwrap().item.clone();
            let is_same = cursor_item == slot_item;

            if is_same {
                let slot = container.get_stack(index).unwrap();
                if slot.count < ItemStack::MAX_STACK_SIZE {
                    let slot_stack = container.get_stack_mut(index).unwrap();
                    slot_stack.count += 1;

                    let cursor_stack = cursor.stack_mut().unwrap();
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
    let Some(source_stack) = source.get_stack(index) else {
        return false;
    };
    if source_stack.is_empty() {
        return false;
    }

    let mut remaining = source_stack.clone();

    // Step 1: 合并到已有堆叠
    for i in 0..dest.slot_count() {
        if remaining.is_empty() {
            break;
        }
        if let Some(dest_stack) = dest.get_stack_mut(i) {
            if dest_stack.is_same_item(&remaining) {
                dest_stack.merge_from(&mut remaining);
            }
        }
    }

    // Step 2: 找空位
    if !remaining.is_empty() {
        for i in 0..dest.slot_count() {
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
