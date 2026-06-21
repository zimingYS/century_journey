use bevy::prelude::*;
use crate::game::inventory::state::InventoryState;
use crate::game::player::components::stats::Defense;
use crate::game::player::components::Player;

/// 每帧从 survival.armor 槽位推导 Defense 值
pub fn armor_calculation_system(
    inventory: Res<InventoryState>,
    mut query: Query<&mut Defense, With<Player>>,
) {
    let armor_vals = [2.0, 6.0, 5.0, 2.0]; // helmet, chest, legs, boots
    let total: f32 = inventory.survival.armor.iter().enumerate()
        .filter(|(_, slot)| slot.is_some())
        .map(|(i, _)| armor_vals.get(i).copied().unwrap_or(0.0))
        .sum();

    for mut defense in &mut query {
        defense.0 = total;
    }
}