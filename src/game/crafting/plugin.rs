//! 组装合成网格、工作台会话和顺序受控的运行时交互系统。

use bevy::prelude::*;

use crate::game::crafting::events::CraftingStationOpened;
use crate::game::crafting::runtime::*;
use crate::game::inventory::container::world::WorldContainers;
use crate::shared::states::AppState;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum CraftingUpdateSet {
    OpenStation,
    Interaction,
    ReturnInputs,
}

/// 组装合成资源、消息和运行时系统的领域插件。
pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldContainers>()
            .add_message::<CraftingStationOpened>()
            .configure_sets(
                Update,
                (
                    CraftingUpdateSet::OpenStation,
                    CraftingUpdateSet::Interaction,
                    CraftingUpdateSet::ReturnInputs,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                open_workbench_system
                    .in_set(CraftingUpdateSet::OpenStation)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                crafting_interaction_system
                    .in_set(CraftingUpdateSet::Interaction)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                return_crafting_on_close_system
                    .in_set(CraftingUpdateSet::ReturnInputs)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
