use bevy::prelude::*;

use crate::content::block::event::BlockInteractEvent;
use crate::content::block::registry::BlockRegistry;
use crate::content::recipe::registry::RecipeRegistry;
use crate::content::tag::runtime::ItemTagIndex;
use crate::game::crafting::grid::{ActiveCrafting, CraftingGrid, PlayerCrafting};
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::world::{ContainerId, WorldContainers};
use crate::game::inventory::events::SlotInteractionEvent;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::slot::SlotAction;
use crate::game::inventory::state::InventoryState;
use crate::game::player::components::PlayerId;
use crate::shared::ui_types::{ContainerKind, SlotKind};

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CraftingStationOpened {
    pub player_id: PlayerId,
    pub container_id: ContainerId,
    pub position: IVec3,
}

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldContainers>()
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
    mut players: Query<(&PlayerId, &mut ActiveCrafting)>,
    mut containers: ResMut<WorldContainers>,
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
        let Some(interactor) = event.interactor else {
            continue;
        };
        let Ok((player_id, mut active)) = players.get_mut(interactor) else {
            continue;
        };
        let Some(container_id) = containers.ensure_at(event.world_pos, ContainerKind::Workbench)
        else {
            continue;
        };
        *active = ActiveCrafting::workbench(event.world_pos, container_id);
        opened.write(CraftingStationOpened {
            player_id: *player_id,
            container_id,
            position: event.world_pos,
        });
    }
}

fn crafting_interaction_system(
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
    mut players: Query<(
        &mut InventoryState,
        &mut ActiveCrafting,
        &mut PlayerCrafting,
    )>,
) {
    for (mut state, mut active, mut player_crafting) in &mut players {
        if active.was_opened && !state.opened {
            let inputs = match active.kind {
                ContainerKind::PlayerCrafting => player_crafting.drain_inputs(),
                ContainerKind::Workbench | ContainerKind::Chest | ContainerKind::Furnace => {
                    Vec::new()
                }
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
        active.was_opened = state.opened;
    }
}

#[cfg(test)]
mod isolation_tests {
    use super::*;
    use crate::game::inventory::container::InventoryContainer;
    use crate::shared::item_id::ItemId;

    #[test]
    fn two_players_and_two_workbenches_are_fully_isolated() {
        let mut app = App::new();
        app.init_resource::<RecipeRegistry>()
            .init_resource::<ItemTagIndex>()
            .init_resource::<WorldContainers>()
            .add_message::<SlotInteractionEvent>()
            .add_systems(Update, crafting_interaction_system);

        let (first_container, second_container) = {
            let mut containers = app.world_mut().resource_mut::<WorldContainers>();
            (
                containers
                    .ensure_at(IVec3::new(1, 2, 3), ContainerKind::Workbench)
                    .unwrap(),
                containers
                    .ensure_at(IVec3::new(9, 2, 3), ContainerKind::Workbench)
                    .unwrap(),
            )
        };
        let first_player_id = PlayerId::new(1);
        let second_player_id = PlayerId::new(2);
        let first_item = ItemId::item("test:first_player_item");
        let second_item = ItemId::item("test:second_player_item");

        let mut first_inventory = InventoryState::default();
        first_inventory
            .cursor
            .set_stack(ItemStack::single(first_item.clone()));
        let mut second_inventory = InventoryState::default();
        second_inventory
            .cursor
            .set_stack(ItemStack::single(second_item.clone()));

        let first_player = app
            .world_mut()
            .spawn((
                first_player_id,
                first_inventory,
                PlayerCrafting::default(),
                ActiveCrafting::workbench(IVec3::new(1, 2, 3), first_container),
            ))
            .id();
        app.world_mut().spawn((
            second_player_id,
            second_inventory,
            PlayerCrafting::default(),
            ActiveCrafting::workbench(IVec3::new(9, 2, 3), second_container),
        ));

        app.world_mut().write_message(SlotInteractionEvent {
            player_id: first_player_id,
            container_id: Some(first_container),
            kind: SlotKind::Container(ContainerKind::Workbench),
            index: 0,
            action: SlotAction::LeftClick,
        });
        app.world_mut().write_message(SlotInteractionEvent {
            player_id: second_player_id,
            container_id: Some(second_container),
            kind: SlotKind::Container(ContainerKind::Workbench),
            index: 0,
            action: SlotAction::LeftClick,
        });
        app.update();

        let containers = app.world().resource::<WorldContainers>();
        assert_eq!(
            containers
                .workbench(first_container)
                .and_then(|workbench| workbench.get_stack(0))
                .map(|stack| &stack.item),
            Some(&first_item)
        );
        assert_eq!(
            containers
                .workbench(second_container)
                .and_then(|workbench| workbench.get_stack(0))
                .map(|stack| &stack.item),
            Some(&second_item)
        );
        assert_ne!(first_container, second_container);

        let mut inventories = app.world_mut().query::<(&PlayerId, &InventoryState)>();
        for (player_id, inventory) in inventories.iter(app.world()) {
            assert!(
                !inventory.cursor.has_item(),
                "cursor was not consumed for {player_id:?}"
            );
        }

        let cross_item = ItemId::item("test:cross_container_attempt");
        app.world_mut()
            .get_mut::<InventoryState>(first_player)
            .unwrap()
            .cursor
            .set_stack(ItemStack::single(cross_item.clone()));
        app.world_mut().write_message(SlotInteractionEvent {
            player_id: first_player_id,
            container_id: Some(second_container),
            kind: SlotKind::Container(ContainerKind::Workbench),
            index: 1,
            action: SlotAction::LeftClick,
        });
        app.update();

        assert!(
            app.world()
                .resource::<WorldContainers>()
                .workbench(second_container)
                .unwrap()
                .get_stack(1)
                .is_none()
        );
        assert_eq!(
            app.world()
                .get::<InventoryState>(first_player)
                .unwrap()
                .cursor
                .stack()
                .map(|stack| &stack.item),
            Some(&cross_item)
        );
    }
}
