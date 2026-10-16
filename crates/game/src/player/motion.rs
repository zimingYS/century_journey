use crate::input::intent::{PlayerIntent, ToggleMovementMode};
use crate::player::camera::PlayerCamera;
use crate::player::collision::move_with_collision;
use bevy::prelude::*;
use cj_runtime::world_data::WorldData;

/// 当前移动模式
#[derive(Debug, Clone, Copy, Default)]
pub enum MovementMode {
    Walk,
    #[default]
    Fly,
}

/// 移动属性组件
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PlayerMotion {
    /// 移动速度
    pub velocity: Vec3,
    /// 是否着地
    pub grounded: bool,
    /// 移动模式
    pub mode: MovementMode,
}

/// 飞行速度
const FLY_SPEED: f32 = 20.0;
/// 冲刺速度倍率
const FLY_SPRINT_MULTIPLIER: f32 = 4.0;
/// 重力加速度
const GRAVITY: f32 = -30.0;
/// 行走速度
const WALK_SPEED: f32 = 5.0;
/// 冲刺倍率
const WALK_SPRINT_MULTIPLIER: f32 = 2.0;
/// 跳跃高度
const JUMP_SPEED: f32 = 10.0;

/// 切换移动模式
pub fn toggle_movement_mode(
    mut messages: MessageReader<ToggleMovementMode>,
    mut player: Query<&mut PlayerMotion>,
) {
    // 只要收到任意一次切换请求就取反
    // 一帧内收到多条仅处理一次
    if messages.read().next().is_none() {
        return;
    }

    for mut motion in &mut player {
        motion.mode = match motion.mode {
            MovementMode::Walk => MovementMode::Fly,
            MovementMode::Fly => MovementMode::Walk,
        };
        // 切换时清掉竖直速度
        // 避免切到Walk后被残留的上升速度弹飞
        motion.velocity.y = 0.0;
    }
}

/// 计算速度
pub fn desired_velocity(camera: &PlayerCamera, intent: &PlayerIntent, mode: MovementMode) -> Vec3 {
    // 水平基准
    let mut dir = camera.forward() * intent.move_axis.z + camera.right() * intent.move_axis.x;
    if matches!(mode, MovementMode::Fly) {
        dir.y += intent.move_axis.y;
    }

    // 计算对应冲刺速度
    let (base, sprint_mult) = match mode {
        MovementMode::Fly => (FLY_SPEED, FLY_SPRINT_MULTIPLIER),
        MovementMode::Walk => (WALK_SPEED, WALK_SPRINT_MULTIPLIER),
    };
    let speed = if intent.sprint {
        base * sprint_mult
    } else {
        base
    };

    dir * speed
}

/// 玩家移动应用
pub fn apply_player_motion(
    intent: Res<PlayerIntent>,
    time: Res<Time>,
    world: Res<WorldData>,
    mut player: Query<(&PlayerCamera, &mut PlayerMotion, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (camera, mut motion, mut transform) in &mut player {
        let mode = motion.mode;
        let desired = desired_velocity(camera, &intent, mode);

        match mode {
            MovementMode::Walk => {
                // 水平：每帧重算，不累积
                motion.velocity.x = desired.x;
                motion.velocity.z = desired.z;

                // 跳跃
                if intent.jump && motion.grounded {
                    motion.velocity.y = JUMP_SPEED;
                }
                motion.velocity.y += GRAVITY * dt;

                let (new_pos, grounded) =
                    move_with_collision(transform.translation, motion.velocity, dt, &world);

                if grounded {
                    motion.velocity.y = 0.0;
                }
                motion.grounded = grounded;
                transform.translation = new_pos;
            }
            MovementMode::Fly => {
                // Fly: 直接用 desired 当速度，无重力无碰撞
                motion.velocity = desired;
                transform.translation += motion.velocity * dt;
                motion.grounded = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_motion_does_not_use_pitch() {
        let mut camera = PlayerCamera::default();

        camera.set_pitch(-1.55);

        let intent = PlayerIntent {
            move_axis: Vec3::Z,
            ..Default::default()
        };

        let velocity = desired_velocity(&camera, &intent, MovementMode::Fly);

        assert!(
            velocity.y.abs() < f32::EPSILON,
            "forward velocity must remain horizontal"
        );
    }

    #[test]
    fn walk_mode_velocity_has_no_vertical_component() {
        let mut camera = PlayerCamera::default();
        camera.set_pitch(-1.55);

        let intent = PlayerIntent {
            move_axis: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        };

        let velocity = desired_velocity(&camera, &intent, MovementMode::Walk);

        assert_eq!(
            velocity.y, 0.0,
            "walk mode must not produce vertical velocity"
        );
    }
}
