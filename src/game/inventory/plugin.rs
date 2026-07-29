use bevy::prelude::*;

use crate::game::inventory::container::world::WorldContainers;
use crate::game::inventory::events::{
    DropItemEvent, InventoryCommand, InventoryFeedbackEvent, SlotInteractionEvent,
};
use crate::game::inventory::runtime;
use crate::game::player::control::command::apply_player_command_system;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;

/// Game 层 Inventory 模块 Plugin。
///
/// 只负责 Game 层运行时系统。
/// Definition/Registry/Loader/Texture 已在 Content 层的 ItemContentPlugin 中注册。
pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::game::inventory::state::AccessorySlotDefinitions>()
            .init_resource::<WorldContainers>()
            .add_message::<SlotInteractionEvent>()
            .add_message::<DropItemEvent>()
            .add_message::<InventoryCommand>()
            .add_message::<InventoryFeedbackEvent>()
            .add_systems(
                Update,
                (
                    runtime::handle_slot_interaction_system,
                    runtime::handle_inventory_command_system,
                ),
            )
            .add_systems(
                FixedUpdate,
                runtime::handle_hotbar_command_system
                    .after(apply_player_command_system)
                    .in_set(SimulationSet::Commands)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
