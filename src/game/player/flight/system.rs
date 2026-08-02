//! 在固定步消费飞行切换请求并维护飞行状态

use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::player::control::action::{PlayerAction, PlayerActionState};
use crate::game::player::flight::components::{PlayerFlight, ToggleFlightRequest};
use crate::game::player::identity::Player;
use crate::game::player::physics::components::PlayerGravity;
use bevy::prelude::*;

/// 在固定步中顺序消费飞行切换请求；
/// 仅允许飞行的模式生效
pub fn toggle_flight_system(
    gamemode: Res<PlayerGameMode>,
    mut requests: MessageReader<ToggleFlightRequest>,
    mut query: Query<(&mut PlayerFlight, &mut PlayerGravity), With<Player>>,
) {
    for _ in requests.read() {
        if !flight_permitted(&gamemode) {
            continue;
        }

        let Ok((mut flight, mut gravity)) = query.single_mut() else {
            continue;
        };
        flight.enabled = !flight.enabled;

        // 关闭飞行时清零垂直速度，避免遗留上升速度导致瞬间弹飞。
        if !flight.enabled {
            gravity.velocity_y = 0.0;
        }
    }
}

/// 模式不再允许飞行时强制落地，保证切回生存后飞行状态立即失效
pub fn cleanup_flight_if_not_permitted_system(
    gamemode: Res<PlayerGameMode>,
    mut query: Query<(&mut PlayerFlight, &mut PlayerGravity), With<Player>>,
) {
    if flight_permitted(&gamemode) {
        return;
    }

    if let Ok((mut flight, mut gravity)) = query.single_mut()
        && flight.enabled
    {
        flight.enabled = false;
        gravity.velocity_y = 0.0;
    }
}

/// 判定当前是否允许飞行；未来其他模式或条件则在此扩展。
pub fn flight_permitted(gamemode: &PlayerGameMode) -> bool {
    gamemode.is_creative()
}

/// 根据输入计算本刻飞行垂直速度
pub fn flight_vertical_velocity(actions: &PlayerActionState, flight: &PlayerFlight) -> f32 {
    let mut velocity = 0.0;
    if actions.pressed(PlayerAction::Jump) {
        velocity += flight.fly_speed;
    }
    if actions.pressed(PlayerAction::Squat) {
        velocity -= flight.fly_speed;
    }
    velocity
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/player/flight/system.rs"]
mod tests;
