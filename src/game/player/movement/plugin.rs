//! 在命令应用后组装固定步玩家移动系统。

use crate::game::player::movement;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;
use bevy::prelude::*;

/// 把玩家移动规则注册到命令之后、物理之前的固定步阶段。
pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            movement::system::player_movement_system
                .in_set(SimulationSet::Movement)
                .run_if(in_state(AppState::InGame)),
        );
    }
}
