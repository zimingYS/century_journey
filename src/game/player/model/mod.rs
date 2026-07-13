pub mod animation;
pub mod animation_pose;
pub mod components;
pub mod config;
pub mod debug;
pub mod rig;

use bevy::prelude::*;

pub struct PlayerModelPlugin;

impl Plugin for PlayerModelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<config::PlayerModelConfig>()
            .init_resource::<animation::PlayerAnimationConfig>()
            .add_message::<animation::AnimationMarkerEvent>()
            .add_systems(
                PostUpdate,
                (
                    animation::player_animation_controller_system,
                    animation::emit_animation_marker_system,
                    animation_pose::apply_player_rig_animation_system,
                )
                    .chain()
                    .before(bevy::transform::TransformSystems::Propagate)
                    .run_if(in_state(crate::shared::states::AppState::InGame)),
            )
            .add_systems(
                Update,
                debug::debug_skeleton_system
                    .run_if(in_state(crate::shared::states::AppState::InGame)),
            );
    }
}
