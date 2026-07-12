use crate::game::inventory::state::InventoryState;
use crate::game::player::action::{PlayerAction, PlayerActionState};
use crate::game::player::components::{LocalPlayer, Player, PlayerGravity};
use crate::game::player::model::rig::{PlayerRigEntities, held_item_grip_transform};
use bevy::prelude::*;

/// 玩家下半身/整体移动状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerLocomotionState {
    /// 原地站立。
    #[default]
    Idle,
    /// 行走。
    Walk,
    /// 奔跑。
    Run,
    /// 起跳上升。
    Jump,
    /// 空中下落。
    Fall,
}

/// 玩家上半身/手部动作状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerHandAction {
    /// 空手自然下垂。
    #[default]
    EmptyHandIdle,
    /// 空手移动摆臂。
    EmptyHandMove,
    /// 空手挥拳，暂时只表现动画，伤害判定后续接入战斗系统。
    EmptyHandAttack,
    /// 空手短暂伸手，用于交互表现。
    EmptyHandInteract,
    /// 空手预备姿态，后续可接入战斗/潜行/举拳模式。
    EmptyHandReady,
    /// 持物静止姿态。
    HeldItemIdle,
    /// 持物移动姿态。
    HeldItemMove,
    /// 持物攻击/挖掘姿态。
    HeldItemAttack,
    /// 持物使用/交互姿态。
    HeldItemInteract,
}

/// 统一的玩家动画状态。
///
/// 第一人称和第三人称都读取这份状态来驱动同一个 PlayerRig。第一人称只允许在相机和可见性上做少量本地增强，
/// 不再拥有一套独立手臂动画。
#[derive(Component, Debug, Clone)]
pub struct PlayerAnimationState {
    /// 当前移动状态。
    pub locomotion: PlayerLocomotionState,
    /// 当前手部动作状态。
    pub hand_action: PlayerHandAction,
    /// 当前动作剩余时间。
    pub action_timer: f32,
    /// 当前动作总时长，用于计算归一化进度。
    pub action_duration: f32,
    /// 步行动画相位。
    pub movement_phase: f32,
    /// 上一帧位置，用于估算本地玩家水平速度。
    pub previous_position: Option<Vec3>,
    /// 当前是否手持非空气物品。
    pub holding_item: bool,
}

impl Default for PlayerAnimationState {
    fn default() -> Self {
        Self {
            locomotion: PlayerLocomotionState::Idle,
            hand_action: PlayerHandAction::EmptyHandIdle,
            action_timer: 0.0,
            action_duration: 0.0,
            movement_phase: 0.0,
            previous_position: None,
            holding_item: false,
        }
    }
}

impl PlayerAnimationState {
    /// 当前瞬时动作是否仍在播放。
    pub fn has_active_action(&self) -> bool {
        self.action_timer > 0.0 && self.action_duration > 0.0
    }

    /// 当前瞬时动作的 0..1 播放进度。
    pub fn action_progress(&self) -> f32 {
        if self.action_duration <= 0.0 {
            return 1.0;
        }
        (1.0 - self.action_timer / self.action_duration).clamp(0.0, 1.0)
    }

    /// 根据移动和持物状态回到基础手部姿态。
    pub fn settle_to_base_hand_action(&mut self) {
        let moving = matches!(
            self.locomotion,
            PlayerLocomotionState::Walk | PlayerLocomotionState::Run
        );
        self.hand_action = match (self.holding_item, moving) {
            (true, true) => PlayerHandAction::HeldItemMove,
            (true, false) => PlayerHandAction::HeldItemIdle,
            (false, true) => PlayerHandAction::EmptyHandMove,
            (false, false) => PlayerHandAction::EmptyHandIdle,
        };
    }

    /// 触发一次短动作。
    pub fn trigger_action(&mut self, action: PlayerHandAction, duration: f32) {
        self.hand_action = action;
        self.action_timer = duration;
        self.action_duration = duration;
    }
}

