//! 组装世界交互相关的固定步系统。

use super::changes::propagate_block_changes_system;
use super::pickup::pickup_system;
use super::support::remove_unsupported_blocks_system;
use crate::game::simulation::SimulationSet;
use crate::game::world::entity::dropped_item::dropped_item_tick_system;
use crate::shared::states::AppState;
use bevy::app::{App, FixedUpdate, Plugin};
use bevy::prelude::{IntoScheduleConfigs, in_state};

/// 组装方块交互与掉落物拾取等世界交互规则。
///
/// 拾取必须发生在掉落物生命周期更新之后，确保本固定步内的可拾取状态已经确定。
pub(in crate::game::world) struct WorldInteractionPlugin;

impl Plugin for WorldInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                propagate_block_changes_system,
                remove_unsupported_blocks_system,
            )
                .chain()
                .after(SimulationSet::Environment)
                .before(SimulationSet::Survival)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            FixedUpdate,
            pickup_system
                .after(dropped_item_tick_system)
                .in_set(SimulationSet::Entities)
                .run_if(in_state(AppState::InGame)),
        );
    }
}
