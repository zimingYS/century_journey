use crate::game::inventory::state::InventoryState;
use crate::game::player::control::action::{PlayerAction, PlayerActionState};
use crate::game::player::identity::LocalPlayer;
use bevy::prelude::{Query, Res, With};

pub fn handle_hotbar_command_system(
    actions: Res<PlayerActionState>,
    mut inventory_query: Query<&mut InventoryState, With<LocalPlayer>>,
) {
    let Ok(mut inventory) = inventory_query.single_mut() else {
        return;
    };
    let direct = [
        PlayerAction::Hotbar1,
        PlayerAction::Hotbar2,
        PlayerAction::Hotbar3,
        PlayerAction::Hotbar4,
        PlayerAction::Hotbar5,
        PlayerAction::Hotbar6,
        PlayerAction::Hotbar7,
        PlayerAction::Hotbar8,
        PlayerAction::Hotbar9,
    ];
    for (index, action) in direct.into_iter().enumerate() {
        if actions.just_pressed(action) {
            inventory.hotbar.active_index = index;
            return;
        }
    }
    if actions.just_pressed(PlayerAction::HotbarPrevious) {
        inventory.hotbar.select_prev();
    }
    if actions.just_pressed(PlayerAction::HotbarNext) {
        inventory.hotbar.select_next();
    }
}
