use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::cursor::CursorData;
use crate::game::inventory::interaction::{left_click_slot, right_click_slot, shift_click};
use crate::game::inventory::item::stack::ItemStack;
use crate::shared::item_id::ItemId;
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
    fn replace_stack(&mut self, index: usize, stack: ItemStack) -> Option<ItemStack> {
        let slot = self.get_stack_mut(index)?;
        let old = std::mem::replace(slot, stack);
        if slot.is_empty() {
            self.slots[index] = None;
        }
        if old.is_empty() { None } else { Some(old) }
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
    container.set_stack(0, stone());
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
    container.set_stack(0, dirt());
    cursor.set_stack(ItemStack::new(ItemId::block("century_journey:dirt"), 32));
    left_click_slot(&mut container, 0, &mut cursor);
    assert!(!cursor.has_item());
    assert_eq!(container.get_stack(0).unwrap().count, 64);
}

#[test]
fn left_click_same_item_overflow_stays_in_cursor() {
    let mut container = TestContainer::new();
    let mut cursor = CursorData::default();
    container.set_stack(0, dirt());
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
    container.set_stack(0, stone());
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
    container.set_stack(0, stone());
    right_click_slot(&mut container, 0, &mut cursor);
    assert_eq!(cursor.stack().unwrap().count, 32);
    assert_eq!(container.get_stack(0).unwrap().count, 32);
}

#[test]
fn right_click_odd_count_rounds_up() {
    let mut container = TestContainer::new();
    let mut cursor = CursorData::default();
    container.set_stack(
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
    container.set_stack(0, dirt());
    cursor.set_stack(dirt());
    right_click_slot(&mut container, 0, &mut cursor);
    assert_eq!(container.get_stack(0).unwrap().count, 33);
    assert_eq!(cursor.stack().unwrap().count, 31);
}

#[test]
fn right_click_full_slot_no_add() {
    let mut container = TestContainer::new();
    let mut cursor = CursorData::default();
    container.set_stack(0, stone());
    cursor.set_stack(stone());
    right_click_slot(&mut container, 0, &mut cursor);
    assert_eq!(container.get_stack(0).unwrap().count, 64);
    assert_eq!(cursor.stack().unwrap().count, 64);
}

#[test]
fn right_click_different_items_noop() {
    let mut container = TestContainer::new();
    let mut cursor = CursorData::default();
    container.set_stack(0, stone());
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
    source.set_stack(0, stone());
    let moved = shift_click(&mut source, &mut dest, 0);
    assert!(moved);
    assert!(source.get_stack(0).is_none());
    assert_eq!(dest.get_stack(0).unwrap().count, 64);
}

#[test]
fn shift_click_merges_with_existing() {
    let mut source = TestContainer::new();
    let mut dest = TestContainer::new();
    source.set_stack(0, dirt());
    dest.set_stack(0, dirt());
    shift_click(&mut source, &mut dest, 0);
    assert!(source.get_stack(0).is_none());
    assert_eq!(dest.get_stack(0).unwrap().count, 64);
}

#[test]
fn shift_click_overflow_goes_to_next_empty() {
    let mut source = TestContainer::new();
    let mut dest = TestContainer::new();
    source.set_stack(0, stone());
    dest.set_stack(
        0,
        ItemStack::new(ItemId::block("century_journey:stone"), 60),
    );
    shift_click(&mut source, &mut dest, 0);
    assert!(source.get_stack(0).is_none());
    assert_eq!(dest.get_stack(0).unwrap().count, 64);
    assert_eq!(dest.get_stack(1).unwrap().count, 60);
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
    source.set_stack(0, dirt());
    for i in 0..9 {
        dest.set_stack(i, stone());
    }
    let moved = shift_click(&mut source, &mut dest, 0);
    assert!(!moved);
    assert!(source.get_stack(0).is_some());
}
