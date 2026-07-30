//! 组装通用玩法资源和固定步命令系统。

use bevy::prelude::*;

use super::gamemode::{PlayerGameMode, ToggleGameModeRequest, toggle_gamemode_system};
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;

/// Game 层通用玩法插件。
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerGameMode>()
            .add_message::<ToggleGameModeRequest>()
            .add_systems(
                FixedUpdate,
                toggle_gamemode_system
                    .in_set(SimulationSet::Commands)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
