use crate::game::player;
use crate::game::player::combat::events::AttackEvent;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;
use bevy::prelude::*;

pub struct PlayerCombatPlugin;

impl Plugin for PlayerCombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<AttackEvent>().add_systems(
            FixedUpdate,
            (
                player::combat::attack::melee_attack_input_system,
                player::combat::attack::attack_damage_system,
            )
                .in_set(SimulationSet::Combat)
                .run_if(in_state(AppState::InGame))
                .chain(),
        );
    }
}
