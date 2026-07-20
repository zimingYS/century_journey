use crate::game::inventory::state::InventoryState;
use crate::game::player::components::Player;
use crate::game::player::components::stats::Defense;
use bevy::prelude::*;

/// 每帧从头盔、胸甲、护腿和靴子槽位推导 Defense 值。
pub fn armor_calculation_system(mut query: Query<(&InventoryState, &mut Defense), With<Player>>) {
    for (inventory, mut defense) in &mut query {
        let armor_vals = [2.0, 6.0, 5.0, 2.0];
        let total: f32 = inventory
            .survival
            .equipment
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.is_some())
            .map(|(i, _)| armor_vals.get(i).copied().unwrap_or(0.0))
            .sum();
        defense.0 = total;
    }
}
