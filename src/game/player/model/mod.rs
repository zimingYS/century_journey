pub mod animation;
pub mod components;
pub mod config;
pub mod debug;
pub mod rig;

use bevy::prelude::*;

pub struct PlayerModelPlugin;

impl Plugin for PlayerModelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<config::PlayerModelConfig>()
            .add_systems(
                Update,
                (
                    animation::player_animation_controller_system,
                    animation::apply_player_rig_animation_system,
                )
                    .chain(),
            )
            .add_systems(Update, (debug::debug_skeleton_system,));
    }
}
