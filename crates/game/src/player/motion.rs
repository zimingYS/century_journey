use crate::input::intent::PlayerIntent;
use crate::player::camera::PlayerCamera;
use bevy::prelude::*;

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
