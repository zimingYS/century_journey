use crate::client::ui::widgets::slot::SlotKind;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::cursor::{CursorData, CursorSource};
use crate::game::inventory::interaction::{left_click_slot, right_click_slot, shift_click};
use crate::shared::item_id::ItemId;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::state::InventoryState;

/// 统一的槽位交互路由
///
/// UI 层只需调用此函数，无需关心具体容器类型。
/// 未来添加 Chest/Furnace 时只需扩展此 match。
pub fn handle_slot_interaction(
    state: &mut InventoryState,
    kind: SlotKind,
    index: usize,
    action: SlotAction,
) {
    match kind {
        SlotKind::CreativeGrid => {
            let item = state
                .creative
                .visible_items
                .get(index)
                .cloned()
                .unwrap_or(ItemId::air());

            if item.is_air() {
                return;
            }

            match action {
                SlotAction::LeftClick => {
                    state
                        .cursor
                        .set_stack(ItemStack::new(item.clone(), ItemStack::MAX_STACK_SIZE));
                    state.cursor.source = None;
                    state.add_recent(item);
                }
                SlotAction::RightClick => {
                    let half = (ItemStack::MAX_STACK_SIZE + 1) / 2;
                    state.cursor.set_stack(ItemStack::new(item.clone(), half));
                    state.cursor.source = None;
                    state.add_recent(item);
                }
                SlotAction::ShiftClick => {
                    shift_into_hotbar(state, &ItemStack::new(item, ItemStack::MAX_STACK_SIZE));
                }
                _ => {}
            }
        }

        SlotKind::Recent => {
            let stack = state
                .recent
                .items
                .get(index)
                .cloned()
                .unwrap_or(ItemStack::empty());

            if stack.is_empty() {
                return;
            }

            match action {
                SlotAction::LeftClick => {
                    state.cursor.set_stack(ItemStack::new(
                        stack.item.clone(),
                        ItemStack::MAX_STACK_SIZE,
                    ));
                    state.cursor.source = None;
                    state.add_recent(stack.item.clone());
                }
                SlotAction::RightClick => {
                    let half = (ItemStack::MAX_STACK_SIZE + 1) / 2;
                    state
                        .cursor
                        .set_stack(ItemStack::new(stack.item.clone(), half));
                    state.cursor.source = None;
                    state.add_recent(stack.item.clone());
                }
                SlotAction::ShiftClick => {
                    shift_into_hotbar(
                        state,
                        &ItemStack::new(stack.item.clone(), ItemStack::MAX_STACK_SIZE),
                    );
                }
                _ => {}
            }
        }

        SlotKind::Hotbar => match action {
            SlotAction::LeftClick => {
                left_click_slot(&mut state.hotbar, index, &mut state.cursor);
                update_cursor_source(&mut state.cursor, CursorSource::Hotbar(index));
            }
            SlotAction::RightClick => {
                right_click_slot(&mut state.hotbar, index, &mut state.cursor);
                update_cursor_source(&mut state.cursor, CursorSource::Hotbar(index));
            }
            SlotAction::ShiftClick => {
                shift_click(&mut state.hotbar, &mut state.survival, index);
            }
            _ => {}
        },

        SlotKind::SurvivalBackpack => match action {
            SlotAction::LeftClick => {
                left_click_slot(&mut state.survival, index, &mut state.cursor);
                update_cursor_source(&mut state.cursor, CursorSource::SurvivalBackpack(index));
            }
            SlotAction::RightClick => {
                right_click_slot(&mut state.survival, index, &mut state.cursor);
                update_cursor_source(&mut state.cursor, CursorSource::SurvivalBackpack(index));
            }
            SlotAction::ShiftClick => {
                shift_click(&mut state.survival, &mut state.hotbar, index);
            }
            _ => {}
        },

        SlotKind::Container => {
            // TODO: 需要从 WorldStorage 查找容器实体
        }
    }
}

fn update_cursor_source(cursor: &mut CursorData, source: CursorSource) {
    if cursor.has_item() {
        cursor.source = Some(source);
    } else {
        cursor.source = None;
    }
}

fn shift_into_hotbar(state: &mut InventoryState, stack: &ItemStack) {
    let mut remaining = stack.clone();

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

    if !remaining.is_empty() {
        for i in 0..state.hotbar.slot_count() {
            if state.hotbar.get_stack(i).is_none_or(|s| s.is_empty()) {
                state.hotbar.set_stack(i, remaining);
                return;
            }
        }
    }
}
