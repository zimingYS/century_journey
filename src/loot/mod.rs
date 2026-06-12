pub mod table;
pub mod block_registry;

use bevy::prelude::*;
use crate::core::state::app_state::AppState;

pub struct LootPlugin;

impl Plugin for LootPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<block_registry::BlockLootRegistry>()
            .add_systems(OnEnter(AppState::InGame), block_registry::init_default_loot_system);
    }
}