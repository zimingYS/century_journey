use crate::input::intent::{PlayerIntent, ToggleMovementMode};
use crate::player::camera::PlayerCamera;
use bevy::prelude::*;

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
const SPRINT_MULTIPLIER: f32 = 4.0;

/// 根据玩家输入和当前相机朝向计算飞行速度
fn fly_velocity(camera: &PlayerCamera, intent: &PlayerIntent) -> Vec3 {
    let mut velocity = camera.forward() * intent.move_axis.z
        + camera.right() * intent.move_axis.x
        + Vec3::Y * intent.move_axis.y;

    let speed = if intent.sprint {
        FLY_SPEED * SPRINT_MULTIPLIER
    } else {
        FLY_SPEED
    };

    velocity *= speed;

    velocity
}

/// 应用玩家飞行移动
pub fn apply_fly_motion(
    intent: Res<PlayerIntent>,
    time: Res<Time>,
    mut player: Query<(&PlayerCamera, &mut Transform)>,
) {
    let delta_secs = time.delta_secs();

    for (camera, mut transform) in &mut player {
        transform.translation += fly_velocity(camera, &intent) * delta_secs;
    }
}

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

        let velocity = fly_velocity(&camera, &intent);

        assert!(
            velocity.y.abs() < f32::EPSILON,
            "forward velocity must remain horizontal"
        );
    }
}
