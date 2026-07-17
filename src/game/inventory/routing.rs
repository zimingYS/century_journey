use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::cursor::{CursorData, CursorSource};
use crate::game::inventory::interaction::{
    left_click_slot, right_click_slot, shift_click, shift_click_into_range,
};
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::state::InventoryState;
use crate::shared::item_id::ItemId;
use crate::shared::ui_types::SlotKind;

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
                    let half = ItemStack::MAX_STACK_SIZE.div_ceil(2);
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
                    let half = ItemStack::MAX_STACK_SIZE.div_ceil(2);
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
                use crate::game::inventory::container::survival::SurvivalInventory;
                shift_click_into_range(
                    &mut state.hotbar,
                    &mut state.survival,
                    index,
                    0..SurvivalInventory::BACKPACK_SIZE,
                );
            }
            _ => {}
        },

        SlotKind::SurvivalBackpack | SlotKind::SurvivalEquipment | SlotKind::SurvivalAccessory => {
            match action {
                _ if survival_index(kind, index).is_none() => {}
                SlotAction::LeftClick => {
                    let index = survival_index(kind, index).expect("checked above");
                    left_click_slot(&mut state.survival, index, &mut state.cursor);
                    update_cursor_source(&mut state.cursor, CursorSource::SurvivalBackpack(index));
                }
                SlotAction::RightClick => {
                    let index = survival_index(kind, index).expect("checked above");
                    right_click_slot(&mut state.survival, index, &mut state.cursor);
                    update_cursor_source(&mut state.cursor, CursorSource::SurvivalBackpack(index));
                }
                SlotAction::ShiftClick => {
                    let index = survival_index(kind, index).expect("checked above");
                    shift_click(&mut state.survival, &mut state.hotbar, index);
                }
                _ => {}
            }
        }

        SlotKind::Container(_) => {
            // 容器界面尚未接入世界实体；收到该类槽位事件时保持状态不变。
        }
    }
}

/// 把各生存 UI 分区的局部索引转换成 SurvivalInventory 的统一索引。
pub fn survival_index(kind: SlotKind, index: usize) -> Option<usize> {
    use crate::game::inventory::container::survival::SurvivalInventory;

    match kind {
        SlotKind::SurvivalBackpack if index < SurvivalInventory::BACKPACK_SIZE => Some(index),
        SlotKind::SurvivalEquipment if index < SurvivalInventory::EQUIPMENT_SIZE => {
            Some(SurvivalInventory::equipment_index(index))
        }
        SlotKind::SurvivalAccessory => Some(SurvivalInventory::accessory_index(index)),
        _ => None,
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
        if let Some(hotbar_stack) = state.hotbar.get_stack_mut(i)
            && hotbar_stack.is_same_item(&remaining)
        {
            remaining.merge_from(hotbar_stack);
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
