use crate::game::player::model::components::{PlayerJoint, PlayerPart};
use bevy::prelude::*;

/// 动画状态
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerAnimationState {
    #[default]
    Idle,
    Walk,
    Run,
    Jump,
    Fall,
    Mine,
    Attack,
}

/// 动画状态切换
pub fn player_animation_controller_system(
    input: Res<ButtonInput<KeyCode>>,
    player_query: Query<
        &Transform,
        (
            With<crate::game::player::components::Player>,
            Without<PlayerJoint>,
        ),
    >,
    mut state_query: Query<&mut PlayerAnimationState>,
    mut prev_pos: Local<Option<Vec3>>,
) {
    let Ok(transform) = player_query.single() else {
        return;
    };
    let speed = prev_pos.map_or(0.0, |p| (transform.translation - p).length());
    *prev_pos = Some(transform.translation);

    for mut state in &mut state_query {
        *state = if input.pressed(KeyCode::Space) {
            PlayerAnimationState::Jump
        } else if speed < 0.01 {
            PlayerAnimationState::Idle
        } else if input.pressed(KeyCode::ShiftLeft) {
            PlayerAnimationState::Walk
        } else if speed > 0.3 {
            PlayerAnimationState::Run
        } else {
            PlayerAnimationState::Walk
        };
    }
}

/// Base 层: 行走/跑步 — 仅旋转 Joint Transform
pub fn walk_animation_system(
    time: Res<Time>,
    state_query: Query<&PlayerAnimationState>,
    mut joint_query: Query<(&PlayerJoint, &mut Transform)>,
) {
    let Ok(state) = state_query.single() else {
        return;
    };
    if !matches!(
        *state,
        PlayerAnimationState::Walk | PlayerAnimationState::Run
    ) {
        return;
    }

    let factor = if *state == PlayerAnimationState::Run {
        2.0
    } else {
        1.0
    };
    let t = time.elapsed_secs() as f32 * 8.0 * factor;
    let swing = (t.sin() * 0.6) as f32;

    for (joint, mut transform) in &mut joint_query {
        let part = joint.0;
        // 手臂与对角腿反向摆动
        if part == PlayerPart::upper_arm_r() || part == PlayerPart::thigh_l() {
            transform.rotation = Quat::from_rotation_x(swing);
        } else if part == PlayerPart::upper_arm_l() || part == PlayerPart::thigh_r() {
            transform.rotation = Quat::from_rotation_x(-swing);
        } else if matches!(part, PlayerPart::CalfL(_)) {
            transform.rotation = Quat::from_rotation_x(-swing.abs() * 0.2);
        }
    }
}

/// 待机: 重置所有关节
pub fn idle_reset_system(
    state_query: Query<&PlayerAnimationState>,
    mut joint_query: Query<(&PlayerJoint, &mut Transform)>,
) {
    let Ok(state) = state_query.single() else {
        return;
    };
    if *state != PlayerAnimationState::Idle {
        return;
    }
    for (_joint, mut transform) in &mut joint_query {
        transform.rotation = Quat::IDENTITY;
    }
}
