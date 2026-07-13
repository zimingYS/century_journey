use bevy::prelude::*;

use crate::client::viewmodel::ViewModelRoot;
use crate::game::player::components::LocalPlayer;
use crate::game::player::model::animation::{
    PlayerAnimationState, PlayerBehaviorState, PlayerLocomotionState,
};

/// 旧 ViewModel 的兼容适配器。
///
/// 当前第一人称直接显示真实 PlayerRig；如果以后重新启用独立镜头模型，它也只能读取共享动画参数，
/// 不能维护自己的攻击、使用或移动状态。
pub fn view_model_animation_system(
    player_query: Query<&PlayerAnimationState, With<LocalPlayer>>,
    mut view_model_query: Query<&mut Transform, With<ViewModelRoot>>,
) {
    let Ok(state) = player_query.single() else {
        return;
    };

    let locomotion_sway = match state.lower_body.current {
        PlayerLocomotionState::Walk => 0.012,
        PlayerLocomotionState::Run => 0.020,
        PlayerLocomotionState::Idle | PlayerLocomotionState::Jump | PlayerLocomotionState::Fall => {
            0.006
        }
    };
    let phase = state.parameters.locomotion_phase;
    let sway_x = phase.sin() * locomotion_sway;
    let sway_y = phase.cos() * locomotion_sway * 0.65;
    let action_pulse = (state.parameters.action_progress * std::f32::consts::PI).sin();

    let (action_rotation, action_translation) = match state.upper_body.current {
        PlayerBehaviorState::Mining | PlayerBehaviorState::Attacking => (
            Quat::from_rotation_x(-0.72 * action_pulse)
                * Quat::from_rotation_z(-0.18 * action_pulse),
            Vec3::new(0.05 * action_pulse, -0.14 * action_pulse, 0.0),
        ),
        PlayerBehaviorState::Placing => (
            Quat::from_rotation_x(-0.28 * action_pulse),
            Vec3::new(0.0, -0.04 * action_pulse, -0.10 * action_pulse),
        ),
        PlayerBehaviorState::Using => (
            Quat::from_rotation_x(-0.48 * action_pulse),
            Vec3::new(0.0, 0.08 * action_pulse, 0.04 * action_pulse),
        ),
        PlayerBehaviorState::Hurt => (
            Quat::from_rotation_z(0.16 * action_pulse),
            Vec3::new(0.06 * action_pulse, 0.0, 0.0),
        ),
        PlayerBehaviorState::Death | PlayerBehaviorState::None => (Quat::IDENTITY, Vec3::ZERO),
    };

    for mut transform in &mut view_model_query {
        transform.rotation =
            action_rotation * Quat::from_rotation_z(sway_x) * Quat::from_rotation_x(sway_y);
        transform.translation = action_translation + Vec3::new(sway_x * 0.25, sway_y * 0.25, 0.0);
    }
}
