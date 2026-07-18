use bevy::prelude::*;

use crate::content::block::event::BlockInteractEvent;
use crate::content::block::registry::BlockRegistry;
use crate::content::recipe::registry::RecipeRegistry;
use crate::content::tag::runtime::ItemTagIndex;
use crate::game::crafting::grid::{
    ActiveCrafting, CraftingGrid, PlayerCrafting, WorkbenchCrafting,
};
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::events::SlotInteractionEvent;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::state::InventoryState;
use crate::shared::ui_types::{ContainerKind, SlotKind};

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CraftingStationOpened {
    pub position: IVec3,
}

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCrafting>()
            .init_resource::<WorkbenchCrafting>()
            .init_resource::<ActiveCrafting>()
            .add_message::<CraftingStationOpened>()
            .add_systems(
                Update,
                (
                    open_workbench_system,
                    crafting_interaction_system,
                    return_crafting_on_close_system,
                ),
            );
    }
}

fn open_workbench_system(
    mut interactions: MessageReader<BlockInteractEvent>,
    registry: Option<Res<BlockRegistry>>,
    mut active: ResMut<ActiveCrafting>,
    mut opened: MessageWriter<CraftingStationOpened>,
) {
    let Some(registry) = registry else { return };
    for event in interactions.read() {
        let is_workbench = registry
            .get_identifier_by_id(event.block_id)
            .is_some_and(|identifier| identifier == "century_journey:crafting_table");
        if !is_workbench {
            continue;
        }
        *active = ActiveCrafting::workbench(event.world_pos);
        opened.write(CraftingStationOpened {
            position: event.world_pos,
        });
    }
}

fn crafting_interaction_system(
    mut reader: MessageReader<SlotInteractionEvent>,
    mut state: ResMut<InventoryState>,
    active: Res<ActiveCrafting>,
    mut player_crafting: ResMut<PlayerCrafting>,
    mut workbench_crafting: ResMut<WorkbenchCrafting>,
    recipes: Res<RecipeRegistry>,
    tags: Option<Res<ItemTagIndex>>,
) {
    let Some(tags) = tags else { return };
    for event in reader.read() {
        let SlotKind::Container(kind) = event.kind else {
            continue;
        };
        if kind != active.kind {
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
            ContainerKind::Workbench => handle_crafting_event(
                event,
                &mut state,
                workbench_crafting.grid_mut(),
                &recipes,
                &tags,
            ),
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
            SlotAction::LeftClick => crate::game::inventory::interaction::left_click_slot(
                crafting,
                event.index,
                &mut state.cursor,
            ),
            SlotAction::RightClick => crate::game::inventory::interaction::right_click_slot(
                crafting,
                event.index,
                &mut state.cursor,
            ),
            SlotAction::ScrollDown => {
                let hotbar_slots = state.hotbar.slot_count();
                if !crate::game::inventory::interaction::move_one_into_range(
                    crafting,
                    &mut state.hotbar,
                    event.index,
                    0..hotbar_slots,
                ) {
                    crate::game::inventory::interaction::move_one_into_range(
                        crafting,
                        &mut state.survival,
                        event.index,
                        0..crate::game::inventory::container::survival::SurvivalInventory::BACKPACK_SIZE,
                    );
                }
            }
            SlotAction::ScrollUp => {
                let hotbar_slots = state.hotbar.slot_count();
                if !crate::game::inventory::interaction::pull_one_matching(
                    crafting,
                    &mut state.hotbar,
                    event.index,
                    0..hotbar_slots,
                ) {
                    crate::game::inventory::interaction::pull_one_matching(
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
    mut active: ResMut<ActiveCrafting>,
    mut player_crafting: ResMut<PlayerCrafting>,
    mut workbench_crafting: ResMut<WorkbenchCrafting>,
    mut was_opened: Local<bool>,
) {
    if *was_opened && !state.opened {
        let inputs = match active.kind {
            ContainerKind::PlayerCrafting => player_crafting.drain_inputs(),
            ContainerKind::Workbench => workbench_crafting.drain_inputs(),
            ContainerKind::Chest | ContainerKind::Furnace => Vec::new(),
        };
        for stack in inputs.into_iter().flatten() {
            let mut remaining = stack;
            let hotbar_slots = state.hotbar.slot_count();
            insert_range(&mut state.hotbar, &mut remaining, 0..hotbar_slots);
            insert_range(
                &mut state.survival,
                &mut remaining,
                0..crate::game::inventory::container::survival::SurvivalInventory::BACKPACK_SIZE,
            );
        }
        *active = ActiveCrafting::player();
    }
    *was_opened = state.opened;
}
