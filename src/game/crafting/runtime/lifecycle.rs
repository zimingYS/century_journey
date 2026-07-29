use crate::game::crafting::grid::{ActiveCrafting, PlayerCrafting};
use crate::game::crafting::runtime::transfer::insert_range;
use crate::game::inventory::InventoryState;
use crate::game::inventory::container::InventoryContainer;
use crate::shared::ui_types::ContainerKind;
use bevy::prelude::Query;

pub fn return_crafting_on_close_system(
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
