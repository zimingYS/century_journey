//! 组装目标检测、方块破坏与放置等玩家交互系统。

use crate::game::gameplay::block_action::{BlockBreakProgress, BlockBreakState};
use crate::game::player;
use crate::game::player::interaction::targeting::TargetVoxel;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;
use bevy::prelude::*;

/// 注册目标检测资源以及体素交互固定步管线。
pub struct PlayerInteractionPlugin;

impl Plugin for PlayerInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TargetVoxel>()
            .init_resource::<BlockBreakState>()
            .init_resource::<BlockBreakProgress>()
            .add_systems(
                FixedUpdate,
                player::interaction::targeting::update_raycast_system
                    .in_set(SimulationSet::Targeting)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                (
                    player::interaction::voxel::voxel_interaction_system,
                    player::interaction::voxel::drop_active_hotbar_action_system,
                    player::interaction::voxel::drop_item_system,
                )
                    .chain()
                    .in_set(SimulationSet::Interaction)
                    .run_if(in_state(AppState::InGame))
                    .run_if(player::lifecycle::rules::player_is_alive),
            );
    }
}
