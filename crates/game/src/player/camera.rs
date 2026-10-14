use crate::input::intent::PlayerIntent;
use crate::player::motion::PlayerMotion;
use bevy::prelude::*;

/// 玩家相机初始世界坐标位置
const INITIAL_POSITION: Vec3 = Vec3::new(60.0, 190.0, 60.0);
/// 初始偏航角(yaw)：控制左右水平旋转，弧度制
const INITIAL_YAW: f32 = -0.785;
/// 初始俯仰角(pitch)：控制上下垂直旋转，弧度制
const INITIAL_PITCH: f32 = -0.5;
/// 鼠标视角灵敏度：鼠标移动像素 × 该值 = 旋转弧度
const LOOK_SENSITIVITY: f32 = 0.002;
/// 俯仰角最大限制(弧度)，防止视角翻转
const PITCH_LIMIT: f32 = 1.55;
/// 眼部相对玩家便宜
const EYE_OFFSET: f32 = 0.8;

/// 玩家相机
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerCamera {
    /// 水平角弧度
    yaw: f32,
    /// 俯仰角弧度（±89°）
    pitch: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            yaw: INITIAL_YAW,
            pitch: INITIAL_PITCH,
        }
    }
}

impl PlayerCamera {
    /// 应用本帧鼠标位移
    pub fn apply_look_delta(&mut self, delta: Vec2) {
        self.yaw -= delta.x * LOOK_SENSITIVITY;
        self.pitch -= delta.y * LOOK_SENSITIVITY;
        self.pitch = self.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    /// 获取当前完整相机旋转
    #[inline]
    pub fn rotation(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }

    /// 获取仅包含 yaw 的前方向
    #[inline]
    pub fn forward(&self) -> Vec3 {
        Quat::from_rotation_y(self.yaw) * -Vec3::Z
    }

    /// 获取仅包含 yaw 的右方向
    #[inline]
    pub fn right(&self) -> Vec3 {
        Quat::from_rotation_y(self.yaw) * Vec3::X
    }

    #[cfg(test)]
    pub(crate) fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch;
    }
}

/// 创建玩家第一人称相机
pub fn spawn_player(mut commands: Commands) {
    let camera = PlayerCamera::default();

    commands
        .spawn((
            camera,
            PlayerMotion::default(),
            Visibility::Visible,
            Transform::from_translation(INITIAL_POSITION).with_rotation(camera.rotation()),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, EYE_OFFSET, 0.0),
            ));
        });
}

/// 根据玩家输入更新相机视角
pub fn apply_look(
    intent: Res<PlayerIntent>,
    mut camera: Query<(&mut PlayerCamera, &mut Transform)>,
) {
    for (mut player_camera, mut transform) in &mut camera {
        player_camera.apply_look_delta(intent.look_delta);
        transform.rotation = player_camera.rotation();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_clamps_to_lower_limit() {
        let mut camera = PlayerCamera::default();
        camera.apply_look_delta(Vec2::new(0.0, 100_000.0));
        assert_eq!(camera.pitch, -PITCH_LIMIT);
    }

    #[test]
    fn pitch_clamps_to_upper_limit() {
        let mut camera = PlayerCamera::default();
        camera.apply_look_delta(Vec2::new(0.0, -100_000.0));
        assert_eq!(camera.pitch, PITCH_LIMIT);
    }
}
