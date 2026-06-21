use bevy::prelude::*;

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