/// 本地玩家动画状态控制。
///
/// 这个系统只负责把输入、移动和当前快捷栏物品转换成 AnimationState。真正的关节姿态由后面的 Rig 系统统一应用。
pub fn player_animation_controller_system(
    time: Res<Time>,
    actions: Res<PlayerActionState>,
    inventory: Res<InventoryState>,
    mut query: Query<(&Transform, &PlayerGravity, &mut PlayerAnimationState), With<LocalPlayer>>,
) {
    let dt = time.delta_secs().max(0.0001);
    let holding_item = !inventory.hotbar.active_item().is_air();

    for (transform, gravity, mut state) in &mut query {
        let previous = state.previous_position.replace(transform.translation);
        let horizontal_speed = previous.map_or(0.0, |last| {
            Vec2::new(
                transform.translation.x - last.x,
                transform.translation.z - last.z,
            )
            .length()
                / dt
        });

        state.holding_item = holding_item;
        state.locomotion = if !gravity.is_grounded {
            if gravity.velocity_y > 0.0 {
                PlayerLocomotionState::Jump
            } else {
                PlayerLocomotionState::Fall
            }
        } else if horizontal_speed < 0.05 {
            PlayerLocomotionState::Idle
        } else if actions.pressed(PlayerAction::Sprint) {
            PlayerLocomotionState::Run
        } else {
            PlayerLocomotionState::Walk
        };

        let phase_speed = match state.locomotion {
            PlayerLocomotionState::Walk => 7.0,
            PlayerLocomotionState::Run => 10.5,
            _ => 1.5,
        };
        state.movement_phase += dt * phase_speed;

        if state.action_timer > 0.0 {
            state.action_timer = (state.action_timer - dt).max(0.0);
        }

        if actions.just_pressed(PlayerAction::Attack) {
            let action = if holding_item {
                PlayerHandAction::HeldItemAttack
            } else {
                PlayerHandAction::EmptyHandAttack
            };
            state.trigger_action(action, 0.34);
            continue;
        }

        if actions.just_pressed(PlayerAction::Use) {
            let action = if holding_item {
                PlayerHandAction::HeldItemInteract
            } else {
                PlayerHandAction::EmptyHandInteract
            };
            state.trigger_action(action, 0.24);
            continue;
        }

        if !state.has_active_action() {
            state.settle_to_base_hand_action();
        }
    }
}

/// 把统一 AnimationState 应用到真实 PlayerRig。
pub fn apply_player_rig_animation_system(
    state_query: Query<(&PlayerAnimationState, &PlayerRigEntities), With<Player>>,
    mut transform_query: Query<&mut Transform>,
) {
    for (state, rig) in &state_query {
        set_rotation(&mut transform_query, rig.body_joint, Quat::IDENTITY);
        set_rotation(&mut transform_query, rig.head_joint, Quat::IDENTITY);

        apply_leg_pose(state, rig, &mut transform_query);
        apply_arm_pose(state, rig, &mut transform_query);
        apply_held_item_anchor_pose(rig, &mut transform_query);
    }
}

fn apply_leg_pose(
    state: &PlayerAnimationState,
    rig: &PlayerRigEntities,
    transform_query: &mut Query<&mut Transform>,
) {
    let amplitude = match state.locomotion {
        PlayerLocomotionState::Walk => 0.42,
        PlayerLocomotionState::Run => 0.68,
        _ => 0.0,
    };
    let swing = state.movement_phase.sin() * amplitude;

    set_rotation(transform_query, rig.thigh_r, Quat::from_rotation_x(-swing));
    set_rotation(transform_query, rig.thigh_l, Quat::from_rotation_x(swing));
    set_rotation(
        transform_query,
        rig.calf_r,
        Quat::from_rotation_x(swing.max(0.0) * 0.35),
    );
    set_rotation(
        transform_query,
        rig.calf_l,
        Quat::from_rotation_x((-swing).max(0.0) * 0.35),
    );
}

fn apply_arm_pose(
    state: &PlayerAnimationState,
    rig: &PlayerRigEntities,
    transform_query: &mut Query<&mut Transform>,
) {
    let move_amplitude = match state.locomotion {
        PlayerLocomotionState::Walk => 0.22,
        PlayerLocomotionState::Run => 0.36,
        _ => 0.0,
    };
    let arm_swing = state.movement_phase.sin() * move_amplitude;

    match state.hand_action {
        PlayerHandAction::EmptyHandIdle => apply_empty_idle_pose(rig, transform_query, 0.0),
        PlayerHandAction::EmptyHandMove => apply_empty_idle_pose(rig, transform_query, arm_swing),
        PlayerHandAction::EmptyHandAttack => {
            apply_empty_attack_pose(rig, transform_query, state.action_progress())
        }
        PlayerHandAction::EmptyHandInteract => {
            apply_empty_interact_pose(rig, transform_query, state.action_progress())
        }
        PlayerHandAction::EmptyHandReady => apply_empty_ready_pose(rig, transform_query),
        PlayerHandAction::HeldItemIdle => apply_held_idle_pose(rig, transform_query, 0.0),
        PlayerHandAction::HeldItemMove => {
            apply_held_idle_pose(rig, transform_query, arm_swing * 0.35)
        }
        PlayerHandAction::HeldItemAttack => {
            apply_held_attack_pose(rig, transform_query, state.action_progress())
        }
        PlayerHandAction::HeldItemInteract => {
            apply_held_interact_pose(rig, transform_query, state.action_progress())
        }
    }
}

fn apply_empty_idle_pose(
    rig: &PlayerRigEntities,
    transform_query: &mut Query<&mut Transform>,
    swing: f32,
) {
    set_rotation(
        transform_query,
        rig.upper_arm_r,
        arm_rotation(0.16 + swing, -0.07),
    );
    set_rotation(
        transform_query,
        rig.upper_arm_l,
        arm_rotation(0.16 - swing, 0.07),
    );
    set_rotation(
        transform_query,
        rig.forearm_r,
        arm_rotation(0.18 + swing * 0.25, -0.03),
    );
    set_rotation(
        transform_query,
        rig.forearm_l,
        arm_rotation(0.18 - swing * 0.25, 0.03),
    );
    set_rotation(transform_query, rig.hand_r, Quat::from_rotation_x(0.04));
    set_rotation(transform_query, rig.hand_l, Quat::from_rotation_x(0.04));
}

