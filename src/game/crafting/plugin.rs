//! 组装合成网格、工作台会话和固定步交互系统。

use bevy::prelude::*;

use crate::game::crafting::events::CraftingStationOpened;
use crate::game::crafting::runtime::*;
use crate::game::inventory::InventorySet;
use crate::game::player::interaction::voxel::voxel_interaction_system;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;

/// 组装合成资源、消息和权威固定步系统的领域插件。
pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CraftingStationOpened>()
            .add_systems(
                FixedUpdate,
                open_workbench_system
                    .in_set(SimulationSet::Interaction)
                    .after(voxel_interaction_system)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                crafting_interaction_system
                    .in_set(SimulationSet::Commands)
                    .after(InventorySet::SlotInteraction)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                return_crafting_on_close_system
                    .in_set(SimulationSet::Commands)
                    .after(crafting_interaction_system)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
