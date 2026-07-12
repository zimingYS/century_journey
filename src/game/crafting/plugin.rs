use bevy::prelude::*;

use crate::content::recipe::registry::RecipeRegistry;
use crate::content::tag::runtime::ItemTagIndex;
use crate::game::crafting::grid::PlayerCrafting;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::events::SlotInteractionEvent;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::state::InventoryState;
use crate::shared::ui_types::SlotKind;

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCrafting>().add_systems(
            Update,
            (crafting_interaction_system, return_crafting_on_close_system),
        );
    }
}

fn crafting_interaction_system(
    mut reader: MessageReader<SlotInteractionEvent>,
    mut state: ResMut<InventoryState>,
    mut crafting: ResMut<PlayerCrafting>,
    recipes: Res<RecipeRegistry>,
    tags: Option<Res<ItemTagIndex>>,
) {
    let Some(tags) = tags else { return };
    for event in reader.read() {
        if event.kind != SlotKind::Container {
            continue;
        }
        if event.index < PlayerCrafting::SLOT_COUNT {
            match event.action {
                SlotAction::LeftClick => crate::game::inventory::interaction::left_click_slot(
                    &mut *crafting,
                    event.index,
                    &mut state.cursor,
                ),
                SlotAction::RightClick => crate::game::inventory::interaction::right_click_slot(
                    &mut *crafting,
                    event.index,
                    &mut state.cursor,
                ),
                _ => continue,
            }
            state.cursor.source = None;
            crafting.refresh(&recipes, &tags);
        } else if event.index == PlayerCrafting::SLOT_COUNT {
            match event.action {
                SlotAction::LeftClick | SlotAction::RightClick => {
                    take_output(&mut state, &mut crafting, &recipes, &tags);
                }
                SlotAction::ShiftClick => {
                    while take_output_to_inventory(&mut state, &mut crafting, &recipes, &tags) {}
                }
                _ => {}
            }
        }
    }
}

fn take_output(
    state: &mut InventoryState,
    crafting: &mut PlayerCrafting,
    recipes: &RecipeRegistry,
    tags: &ItemTagIndex,
) {
    let Some(result) = crafting.output().cloned() else {
        return;
    };
    let can_take = state.cursor.stack().is_none_or(|cursor| {
        cursor.item == result.item
            && cursor.count.saturating_add(result.count) <= ItemStack::MAX_STACK_SIZE
    });
    if !can_take {
        return;
    }
    if let Some(cursor) = state.cursor.stack_mut() {
        cursor.count += result.count;
    } else {
        state.cursor.set_stack(result);
    }
    state.cursor.source = None;
    crafting.consume_recipe();
    crafting.refresh(recipes, tags);
}

fn take_output_to_inventory(
    state: &mut InventoryState,
    crafting: &mut PlayerCrafting,
    recipes: &RecipeRegistry,
    tags: &ItemTagIndex,
) -> bool {
    let Some(result) = crafting.output().cloned() else {
        return false;
    };
    if capacity_range(&state.hotbar, &result, 0..state.hotbar.slot_count())
        + capacity_range(
            &state.survival,
            &result,
            0..crate::game::inventory::container::survival::SurvivalInventory::BACKPACK_SIZE,
        )
        < result.count
    {
        return false;
    }

    let mut remaining = result;
    let hotbar_slots = state.hotbar.slot_count();
    insert_range(&mut state.hotbar, &mut remaining, 0..hotbar_slots);
    insert_range(
        &mut state.survival,
        &mut remaining,
        0..crate::game::inventory::container::survival::SurvivalInventory::BACKPACK_SIZE,
    );
    if !remaining.is_empty() {
        return false;
    }
    crafting.consume_recipe();
    crafting.refresh(recipes, tags);
    true
}

fn capacity_range<C: InventoryContainer + ?Sized>(
    container: &C,
    incoming: &ItemStack,
    range: std::ops::Range<usize>,
) -> u32 {
    range
        .map(|index| match container.get_stack(index) {
            Some(stack) if stack.item == incoming.item => stack.remaining_space(),
            None => ItemStack::MAX_STACK_SIZE,
            _ => 0,
        })
        .sum()
}

fn insert_range<C: InventoryContainer + ?Sized>(
    container: &mut C,
    incoming: &mut ItemStack,
    range: std::ops::Range<usize>,
) {
    for index in range.clone() {
        if incoming.is_empty() {
            return;
        }
        if let Some(stack) = container.get_stack_mut(index)
            && stack.item == incoming.item
        {
            stack.merge_from(incoming);
        }
    }
    for index in range {
        if incoming.is_empty() {
            return;
        }
        if container.get_stack(index).is_none_or(ItemStack::is_empty) {
            container.set_stack(index, std::mem::take(incoming));
        }
    }
}

fn return_crafting_on_close_system(
    mut state: ResMut<InventoryState>,
    mut crafting: ResMut<PlayerCrafting>,
    mut was_opened: Local<bool>,
) {
    if *was_opened && !state.opened {
        for stack in crafting.drain_inputs().into_iter().flatten() {
            let mut remaining = stack;
            let hotbar_slots = state.hotbar.slot_count();
            insert_range(&mut state.hotbar, &mut remaining, 0..hotbar_slots);
            insert_range(
                &mut state.survival,
                &mut remaining,
                0..crate::game::inventory::container::survival::SurvivalInventory::BACKPACK_SIZE,
            );
        }
    }
    *was_opened = state.opened;
}