fn apply_empty_attack_pose(
    rig: &PlayerRigEntities,
    transform_query: &mut Query<&mut Transform>,
    progress: f32,
) {
    apply_empty_idle_pose(rig, transform_query, 0.0);
    let punch = (progress * std::f32::consts::PI).sin();
    set_rotation(
        transform_query,
        rig.upper_arm_r,
        arm_rotation(0.24 + punch * 1.12, -0.24),
    );
    set_rotation(
        transform_query,
        rig.forearm_r,
        arm_rotation(0.22 + punch * 0.68, -0.08),
    );
    set_rotation(
        transform_query,
        rig.hand_r,
        Quat::from_rotation_x(punch * 0.22),
    );
}

fn apply_empty_interact_pose(
    rig: &PlayerRigEntities,
    transform_query: &mut Query<&mut Transform>,
    progress: f32,
) {
    apply_empty_idle_pose(rig, transform_query, 0.0);
    let reach = (progress * std::f32::consts::PI).sin();
    set_rotation(
        transform_query,
        rig.upper_arm_r,
        arm_rotation(0.22 + reach * 0.68, -0.18),
    );
    set_rotation(
        transform_query,
        rig.forearm_r,
        arm_rotation(0.2 + reach * 0.42, -0.06),
    );
}

fn apply_empty_ready_pose(rig: &PlayerRigEntities, transform_query: &mut Query<&mut Transform>) {
    set_rotation(transform_query, rig.upper_arm_r, arm_rotation(0.74, -0.24));
    set_rotation(transform_query, rig.upper_arm_l, arm_rotation(0.54, 0.18));
    set_rotation(transform_query, rig.forearm_r, arm_rotation(0.36, -0.08));
    set_rotation(transform_query, rig.forearm_l, arm_rotation(0.28, 0.06));
    set_rotation(transform_query, rig.hand_r, Quat::from_rotation_x(0.08));
    set_rotation(transform_query, rig.hand_l, Quat::from_rotation_x(0.08));
}

fn apply_held_idle_pose(
    rig: &PlayerRigEntities,
    transform_query: &mut Query<&mut Transform>,
    sway: f32,
) {
    set_rotation(
        transform_query,
        rig.upper_arm_r,
        arm_rotation(0.72 + sway, -0.32),
    );
    set_rotation(
        transform_query,
        rig.forearm_r,
        arm_rotation(0.42 + sway * 0.2, -0.10),
    );
    set_rotation(transform_query, rig.hand_r, Quat::from_rotation_x(0.10));

    set_rotation(
        transform_query,
        rig.upper_arm_l,
        arm_rotation(0.14 - sway, 0.07),
    );
    set_rotation(transform_query, rig.forearm_l, arm_rotation(0.18, 0.03));
    set_rotation(transform_query, rig.hand_l, Quat::from_rotation_x(0.04));
}

fn apply_held_attack_pose(
    rig: &PlayerRigEntities,
    transform_query: &mut Query<&mut Transform>,
    progress: f32,
) {
    let swing = (progress * std::f32::consts::PI).sin();
    apply_held_idle_pose(rig, transform_query, 0.0);
    set_rotation(
        transform_query,
        rig.upper_arm_r,
        arm_rotation(0.72 + swing * 0.72, -0.38),
    );
    set_rotation(
        transform_query,
        rig.forearm_r,
        arm_rotation(0.38 + swing * 0.52, -0.12),
    );
}

fn apply_held_interact_pose(
    rig: &PlayerRigEntities,
    transform_query: &mut Query<&mut Transform>,
    progress: f32,
) {
    let reach = (progress * std::f32::consts::PI).sin();
    apply_held_idle_pose(rig, transform_query, 0.0);
    set_rotation(
        transform_query,
        rig.upper_arm_r,
        arm_rotation(0.72 + reach * 0.36, -0.30),
    );
    set_rotation(
        transform_query,
        rig.forearm_r,
        arm_rotation(0.42 + reach * 0.24, -0.08),
    );
}

fn apply_held_item_anchor_pose(
    rig: &PlayerRigEntities,
    transform_query: &mut Query<&mut Transform>,
) {
    if let Ok(mut transform) = transform_query.get_mut(rig.held_item) {
        *transform = held_item_grip_transform();
    }
}

fn arm_rotation(x: f32, z: f32) -> Quat {
    Quat::from_rotation_z(z) * Quat::from_rotation_x(x)
}

fn set_rotation(transform_query: &mut Query<&mut Transform>, entity: Entity, rotation: Quat) {
    if let Ok(mut transform) = transform_query.get_mut(entity) {
        transform.rotation = rotation;
    }
}
