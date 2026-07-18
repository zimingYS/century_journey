use bevy::prelude::*;

/// 留出 5 度余量，避免视线与竖直方向重合后发生翻转和方向奇异。
pub const MAX_CAMERA_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 5.0 * std::f32::consts::PI / 180.0;
pub const MIN_CAMERA_PITCH: f32 = -MAX_CAMERA_PITCH;

/// 玩家摄像机视角。
///
/// 第二人称是位于玩家前方、面向玩家的观察视角；第三人称位于玩家背后。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CameraPerspective {
    #[default]
    FirstPerson,
    SecondPerson,
    ThirdPerson,
}

impl CameraPerspective {
    pub const fn next(self) -> Self {
        match self {
            Self::FirstPerson => Self::SecondPerson,
            Self::SecondPerson => Self::ThirdPerson,
            Self::ThirdPerson => Self::FirstPerson,
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::FirstPerson => "第一人称",
            Self::SecondPerson => "第二人称",
            Self::ThirdPerson => "第三人称",
        }
    }
}

/// 玩家摄像机组件 — 多个模块共享。
#[derive(Component)]
pub struct FpsCamera {
    pub mouse_sensitivity: f32,
    pub pitch: f32,
    pub perspective: CameraPerspective,
}

impl Default for FpsCamera {
    fn default() -> FpsCamera {
        Self {
            mouse_sensitivity: 0.015,
            pitch: 0.0,
            perspective: CameraPerspective::FirstPerson,
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

    pub const fn is_first_person(&self) -> bool {
        matches!(self.perspective, CameraPerspective::FirstPerson)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perspective_cycles_first_second_third() {
        let first = CameraPerspective::FirstPerson;
        assert_eq!(first.next(), CameraPerspective::SecondPerson);
        assert_eq!(first.next().next(), CameraPerspective::ThirdPerson);
        assert_eq!(first.next().next().next(), first);
    }
}
