//! 组装通用玩法资源和固定步系统。

use super::gamemode::PlayerGameMode;
use crate::game::gameplay::rules::GameRules;
use bevy::prelude::*;

/// Game 层通用玩法插件。
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerGameMode>()
            .init_resource::<GameRules>();
    }
}
