//! 在移动之后组装玩家重力和碰撞固定步系统。

use crate::game::player::physics;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;
use bevy::prelude::*;

/// 把玩家重力和碰撞注册到移动之后的固定步物理阶段。
pub struct PlayerPhysicsPlugin;

impl Plugin for PlayerPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            physics::gravity::player_gravity_system
                .in_set(SimulationSet::Physics)
                .run_if(in_state(AppState::InGame)),
        );
    }
}
