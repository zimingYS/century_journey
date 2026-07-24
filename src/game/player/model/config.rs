use crate::game::player::model::components::PlayerPart;
use bevy::prelude::*;

/// 玩家模型设置
#[derive(Resource, Debug, Clone)]
pub struct PlayerModelConfig {
    /// 全局统一缩放系数
    pub base_scale: f32,
}

impl Default for PlayerModelConfig {
    fn default() -> Self {
        // 标准大小
        Self { base_scale: 1.0 }
    }
}

/// 身体每部分尺寸
#[derive(Debug, Clone, Copy)]
pub struct PartDimensions {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl PlayerModelConfig {
    /// 部件未缩放的半尺寸
    pub fn half_dims(part: PlayerPart) -> Vec3 {
        match part {
            PlayerPart::Head => Vec3::new(0.21, 0.215, 0.20),
            PlayerPart::Body => Vec3::new(0.245, 0.34, 0.14),
            PlayerPart::UpperArmL(_) => Vec3::new(0.105, 0.18, 0.105),
            PlayerPart::ForearmL(_) => Vec3::new(0.095, 0.165, 0.095),
            PlayerPart::HandL(_) => Vec3::new(0.10, 0.07, 0.105),
            PlayerPart::ThighL(_) => Vec3::new(0.12, 0.20, 0.125),
            PlayerPart::CalfL(_) => Vec3::new(0.105, 0.16, 0.11),
            PlayerPart::FootL(_) => Vec3::new(0.12, 0.075, 0.18),
        }
    }

    /// Mesh 缩放
    pub fn scale(&self, part: PlayerPart) -> Vec3 {
        let h = Self::half_dims(part);
        Vec3::new(h.x * 2.0, h.y * 2.0, h.z * 2.0) * self.base_scale
    }

    /// 部件颜色
    pub fn color(part: PlayerPart) -> Color {
        match part {
            PlayerPart::Head => Color::srgb(0.88, 0.68, 0.52),
            PlayerPart::Body | PlayerPart::UpperArmL(_) => Color::srgb(0.07, 0.28, 0.31),
            PlayerPart::ForearmL(_) | PlayerPart::HandL(_) => Color::srgb(0.88, 0.68, 0.52),
            PlayerPart::ThighL(_) => Color::srgb(0.12, 0.17, 0.26),
            PlayerPart::CalfL(_) => Color::srgb(0.08, 0.12, 0.19),
            PlayerPart::FootL(_) => Color::srgb(0.035, 0.045, 0.06),
        }
    }

    /// 父关节对子关节的偏移
    pub fn joint_offset(child: PlayerPart) -> Vec3 {
        match child {
            // 直接挂在Root下的部分
            PlayerPart::Body => Vec3::new(0.0, 0.20, 0.0),
            PlayerPart::Head => Vec3::new(0.0, 0.77, 0.0),
            PlayerPart::UpperArmL(false) => Vec3::new(-0.35, 0.50, 0.0),
            PlayerPart::UpperArmL(true) => Vec3::new(0.35, 0.50, 0.0),
            PlayerPart::ThighL(false) => Vec3::new(-0.12, -0.10, 0.0),
            PlayerPart::ThighL(true) => Vec3::new(0.12, -0.10, 0.0),
            // 子关节
            PlayerPart::ForearmL(_) => Vec3::new(0.0, -0.36, 0.0),
            PlayerPart::HandL(_) => Vec3::new(0.0, -0.33, 0.0),
            PlayerPart::CalfL(_) => Vec3::new(0.0, -0.40, 0.0),
            PlayerPart::FootL(_) => Vec3::new(0.0, -0.25, 0.0),
        }
    }

    /// 关节到Mesh中心的偏移
    pub fn mesh_offset(part: PlayerPart) -> Vec3 {
        match part {
            PlayerPart::Head | PlayerPart::Body => Vec3::ZERO,
            PlayerPart::UpperArmL(_)
            | PlayerPart::ForearmL(_)
            | PlayerPart::ThighL(_)
            | PlayerPart::CalfL(_) => {
                let h = Self::half_dims(part);
                Vec3::new(0.0, -h.y, 0.0)
            }
            PlayerPart::HandL(_) => {
                let h = Self::half_dims(part);
                Vec3::new(0.0, -h.y, 0.0)
            }
            PlayerPart::FootL(_) => {
                let h = Self::half_dims(part);
                Vec3::new(0.0, -h.y, -0.07)
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/player/model/config.rs"]
mod tests;
