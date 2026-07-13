use bevy::prelude::*;

use crate::game::player::components::Player;
use crate::game::player::model::animation::{
    PlayerAnimationState, PlayerBehaviorState, PlayerLocomotionState,
};
use crate::game::player::model::rig::{PlayerRigEntities, held_item_grip_transform};

/// 将分层动画状态转换成骨架姿态。
///
/// 这里仅写关节 Transform，不修改移动、伤害或交互状态。
pub fn apply_player_rig_animation_system(
    state_query: Query<(&PlayerAnimationState, &PlayerRigEntities), With<Player>>,
    mut transform_query: Query<&mut Transform>,
) {
    for (state, rig) in &state_query {
        let lower = blended_lower_pose(state);
        let upper = blended_upper_pose(state);

        set_rotation(&mut transform_query, rig.body_joint, upper.body);
        set_rotation(&mut transform_query, rig.head_joint, upper.head);
        set_rotation(&mut transform_query, rig.upper_arm_r, upper.upper_arm_r);
        set_rotation(&mut transform_query, rig.upper_arm_l, upper.upper_arm_l);
        set_rotation(&mut transform_query, rig.forearm_r, upper.forearm_r);
        set_rotation(&mut transform_query, rig.forearm_l, upper.forearm_l);
        set_rotation(&mut transform_query, rig.hand_r, upper.hand_r);
        set_rotation(&mut transform_query, rig.hand_l, upper.hand_l);
        set_rotation(
            &mut transform_query,
            rig.thigh_r,
            Quat::from_rotation_x(lerp(lower.thigh_r, 0.45, upper.death_weight)),
        );
        set_rotation(
            &mut transform_query,
            rig.thigh_l,
            Quat::from_rotation_x(lerp(lower.thigh_l, -0.35, upper.death_weight)),
        );
        set_rotation(
            &mut transform_query,
            rig.calf_r,
            Quat::from_rotation_x(lerp(lower.calf_r, 0.25, upper.death_weight)),
        );
        set_rotation(
            &mut transform_query,
            rig.calf_l,
            Quat::from_rotation_x(lerp(lower.calf_l, 0.20, upper.death_weight)),
        );

        if let Ok(mut transform) = transform_query.get_mut(rig.held_item) {
            *transform = held_item_grip_transform();
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct LowerBodyPose {
    thigh_r: f32,
    thigh_l: f32,
    calf_r: f32,
    calf_l: f32,
}

impl LowerBodyPose {
    fn blend(self, other: Self, amount: f32) -> Self {
        Self {
            thigh_r: lerp(self.thigh_r, other.thigh_r, amount),
            thigh_l: lerp(self.thigh_l, other.thigh_l, amount),
            calf_r: lerp(self.calf_r, other.calf_r, amount),
            calf_l: lerp(self.calf_l, other.calf_l, amount),
        }
    }
}

fn blended_lower_pose(state: &PlayerAnimationState) -> LowerBodyPose {
    let phase = state.parameters.locomotion_phase;
    let previous = lower_pose(state.lower_body.previous, phase);
    let current = lower_pose(state.lower_body.current, phase);
    previous.blend(current, state.lower_body.blend_factor())
}

fn lower_pose(state: PlayerLocomotionState, phase: f32) -> LowerBodyPose {
    match state {
        PlayerLocomotionState::Idle => LowerBodyPose::default(),
        PlayerLocomotionState::Walk | PlayerLocomotionState::Run => {
            let amplitude = if state == PlayerLocomotionState::Run {
                0.68
            } else {
                0.42
            };
            let swing = phase.sin() * amplitude;
            LowerBodyPose {
                thigh_r: -swing,
                thigh_l: swing,
                calf_r: swing.max(0.0) * 0.35,
                calf_l: (-swing).max(0.0) * 0.35,
            }
        }
        PlayerLocomotionState::Jump => LowerBodyPose {
            thigh_r: -0.24,
            thigh_l: 0.18,
            calf_r: 0.22,
            calf_l: 0.08,
        },
        PlayerLocomotionState::Fall => LowerBodyPose {
            thigh_r: 0.18,
            thigh_l: 0.12,
            calf_r: 0.34,
            calf_l: 0.28,
        },
    }
}

#[derive(Debug, Clone, Copy)]
struct UpperBodyPose {
    body: Quat,
    head: Quat,
    upper_arm_r: Quat,
    upper_arm_l: Quat,
    forearm_r: Quat,
    forearm_l: Quat,
    hand_r: Quat,
    hand_l: Quat,
    death_weight: f32,
}

impl UpperBodyPose {
    fn blend(self, other: Self, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        Self {
            body: self.body.slerp(other.body, amount),
            head: self.head.slerp(other.head, amount),
            upper_arm_r: self.upper_arm_r.slerp(other.upper_arm_r, amount),
            upper_arm_l: self.upper_arm_l.slerp(other.upper_arm_l, amount),
            forearm_r: self.forearm_r.slerp(other.forearm_r, amount),
            forearm_l: self.forearm_l.slerp(other.forearm_l, amount),
            hand_r: self.hand_r.slerp(other.hand_r, amount),
            hand_l: self.hand_l.slerp(other.hand_l, amount),
            death_weight: lerp(self.death_weight, other.death_weight, amount),
        }
    }
}

fn blended_upper_pose(state: &PlayerAnimationState) -> UpperBodyPose {
    let base = base_upper_pose(state);
    let previous = behavior_pose(
        state.upper_body.previous,
        state.previous_behavior_progress,
        base,
    );
    let current = behavior_pose(
        state.upper_body.current,
        state.parameters.action_progress,
        base,
    );
    let transitioned = previous.blend(current, state.upper_body.blend_factor());
    base.blend(transitioned, state.upper_body.weight)
}

fn base_upper_pose(state: &PlayerAnimationState) -> UpperBodyPose {
    let move_amplitude = match state.lower_body.current {
        PlayerLocomotionState::Walk => 0.22,
        PlayerLocomotionState::Run => 0.36,
        PlayerLocomotionState::Idle | PlayerLocomotionState::Jump | PlayerLocomotionState::Fall => {
            0.0
        }
    };
    let swing = state.parameters.locomotion_phase.sin() * move_amplitude;

    if state.parameters.holding_item {
        UpperBodyPose {
            body: Quat::IDENTITY,
            head: Quat::IDENTITY,
            upper_arm_r: arm_rotation(0.72 + swing * 0.2, -0.32),
            upper_arm_l: arm_rotation(0.14 - swing, 0.07),
            forearm_r: arm_rotation(0.42 + swing * 0.08, -0.10),
            forearm_l: arm_rotation(0.18, 0.03),
            hand_r: Quat::from_rotation_x(0.10),
            hand_l: Quat::from_rotation_x(0.04),
            death_weight: 0.0,
        }
    } else {
        UpperBodyPose {
            body: Quat::IDENTITY,
            head: Quat::IDENTITY,
            upper_arm_r: arm_rotation(0.16 + swing, -0.07),
            upper_arm_l: arm_rotation(0.16 - swing, 0.07),
            forearm_r: arm_rotation(0.18 + swing * 0.25, -0.03),
            forearm_l: arm_rotation(0.18 - swing * 0.25, 0.03),
            hand_r: Quat::from_rotation_x(0.04),
            hand_l: Quat::from_rotation_x(0.04),
            death_weight: 0.0,
        }
    }
}

fn behavior_pose(
    behavior: PlayerBehaviorState,
    progress: f32,
    mut pose: UpperBodyPose,
) -> UpperBodyPose {
    let pulse = (progress.clamp(0.0, 1.0) * std::f32::consts::PI).sin();
    match behavior {
        PlayerBehaviorState::None => {}
        PlayerBehaviorState::Mining => {
            pose.body = Quat::from_rotation_y(-0.10 * pulse);
            pose.upper_arm_r = arm_rotation(0.72 + pulse * 0.86, -0.40);
            pose.forearm_r = arm_rotation(0.38 + pulse * 0.62, -0.14);
            pose.hand_r = Quat::from_rotation_x(pulse * 0.20);
        }
        PlayerBehaviorState::Placing => {
            pose.body = Quat::from_rotation_y(-0.08 * pulse);
            pose.upper_arm_r = arm_rotation(0.45 + pulse * 0.55, -0.22);
            pose.forearm_r = arm_rotation(0.30 + pulse * 0.32, -0.08);
        }
        PlayerBehaviorState::Using => {
            pose.upper_arm_r = arm_rotation(0.72 + pulse * 0.80, -0.48);
            pose.forearm_r = arm_rotation(0.42 + pulse * 0.88, -0.12);
            pose.hand_r = Quat::from_rotation_x(-pulse * 0.18);
        }
        PlayerBehaviorState::Attacking => {
            pose.body = Quat::from_rotation_y(-0.16 * pulse);
            pose.upper_arm_r = arm_rotation(0.24 + pulse * 1.12, -0.24);
            pose.forearm_r = arm_rotation(0.22 + pulse * 0.68, -0.08);
            pose.hand_r = Quat::from_rotation_x(pulse * 0.22);
        }
        PlayerBehaviorState::Hurt => {
            pose.body = Quat::from_rotation_z(0.14 * pulse) * Quat::from_rotation_x(-0.16 * pulse);
            pose.head = Quat::from_rotation_x(0.12 * pulse);
            pose.upper_arm_r = arm_rotation(-0.20 * pulse, -0.32);
            pose.upper_arm_l = arm_rotation(-0.16 * pulse, 0.32);
        }
        PlayerBehaviorState::Death => {
            let amount = smoothstep(progress);
            pose.body = Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2 * amount);
            pose.head = Quat::from_rotation_x(0.24 * amount);
            pose.upper_arm_r = arm_rotation(-0.30 * amount, -0.28);
            pose.upper_arm_l = arm_rotation(-0.24 * amount, 0.28);
            pose.forearm_r = arm_rotation(0.12, -0.10);
            pose.forearm_l = arm_rotation(0.12, 0.10);
            pose.death_weight = amount;
        }
    }
    pose
}

fn arm_rotation(x: f32, z: f32) -> Quat {
    Quat::from_rotation_z(z) * Quat::from_rotation_x(x)
}

fn set_rotation(transform_query: &mut Query<&mut Transform>, entity: Entity, rotation: Quat) {
    if let Ok(mut transform) = transform_query.get_mut(entity) {
        transform.rotation = rotation;
    }
}

fn lerp(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount.clamp(0.0, 1.0)
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}
