use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::cursor::{CursorData, CursorSource};
use crate::game::inventory::item::id::ItemId;
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
        // 情况 1：光标空 + 槽有物 → 拿起全部
        (false, true) => {
            if let Some(stack) = container.replace_stack(index, ItemStack::empty()) {
                cursor.set_stack(stack);
            }
        }

        // 情况 2：光标有物 + 槽空 → 放下全部
        (true, false) => {
            if let Some(stack) = cursor.take_stack() {
                container.set_stack(index, stack);
            }
        }

        // 情况 3 & 4：光标有物 + 槽有物
        (true, true) => {
            let slot_item = container.get_stack(index).unwrap().item.clone();
            let cursor_item = cursor.stack().unwrap().item.clone();
            let is_same = cursor_item == slot_item;

            if is_same {
                // 情况 3：尝试将光标物品合并到槽位（Minecraft 标准行为）
                let slot_stack = container.get_stack_mut(index).unwrap();
                slot_stack.merge_from(cursor.stack_mut().unwrap());

                // 合并后如果光标为空，清空
                if cursor.stack().is_some_and(|s| s.is_empty()) {
                    cursor.clear();
                }
            } else {
                // 情况 4：交换
                if let Some(old) = container.replace_stack(index, cursor.take_stack().unwrap()) {
                    cursor.set_stack(old);
                }
            }
        }

        // 光标空 + 槽空 → 无操作
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
    let cursor_has = cursor.has_item();

    if !cursor_has {
        // 情况 1：光标空 + 槽有物 → 拿走一半
        if let Some(slot_stack) = container.get_stack_mut(index) {
            if !slot_stack.is_empty() {
                let take_count = (slot_stack.count + 1) / 2; // 奇数向上取整
                if let Some(taken) = slot_stack.take(take_count) {
                    cursor.set_stack(taken);
                }
                // 如果槽位被取空，保持不变（count=0 的空 ItemStack）
            }
        }
    } else {
        // 光标有物
        let slot_has = container.get_stack(index).is_some_and(|s| !s.is_empty());

        if !slot_has {
            // 情况 2：光标有物 + 槽空 → 放入 1 个
            let one = cursor.stack_mut().and_then(|cs| cs.take(1));
            if let Some(stack) = one {
                container.set_stack(index, stack);
            }
            if cursor.stack().is_some_and(|s| s.is_empty()) {
                cursor.clear();
            }
        } else {
            // 情况 3 & 4：光标有物 + 槽有物
            let cursor_item = cursor.stack().map(|s| s.item.clone());
            let slot_item = container.get_stack(index).map(|s| s.item.clone());

            if cursor_item == slot_item {
                // 情况 3：同种 → 放入 1 个
                if let Some(slot_stack) = container.get_stack_mut(index) {
                    if slot_stack.count < ItemStack::MAX_STACK_SIZE {
                        slot_stack.count += 1;
                        if let Some(cs) = cursor.stack_mut() {
                            cs.count -= 1;
                            if cs.count == 0 {
                                cursor.clear();
                            }
                        }
                    }
                }
            }
            // 情况 4：不同物品 → 无操作
        }
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
    let source_stack = match source.get_stack(index) {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return false,
    };

    let mut remaining = source_stack;
    let original_count = remaining.count;

    // 第一遍：尝试合并到已有的同种堆叠
    for i in 0..dest.slot_count() {
        if remaining.is_empty() {
            break;
        }
        if let Some(dest_stack) = dest.get_stack_mut(i) {
            if dest_stack.is_same_item(&remaining) && dest_stack.count < ItemStack::MAX_STACK_SIZE {
                remaining.merge_from(dest_stack);
            }
        }
    }

    // 第二遍：放入第一个空槽位
    if !remaining.is_empty() {
        for i in 0..dest.slot_count() {
            let is_empty = dest.get_stack(i).is_none_or(|s| s.is_empty());
            if is_empty {
                dest.set_stack(i, remaining.clone());
                remaining = ItemStack::empty();
                break;
            }
        }
    }

    let moved = original_count - remaining.count;

    if moved > 0 {
        // 更新源槽位
        if remaining.is_empty() {
            source.replace_stack(index, ItemStack::empty());
        } else {
            source.replace_stack(index, remaining);
        }
        true
    } else {
        false
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 路由：根据 SlotKind 分发到正确的容器
// ═══════════════════════════════════════════════════════════════════════════════

use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::state::InventoryState;
use crate::client::ui::widgets::slot::SlotKind;

/// 统一的槽位交互路由
///
/// UI 层只需调用此函数，无需关心具体容器类型。
/// 未来添加 Chest/Furnace 时只需扩展此 match。
pub fn handle_slot_interaction(state: &mut InventoryState, kind: SlotKind, index: usize, action: SlotAction) {
    match kind {
        // ── 创造模式网格（目录，非真实库存）──
        SlotKind::CreativeGrid => {
            let item = state.creative.visible_items
                .get(index)
                .cloned()
                .unwrap_or(ItemId::air());

            if item.is_air() {
                return;
            }

            match action {
                SlotAction::LeftClick => {
                    state.cursor.set_stack(ItemStack::new(item.clone(), ItemStack::MAX_STACK_SIZE));
                    state.cursor.source = None; // 创造网格无"来源"
                    state.add_recent(item);
                }
                SlotAction::RightClick => {
                    // 右键：从创造网格拿起一半
                    let half = (ItemStack::MAX_STACK_SIZE + 1) / 2;
                    state.cursor.set_stack(ItemStack::new(item.clone(), half));
                    state.cursor.source = None;
                    state.add_recent(item);
                }
                SlotAction::ShiftClick => {
                    // 创造模式 Shift：尝试转移到快捷栏
                    shift_into_hotbar(state, &ItemStack::new(item, ItemStack::MAX_STACK_SIZE));
                }
                _ => {}
            }
        }

        // ── 最近使用面板（目录，非真实库存）──
        SlotKind::Recent => {
            let stack = state.recent.items
                .get(index)
                .cloned()
                .unwrap_or(ItemStack::empty());

            if stack.is_empty() {
                return;
            }

            match action {
                SlotAction::LeftClick => {
                    state.cursor.set_stack(ItemStack::new(stack.item.clone(), ItemStack::MAX_STACK_SIZE));
                    state.cursor.source = None;
                    state.add_recent(stack.item.clone());
                }
                SlotAction::RightClick => {
                    let half = (ItemStack::MAX_STACK_SIZE + 1) / 2;
                    state.cursor.set_stack(ItemStack::new(stack.item.clone(), half));
                    state.cursor.source = None;
                    state.add_recent(stack.item.clone());
                }
                SlotAction::ShiftClick => {
                    shift_into_hotbar(state, &ItemStack::new(stack.item.clone(), ItemStack::MAX_STACK_SIZE));
                }
                _ => {}
            }
        }

        // ── 快捷栏（真实库存）──
        SlotKind::Hotbar => {
            match action {
                SlotAction::LeftClick => {
                    left_click_slot(&mut state.hotbar, index, &mut state.cursor);
                    update_cursor_source(&mut state.cursor, CursorSource::Hotbar(index));
                }
                SlotAction::RightClick => {
                    right_click_slot(&mut state.hotbar, index, &mut state.cursor);
                    update_cursor_source(&mut state.cursor, CursorSource::Hotbar(index));
                }
                SlotAction::ShiftClick => {
                    // 快捷栏 ← → 生存背包
                    // 源 = 快捷栏，目标 = 生存背包
                    shift_click(
                        &mut state.hotbar,
                        &mut state.survival,   // 注意：这里只转移背包部分
                        index,
                    );
                }
                _ => {}
            }
        }

        // ── 生存背包（真实库存）──
        SlotKind::SurvivalBackpack => {
            match action {
                SlotAction::LeftClick => {
                    left_click_slot(&mut state.survival, index, &mut state.cursor);
                    update_cursor_source(&mut state.cursor, CursorSource::SurvivalBackpack(index));
                }
                SlotAction::RightClick => {
                    right_click_slot(&mut state.survival, index, &mut state.cursor);
                    update_cursor_source(&mut state.cursor, CursorSource::SurvivalBackpack(index));
                }
                SlotAction::ShiftClick => {
                    // 源 = 背包，目标 = 快捷栏
                    shift_click(
                        &mut state.survival,
                        &mut state.hotbar,
                        index,
                    );
                }
                _ => {}
            }
        }

        // ── 通用容器（箱子/熔炉/工作台，未来扩展）──
        SlotKind::Container => {
            // TODO: 需要从 WorldStorage 查找容器实体
            // 当前占位，后续实现
        }
    }
}

/// 当光标有物品时更新来源槽位；空光标则清除来源
fn update_cursor_source(cursor: &mut CursorData, source: CursorSource) {
    if cursor.has_item() {
        cursor.source = Some(source);
    } else {
        cursor.source = None;
    }
}

/// 辅助：将物品堆叠尝试转移到快捷栏
fn shift_into_hotbar(state: &mut InventoryState, stack: &ItemStack) {
    let mut remaining = stack.clone();

    // 先尝试合并
    for i in 0..state.hotbar.slot_count() {
        if remaining.is_empty() {
            break;
        }
        if let Some(hotbar_stack) = state.hotbar.get_stack_mut(i) {
            if hotbar_stack.is_same_item(&remaining) {
                remaining.merge_from(hotbar_stack);
            }
        }
    }

    // 再找空位
    if !remaining.is_empty() {
        for i in 0..state.hotbar.slot_count() {
            if state.hotbar.get_stack(i).is_none_or(|s| s.is_empty()) {
                state.hotbar.set_stack(i, remaining);
                return;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 单元测试
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::inventory::container::hotbar::{HotbarData, HOTBAR_SIZE};
    use std::array;

    /// 简易容器 — 测试用
    struct TestContainer {
        slots: [Option<ItemStack>; 9],
    }

    impl TestContainer {
        fn new() -> Self {
            Self { slots: array::from_fn(|_| None) }
        }
    }

    impl InventoryContainer for TestContainer {
        fn slot_count(&self) -> usize { 9 }
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

    fn stone_single() -> ItemStack {
        ItemStack::single(ItemId::block("century_journey:stone"))
    }

    // ─────────────────────────────────────────────────────────────────
    // 左键测试
    // ─────────────────────────────────────────────────────────────────

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
        container.replace_stack(0, dirt());          // 32 dirt
        cursor.set_stack(ItemStack::new(ItemId::block("century_journey:dirt"), 32)); // +32

        left_click_slot(&mut container, 0, &mut cursor);

        // 合并后槽位 64，光标空
        assert!(!cursor.has_item());
        assert_eq!(container.get_stack(0).unwrap().count, 64);
    }

    #[test]
    fn left_click_same_item_overflow_stays_in_cursor() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, dirt());          // 32 dirt
        cursor.set_stack(ItemStack::new(ItemId::block("century_journey:dirt"), 40)); // +40 → 64+8

        left_click_slot(&mut container, 0, &mut cursor);

        assert!(cursor.has_item());
        assert_eq!(cursor.stack().unwrap().count, 8);  // 超出 8 留在光标
        assert_eq!(container.get_stack(0).unwrap().count, 64);
    }

    #[test]
    fn left_click_different_items_swaps() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, stone());  // stone
        cursor.set_stack(dirt());             // dirt

        left_click_slot(&mut container, 0, &mut cursor);

        // 交换后：槽位 = dirt, 光标 = stone
        assert_eq!(cursor.stack().unwrap().item, ItemId::block("century_journey:stone"));
        assert_eq!(cursor.stack().unwrap().count, 64);
        assert_eq!(container.get_stack(0).unwrap().item, ItemId::block("century_journey:dirt"));
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

    // ─────────────────────────────────────────────────────────────────
    // 右键测试
    // ─────────────────────────────────────────────────────────────────

    #[test]
    fn right_click_empty_cursor_takes_half() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, stone());  // 64 stone

        right_click_slot(&mut container, 0, &mut cursor);

        assert_eq!(cursor.stack().unwrap().count, 32);
        assert_eq!(container.get_stack(0).unwrap().count, 32);
    }

    #[test]
    fn right_click_odd_count_rounds_up() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, ItemStack::new(ItemId::block("century_journey:stone"), 63));

        right_click_slot(&mut container, 0, &mut cursor);

        // 63 → (63+1)/2 = 32 被拿走，31 留在槽位
        assert_eq!(cursor.stack().unwrap().count, 32);
        assert_eq!(container.get_stack(0).unwrap().count, 31);
    }

    #[test]
    fn right_click_full_cursor_empty_slot_puts_one() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        cursor.set_stack(stone());  // 64 stone in cursor

        right_click_slot(&mut container, 0, &mut cursor);

        assert_eq!(container.get_stack(0).unwrap().count, 1);
        assert_eq!(cursor.stack().unwrap().count, 63);
    }

    #[test]
    fn right_click_same_item_adds_one() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, dirt());  // 32 dirt in slot
        cursor.set_stack(dirt());            // 32 dirt in cursor

        right_click_slot(&mut container, 0, &mut cursor);

        assert_eq!(container.get_stack(0).unwrap().count, 33);
        assert_eq!(cursor.stack().unwrap().count, 31);
    }

    #[test]
    fn right_click_full_slot_no_add() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();
        container.replace_stack(0, stone());  // 64 stone (full)
        cursor.set_stack(stone());            // 64 stone

        right_click_slot(&mut container, 0, &mut cursor);

        // 槽位已满，不放入
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

        // 不同物品，无操作
        assert_eq!(container.get_stack(0).unwrap().item, ItemId::block("century_journey:stone"));
        assert_eq!(cursor.stack().unwrap().item, ItemId::block("century_journey:dirt"));
    }

    #[test]
    fn right_click_empty_cursor_empty_slot_noop() {
        let mut container = TestContainer::new();
        let mut cursor = CursorData::default();

        right_click_slot(&mut container, 0, &mut cursor);

        assert!(!cursor.has_item());
        assert!(container.get_stack(0).is_none());
    }

    // ─────────────────────────────────────────────────────────────────
    // Shift 转移测试
    // ─────────────────────────────────────────────────────────────────

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
        source.replace_stack(0, dirt());  // 32
        dest.replace_stack(0, dirt());    // 32

        shift_click(&mut source, &mut dest, 0);

        // 合并后 dest[0] = 64
        assert!(source.get_stack(0).is_none());
        assert_eq!(dest.get_stack(0).unwrap().count, 64);
    }

    #[test]
    fn shift_click_overflow_goes_to_next_empty() {
        let mut source = TestContainer::new();
        let mut dest = TestContainer::new();
        source.replace_stack(0, stone());                       // 64
        dest.replace_stack(0, ItemStack::new(ItemId::block("century_journey:stone"), 60));  // 60

        shift_click(&mut source, &mut dest, 0);

        // dest[0] = 64 (满), dest[1] = 4 (剩余)
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

        // 填满 dest
        for i in 0..9 {
            dest.replace_stack(i, stone());
        }

        let moved = shift_click(&mut source, &mut dest, 0);

        // 不同物品无法合并，没有空位，转移失败
        assert!(!moved);
        assert!(source.get_stack(0).is_some());
    }
}