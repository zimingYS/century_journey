use crate::game::player::control::action;
use crate::game::player::control::command;
use crate::game::player::control::command::apply_player_command_system;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;
use bevy::prelude::*;

pub struct PlayerControlPlugin;

impl Plugin for PlayerControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<action::PlayerActionState>()
            .init_resource::<command::PlayerCommandBuffer>()
            .add_systems(
                OnEnter(AppState::InGame),
                command::reset_player_command_pipeline_system,
            )
            .add_systems(
                FixedUpdate,
                apply_player_command_system
                    .in_set(SimulationSet::Commands)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
