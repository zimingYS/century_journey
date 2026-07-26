use crate::game::player;
use bevy::prelude::*;

/// 纯游戏逻辑 Plugin — 仅注册 Game 层系统，不依赖 Client。
pub struct GamePlayerPlugin;

impl Plugin for GamePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(player::control::plugin::PlayerControlPlugin)
            .add_plugins(player::interaction::plugin::PlayerInteractionPlugin)
            .add_plugins(player::movement::plugin::PlayerMovementPlugin)
            .add_plugins(player::physics::plugin::PlayerPhysicsPlugin)
            .add_plugins(player::survival::plugin::PlayerSurvivalPlugin)
            .add_plugins(player::lifecycle::plugin::PlayerLifecyclePlugin)
            .add_plugins(player::combat::plugin::PlayerCombatPlugin);
    }
}
