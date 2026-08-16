//! 组织玩家模型骨架、部件配置、动画姿态和调试表现。

pub mod animation;
pub mod animation_pose;
pub mod components;
pub mod config;
pub mod debug;
pub mod gltf_rig;
pub mod rig;

use bevy::prelude::*;

/// 注册玩家骨架生成、动画驱动和调试表现系统。
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
                    animation_pose::enforce_feet_offset_system,
                )
                    .chain()
                    .before(TransformSystems::Propagate)
                    .run_if(in_state(crate::shared::states::AppState::InGame)),
            )
            .add_observer(gltf_rig::bind_player_rig_on_ready)
            .add_systems(
                Update,
                debug::debug_skeleton_system
                    .run_if(in_state(crate::shared::states::AppState::InGame)),
            );
    }
}
