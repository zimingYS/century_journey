//! 组装飞行切换系统到固定步命令阶段。

use crate::game::player::flight;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;
use bevy::prelude::*;

/// 把飞行切换与落地兜底注册到固定步命令阶段。
pub struct PlayerFlightPlugin;

impl Plugin for PlayerFlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<flight::components::ToggleFlightRequest>()
            .add_systems(
                FixedUpdate,
                (
                    flight::system::toggle_flight_system,
                    flight::system::cleanup_flight_if_not_permitted_system,
                )
                    .in_set(SimulationSet::Commands)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
