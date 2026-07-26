use crate::game::player::physics;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;
use bevy::prelude::*;

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
