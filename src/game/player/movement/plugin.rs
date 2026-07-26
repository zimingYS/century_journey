use crate::game::player::movement;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;
use bevy::prelude::*;

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
