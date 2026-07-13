pub mod block_registry;
pub mod loader;
pub mod table;

use crate::shared::states::app_state::AppState;
use bevy::prelude::*;

pub struct LootPlugin;

impl Plugin for LootPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<block_registry::BlockLootRegistry>()
            .add_systems(
                OnEnter(AppState::InGame),
                block_registry::init_default_loot_system
                    .run_if(crate::app::flow::fresh_game_session),
            );
    }
}
