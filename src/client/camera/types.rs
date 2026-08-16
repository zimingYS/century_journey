//! 摄像机数据类型：视角枚举与本地玩家相机组件。

use bevy::prelude::*;

/// 摄像机俯仰上限；预留五度避免与竖直方向重合后出现翻转奇异。
pub const MAX_CAMERA_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 5.0 * std::f32::consts::PI / 180.0;
/// 摄像机俯仰下限。
pub const MIN_CAMERA_PITCH: f32 = -MAX_CAMERA_PITCH;

/// 本地玩家摄像机视角。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CameraPerspective {
    #[default]
    FirstPerson,
    SecondPerson,
    ThirdPerson,
}

impl CameraPerspective {
    /// 按第一、第二、第三人称顺序循环。
    pub const fn next(self) -> Self {
        match self {
            Self::FirstPerson => Self::SecondPerson,
            Self::SecondPerson => Self::ThirdPerson,
            Self::ThirdPerson => Self::FirstPerson,
        }
    }

    /// 返回面向玩家的中文视角名称。
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::FirstPerson => "第一人称",
            Self::SecondPerson => "第二人称",
            Self::ThirdPerson => "第三人称",
        }
    }
}

/// 本地玩家摄像机的灵敏度、俯仰角和观察视角。
#[derive(Component)]
pub struct FpsCamera {
    pub mouse_sensitivity: f32,
    pub pitch: f32,
    pub perspective: CameraPerspective,
}

impl Default for FpsCamera {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.015,
            pitch: 0.0,
            perspective: CameraPerspective::FirstPerson,
        }
    }
}

impl FpsCamera {
    /// 设置经过翻转保护约束的绝对俯仰角。
    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch.clamp(MIN_CAMERA_PITCH, MAX_CAMERA_PITCH);
    }

    /// 在当前俯仰角上累加输入增量。
    pub fn add_pitch(&mut self, delta: f32) {
        self.set_pitch(self.pitch + delta);
    }

    /// 返回当前俯仰角对应的局部旋转。
    pub fn pitch_rotation(&self) -> Quat {
        Quat::from_rotation_x(self.pitch.clamp(MIN_CAMERA_PITCH, MAX_CAMERA_PITCH))
    }

    /// 当前是否使用第一人称。
    pub const fn is_first_person(&self) -> bool {
        matches!(self.perspective, CameraPerspective::FirstPerson)
    }
}
