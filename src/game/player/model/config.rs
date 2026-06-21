use bevy::prelude::*;
use crate::game::player::model::components::PlayerPart;

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
    pub z: f32
}

impl PlayerModelConfig {
    /// 部件未缩放的半尺寸
    pub fn half_dims(part: PlayerPart) -> Vec3 {
        match part {
            PlayerPart::Head       => Vec3::new(0.25, 0.25, 0.25),
            PlayerPart::Body       => Vec3::new(0.25, 0.375, 0.125),
            PlayerPart::UpperArmL(_) | PlayerPart::ForearmL(_) => Vec3::new(0.125, 0.1875, 0.125),
            PlayerPart::HandL(_)   => Vec3::new(0.125, 0.0625, 0.125),
            PlayerPart::ThighL(_) | PlayerPart::CalfL(_)  => Vec3::new(0.125, 0.1875, 0.125),
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
            PlayerPart::Head => Color::srgb(0.90, 0.75, 0.60),
            PlayerPart::Body => Color::srgb(0.30, 0.50, 0.80),
            PlayerPart::UpperArmL(r) if r => Color::srgb(0.90, 0.75, 0.60),
            PlayerPart::UpperArmL(_)   => Color::srgb(0.95, 0.80, 0.65),
            PlayerPart::ForearmL(r) if r => Color::srgb(0.90, 0.75, 0.60),
            PlayerPart::ForearmL(_)    => Color::srgb(0.95, 0.80, 0.65),
            PlayerPart::HandL(r) if r  => Color::srgb(0.90, 0.75, 0.60),
            PlayerPart::HandL(_)       => Color::srgb(0.95, 0.80, 0.65),
            PlayerPart::ThighL(_) | PlayerPart::CalfL(_) => Color::srgb(0.25, 0.35, 0.60),
        }
    }

    /// 父关节对子关节的偏移
    pub fn joint_offset(child: PlayerPart) -> Vec3 {
        match child {
            // 直接挂在Root下的部分
            PlayerPart::Body => Vec3::new(0.0, 0.225, 0.0),
            PlayerPart::Head => Vec3::new(0.0, 0.8, 0.0),
            PlayerPart::UpperArmL(false) => Vec3::new(-0.375, 0.55, 0.0),
            PlayerPart::UpperArmL(true)  => Vec3::new( 0.375, 0.55, 0.0),
            PlayerPart::ThighL(false)    => Vec3::new(-0.125, -0.15, 0.0),
            PlayerPart::ThighL(true)     => Vec3::new( 0.125, -0.15, 0.0),
            // 子关节
            PlayerPart::ForearmL(_) => Vec3::new(0.0, -0.375, 0.0),
            PlayerPart::HandL(_)    => Vec3::new(0.0, -0.25, 0.0),
            PlayerPart::CalfL(_)    => Vec3::new(0.0, -0.375, 0.0),
        }
    }

    /// 关节到Mesh中心的偏移
    pub fn mesh_offset(part: PlayerPart) -> Vec3 {
        match part {
            PlayerPart::Head | PlayerPart::Body => Vec3::ZERO,
            PlayerPart::UpperArmL(_) | PlayerPart::ForearmL(_)
            | PlayerPart::ThighL(_) | PlayerPart::CalfL(_) => {
                let h = Self::half_dims(part);
                Vec3::new(0.0, -h.y, 0.0)
            }
            PlayerPart::HandL(_) => {
                let h = Self::half_dims(part);
                Vec3::new(0.0, -h.y, 0.0)
            }
        }
    }
}
