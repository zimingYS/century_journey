use crate::game::inventory::state::InventoryState;
use crate::game::player::identity::Player;
use bevy::prelude::{Component, Query, With};

/// 防御值
#[derive(Component, Debug, Clone, Default)]
pub struct Defense(pub f32);

impl Defense {
    pub fn damage_reduction(&self) -> f32 {
        if !self.0.is_finite() {
            return 0.0;
        }
        let defense = self.0.max(0.0);
        defense / (defense + 10.0)
    }
}

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
