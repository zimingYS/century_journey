//! 将通用槽位意图路由到对应的权威容器。

use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
use crate::game::inventory::container::survival::SurvivalInventory;
use crate::game::inventory::interaction::click::{
    left_click_slot, move_one_into_range, pull_one_matching, right_click_slot, shift_click,
    shift_click_into_range,
};
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::slot::SlotKind;
use crate::game::inventory::state::{CursorData, CursorSource, InventoryState};
use crate::shared::item_id::ItemId;

/// 统一的槽位交互入口。
///
/// 调用方只提供槽位类型、索引和动作；具体容器映射集中在此，避免 UI 复制权威规则。
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
                    state.add_recent_stack(ItemStack::single(item));
                }
                SlotAction::RightClick => {
                    let half = ItemStack::MAX_STACK_SIZE.div_ceil(2);
                    state.cursor.set_stack(ItemStack::new(item.clone(), half));
                    state.cursor.source = None;
                    state.add_recent_stack(ItemStack::single(item));
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
                    state.add_recent_stack(ItemStack::single(stack.item.clone()));
                }
                SlotAction::RightClick => {
                    let half = ItemStack::MAX_STACK_SIZE.div_ceil(2);
                    state
                        .cursor
                        .set_stack(ItemStack::new(stack.item.clone(), half));
                    state.cursor.source = None;
                    state.add_recent_stack(ItemStack::single(stack.item.clone()));
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
                shift_click_into_range(
                    &mut state.hotbar,
                    &mut state.survival,
                    index,
                    0..SurvivalInventory::BACKPACK_SIZE,
                );
            }
            SlotAction::ScrollDown => {
                move_one_into_range(
                    &mut state.hotbar,
                    &mut state.survival,
                    index,
                    0..SurvivalInventory::BACKPACK_SIZE,
                );
            }
            SlotAction::ScrollUp => {
                pull_one_matching(
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
                SlotAction::ScrollDown if kind == SlotKind::SurvivalBackpack => {
                    let index = survival_index(kind, index).expect("checked above");
                    move_one_into_range(
                        &mut state.survival,
                        &mut state.hotbar,
                        index,
                        0..HOTBAR_SIZE,
                    );
                }
                SlotAction::ScrollUp if kind == SlotKind::SurvivalBackpack => {
                    let index = survival_index(kind, index).expect("checked above");
                    pull_one_matching(
                        &mut state.survival,
                        &mut state.hotbar,
                        index,
                        0..HOTBAR_SIZE,
                    );
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
            hotbar_stack.merge_from(&mut remaining);
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
