use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::cursor::CursorData;
use crate::shared::item_id::ItemId;
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

                let mut cursor_stack =
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

// ═══════════════════════════════════════════════════════════════════════════════
// 单元测试
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::array;

    struct TestContainer {
        slots: [Option<ItemStack>; 9],
    }

    impl TestContainer {
        fn new() -> Self {
            Self {
                slots: array::from_fn(|_| None),
            }
        }
    }

    impl InventoryContainer for TestContainer {
        fn slot_count(&self) -> usize {
            9
        }
        fn get_stack(&self, index: usize) -> Option<&ItemStack> {
            self.slots.get(index).and_then(|s| s.as_ref())
        }
        fn get_stack_mut(&mut self, index: usize) -> Option<&mut ItemStack> {
            self.slots.get_mut(index).and_then(|s| s.as_mut())
        }
        fn set_stack(&mut self, index: usize, stack: ItemStack) {
            if index < 9 {
                if stack.is_empty() {
                    self.slots[index] = None;
                } else {
                    self.slots[index] = Some(stack);
                }
            }
        }
    }

    fn stone() -> ItemStack {
        ItemStack::new(ItemId::block("century_journey:stone"), 64)
    }

    fn dirt() -> ItemStack {
        ItemStack::new(ItemId::block("century_journey:dirt"), 32)
    }

    #[test]
    fn left_click_empty_cursor_full_slot_picks_up_all() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, stone());
        left_click_slot(&mut container, 0, &mut cursor);
        assert!(cursor.has_item());
        assert_eq!(cursor.stack().unwrap().count, 64);
        assert!(container.get_stack(0).is_none());
    }

    #[test]
    fn left_click_full_cursor_empty_slot_puts_down_all() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        cursor.set_stack(stone());
        left_click_slot(&mut container, 0, &mut cursor);
        assert!(!cursor.has_item());
        let slot = container.get_stack(0).unwrap();
        assert_eq!(slot.item, ItemId::block("century_journey:stone"));
        assert_eq!(slot.count, 64);
    }

    #[test]
    fn left_click_same_item_merges() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, dirt());
        cursor.set_stack(ItemStack::new(ItemId::block("century_journey:dirt"), 32));
        left_click_slot(&mut container, 0, &mut cursor);
        assert!(!cursor.has_item());
        assert_eq!(container.get_stack(0).unwrap().count, 64);
    }

    #[test]
    fn left_click_same_item_overflow_stays_in_cursor() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, dirt());
        cursor.set_stack(ItemStack::new(ItemId::block("century_journey:dirt"), 40));
        left_click_slot(&mut container, 0, &mut cursor);
        assert!(cursor.has_item());
        assert_eq!(cursor.stack().unwrap().count, 8);
        assert_eq!(container.get_stack(0).unwrap().count, 64);
    }

    #[test]
    fn left_click_different_items_swaps() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, stone());
        cursor.set_stack(dirt());
        left_click_slot(&mut container, 0, &mut cursor);
        assert_eq!(
            cursor.stack().unwrap().item,
            ItemId::block("century_journey:stone")
        );
        assert_eq!(cursor.stack().unwrap().count, 64);
        assert_eq!(
            container.get_stack(0).unwrap().item,
            ItemId::block("century_journey:dirt")
        );
        assert_eq!(container.get_stack(0).unwrap().count, 32);
    }

    #[test]
    fn left_click_both_empty_noop() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        left_click_slot(&mut container, 0, &mut cursor);
        assert!(!cursor.has_item());
        assert!(container.get_stack(0).is_none());
    }

    #[test]
    fn right_click_empty_cursor_takes_half() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, stone());
        right_click_slot(&mut container, 0, &mut cursor);
        assert_eq!(cursor.stack().unwrap().count, 32);
        assert_eq!(container.get_stack(0).unwrap().count, 32);
    }

    #[test]
    fn right_click_odd_count_rounds_up() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(
            0,
            ItemStack::new(ItemId::block("century_journey:stone"), 63),
        );
        right_click_slot(&mut container, 0, &mut cursor);
        assert_eq!(cursor.stack().unwrap().count, 32);
        assert_eq!(container.get_stack(0).unwrap().count, 31);
    }

    #[test]
    fn right_click_full_cursor_empty_slot_puts_one() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        cursor.set_stack(stone());
        right_click_slot(&mut container, 0, &mut cursor);
        assert_eq!(container.get_stack(0).unwrap().count, 1);
        assert_eq!(cursor.stack().unwrap().count, 63);
    }

    #[test]
    fn right_click_same_item_adds_one() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, dirt());
        cursor.set_stack(dirt());
        right_click_slot(&mut container, 0, &mut cursor);
        assert_eq!(container.get_stack(0).unwrap().count, 33);
        assert_eq!(cursor.stack().unwrap().count, 31);
    }

    #[test]
    fn right_click_full_slot_no_add() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, stone());
        cursor.set_stack(stone());
        right_click_slot(&mut container, 0, &mut cursor);
        assert_eq!(container.get_stack(0).unwrap().count, 64);
        assert_eq!(cursor.stack().unwrap().count, 64);
    }

    #[test]
    fn right_click_different_items_noop() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, stone());
        cursor.set_stack(dirt());
        right_click_slot(&mut container, 0, &mut cursor);
        assert_eq!(
            container.get_stack(0).unwrap().item,
            ItemId::block("century_journey:stone")
        );
        assert_eq!(
            cursor.stack().unwrap().item,
            ItemId::block("century_journey:dirt")
        );
    }

    #[test]
    fn right_click_empty_cursor_empty_slot_noop() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        right_click_slot(&mut container, 0, &mut cursor);
        assert!(!cursor.has_item());
        assert!(container.get_stack(0).is_none());
    }

    #[test]
    fn shift_click_moves_to_empty_slot() {
        let mut source = TestContainer::new();
        let mut dest = TestContainer::new();
        source.replace_stack(0, stone());
        let moved = shift_click(&mut source, &mut dest, 0);
        assert!(moved);
        assert!(source.get_stack(0).is_none());
        assert_eq!(dest.get_stack(0).unwrap().count, 64);
    }

    #[test]
    fn shift_click_merges_with_existing() {
        let mut source = TestContainer::new();
        let mut dest = TestContainer::new();
        source.replace_stack(0, dirt());
        dest.replace_stack(0, dirt());
        shift_click(&mut source, &mut dest, 0);
        assert!(source.get_stack(0).is_none());
        assert_eq!(dest.get_stack(0).unwrap().count, 64);
    }

    #[test]
    fn shift_click_overflow_goes_to_next_empty() {
        let mut source = TestContainer::new();
        let mut dest = TestContainer::new();
        source.replace_stack(0, stone());
        dest.replace_stack(
            0,
            ItemStack::new(ItemId::block("century_journey:stone"), 60),
        );
        shift_click(&mut source, &mut dest, 0);
        assert!(source.get_stack(0).is_none());
        assert_eq!(dest.get_stack(0).unwrap().count, 64);
        assert_eq!(dest.get_stack(1).unwrap().count, 4);
    }

    #[test]
    fn shift_click_empty_source_noop() {
        let mut source = TestContainer::new();
        let mut dest = TestContainer::new();
        let moved = shift_click(&mut source, &mut dest, 0);
        assert!(!moved);
    }

    #[test]
    fn shift_click_full_dest_noop() {
        let mut source = TestContainer::new();
        let mut dest = TestContainer::new();
        source.replace_stack(0, dirt());
        for i in 0..9 {
            dest.replace_stack(i, stone());
        }
        let moved = shift_click(&mut source, &mut dest, 0);
        assert!(!moved);
        assert!(source.get_stack(0).is_some());
    }
}
