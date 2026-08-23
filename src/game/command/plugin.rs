//! 组装指令系统的消息与固定步执行系统。

use bevy::prelude::*;

use crate::game::command::components::{CommandOutput, GameCommandSubmitted};
use crate::game::command::execute::execute_game_command_system;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;

/// Game 层指令插件：注册指令消息并在固定步指令阶段执行。
pub struct CommandPlugin;

impl Plugin for CommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GameCommandSubmitted>()
            .add_message::<CommandOutput>()
            .add_systems(
                FixedUpdate,
                execute_game_command_system
                    .in_set(SimulationSet::Commands)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
