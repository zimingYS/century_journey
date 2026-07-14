use bevy::prelude::*;

/// 留出 5 度余量，避免视线与竖直方向重合后发生翻转和方向奇异。
pub const MAX_CAMERA_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 5.0 * std::f32::consts::PI / 180.0;
pub const MIN_CAMERA_PITCH: f32 = -MAX_CAMERA_PITCH;

/// 第一人称摄像机组件 — 多个模块共享。
#[derive(Component)]
pub struct FpsCamera {
    pub mouse_sensitivity: f32,
    pub pitch: f32,
    /// 是否为第一人称
    pub is_first_person: bool,
}

impl Default for FpsCamera {
    fn default() -> FpsCamera {
        Self {
            mouse_sensitivity: 0.015,
            pitch: 0.0,
            is_first_person: true,
        }
    }
}

impl FpsCamera {
    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch.clamp(MIN_CAMERA_PITCH, MAX_CAMERA_PITCH);
    }

    pub fn add_pitch(&mut self, delta: f32) {
        self.set_pitch(self.pitch + delta);
    }

    pub fn pitch_rotation(&self) -> Quat {
        Quat::from_rotation_x(self.pitch.clamp(MIN_CAMERA_PITCH, MAX_CAMERA_PITCH))
    }
}
