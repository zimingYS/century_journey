use crate::game::player;
use crate::game::player::lifecycle;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;
use bevy::prelude::*;

pub struct PlayerLifecyclePlugin;

impl Plugin for PlayerLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Startup, lifecycle::spawn::PlayerStartupSet::Authority)
            .init_resource::<lifecycle::rules::DeathRules>()
            .init_resource::<lifecycle::rules::LastDeathInfo>()
            .add_message::<lifecycle::events::DeathEvent>()
            .add_message::<lifecycle::events::RespawnRequest>()
            .add_systems(
                Startup,
                lifecycle::spawn::spawn_authoritative_player_system
                    .in_set(lifecycle::spawn::PlayerStartupSet::Authority),
            )
            .add_systems(
                FixedUpdate,
                (
                    lifecycle::rules::death_system,
                    lifecycle::rules::respawn_request_system,
                    lifecycle::rules::respawn_transition_system,
                )
                    .chain()
                    .in_set(SimulationSet::Combat)
                    .run_if(in_state(AppState::InGame))
                    .after(player::survival::health::heal_system),
            );
    }
}
