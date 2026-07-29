use super::transfer::{capacity_range, insert_range};
use crate::content::recipe::registry::RecipeRegistry;
use crate::content::tag::runtime::ItemTagIndex;
use crate::game::crafting::grid::{ActiveCrafting, CraftingGrid, PlayerCrafting};
use crate::game::inventory::InventoryState;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::world::WorldContainers;
use crate::game::inventory::events::SlotInteractionEvent;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::slot::SlotAction;
use crate::game::player::identity::PlayerId;
use crate::shared::ui_types::{ContainerKind, SlotKind};
use bevy::prelude::*;

pub fn crafting_interaction_system(
    mut reader: MessageReader<SlotInteractionEvent>,
    mut players: Query<(
        &PlayerId,
        &mut InventoryState,
        &mut PlayerCrafting,
        &ActiveCrafting,
    )>,
    mut containers: ResMut<WorldContainers>,
    recipes: Res<RecipeRegistry>,
    tags: Option<Res<ItemTagIndex>>,
) {
    let Some(tags) = tags else { return };
    for event in reader.read() {
        let SlotKind::Container(kind) = event.kind else {
            continue;
        };
        let Some((_, mut state, mut player_crafting, active)) = players
            .iter_mut()
            .find(|(player_id, _, _, _)| **player_id == event.player_id)
        else {
            continue;
        };
        if kind != active.kind || event.container_id != active.container_id {
            continue;
        }
        match kind {
            ContainerKind::PlayerCrafting => handle_crafting_event(
                event,
                &mut state,
                player_crafting.grid_mut(),
                &recipes,
                &tags,
            ),
            ContainerKind::Workbench => {
                let Some(container_id) = event.container_id else {
                    continue;
                };
                let Some(workbench) = containers.workbench_mut(container_id) else {
                    continue;
                };
                handle_crafting_event(event, &mut state, workbench.grid_mut(), &recipes, &tags);
            }
            ContainerKind::Chest | ContainerKind::Furnace => {}
        }
    }
}

fn handle_crafting_event(
    event: &SlotInteractionEvent,
    state: &mut InventoryState,
    crafting: &mut CraftingGrid,
    recipes: &RecipeRegistry,
    tags: &ItemTagIndex,
) {
    if event.index < crafting.slot_count() {
        match event.action {
            SlotAction::LeftClick => crate::game::inventory::interaction::click::left_click_slot(
                crafting,
                event.index,
                &mut state.cursor,
            ),
            SlotAction::RightClick => crate::game::inventory::interaction::click::right_click_slot(
                crafting,
                event.index,
                &mut state.cursor,
            ),
            SlotAction::ScrollDown => {
                let hotbar_slots = state.hotbar.slot_count();
                if !crate::game::inventory::interaction::click::move_one_into_range(
                    crafting,
                    &mut state.hotbar,
                    event.index,
                    0..hotbar_slots,
                ) {
                    crate::game::inventory::interaction::click::move_one_into_range(
                        crafting,
                        &mut state.survival,
                        event.index,
                        0..crate::game::inventory::container::survival::SurvivalInventory::BACKPACK_SIZE,
                    );
                }
            }
            SlotAction::ScrollUp => {
                let hotbar_slots = state.hotbar.slot_count();
                if !crate::game::inventory::interaction::click::pull_one_matching(
                    crafting,
                    &mut state.hotbar,
                    event.index,
                    0..hotbar_slots,
                ) {
                    crate::game::inventory::interaction::click::pull_one_matching(
                        crafting,
                        &mut state.survival,
                        event.index,
                        0..crate::game::inventory::container::survival::SurvivalInventory::BACKPACK_SIZE,
                    );
                }
            }
            _ => return,
        }
        if matches!(event.action, SlotAction::LeftClick | SlotAction::RightClick) {
            state.cursor.source = None;
        }
        crafting.refresh(recipes, tags);
    } else if event.index == crafting.slot_count() {
        match event.action {
            SlotAction::LeftClick | SlotAction::RightClick => {
                take_output(state, crafting, recipes, tags);
            }
            SlotAction::ShiftClick => {
                while take_output_to_inventory(state, crafting, recipes, tags) {}
            }
            SlotAction::ScrollDown => {
                take_output_to_inventory(state, crafting, recipes, tags);
            }
            _ => {}
        }
    }
}

fn take_output(
    state: &mut InventoryState,
    crafting: &mut CraftingGrid,
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
    crafting: &mut CraftingGrid,
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
