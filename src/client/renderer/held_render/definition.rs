use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 手持渲染统一描述符
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HeldRenderDefinition {
    /// 空手
    #[serde(rename = "empty")]
    Empty,

    /// 方块立方体
    /// 由方块纹理渲染出立方体
    #[serde(rename = "block")]
    Block,

    /// 平面物品
    /// 根据物品图标生成带厚度的模型
    #[serde(rename = "flat_item")]
    FlatItem {
        /// 挤出厚度
        #[serde(default = "default_thickness")]
        thickness: f32,
    },

    /// 外部3D模型
    /// 加载GLB/GLTF文件
    #[serde(rename = "model")]
    Model {
        /// 模型文件路径 (相对于 assets/)
        path: String,
    },
}

/// 默认厚度设置
fn default_thickness() -> f32 { 0.1 }

impl Default for HeldRenderDefinition {
    fn default() -> Self {
        HeldRenderDefinition::Empty
    }
}


/// 手持物品的完整渲染配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeldItemConfig {
    /// 渲染方式
    #[serde(default)]
    pub render: HeldRenderDefinition,

    /// 第一人称位置
    #[serde(default = "default_fp_translation")]
    pub first_person_translation: [f32; 3],

    /// 第一人称旋转
    #[serde(default)]
    pub first_person_rotation: [f32; 3],

    /// 统一缩放
    #[serde(default = "default_fp_scale")]
    pub first_person_scale: f32,

    /// 动画配置
    #[serde(default)]
    pub animations: AnimationConfig,
}

/// 默认第一人称位置
fn default_fp_translation() -> [f32; 3] { [0.30, -0.05, -0.40] }
/// 默认第一人称缩放
fn default_fp_scale() -> f32 { 0.55 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationConfig {
    /// 支持挥动动画
    #[serde(default)]
    pub swing: bool,

    /// 支持食用动画
    #[serde(default)]
    pub eat: bool,

    /// 支持使用/交互动画
    #[serde(default)]
    pub use_anim: bool,

    /// 支持望远镜/瞄准动画
    #[serde(default)]
    pub spyglass: bool,
}

impl Default for HeldItemConfig {
    fn default() -> Self {
        Self {
            render: HeldRenderDefinition::Empty,
            first_person_translation: default_fp_translation(),
            first_person_rotation: [0.0; 3],
            first_person_scale: default_fp_scale(),
            animations: AnimationConfig::default(),
        }
    }
}

impl HeldItemConfig {
    pub fn to_transform(&self) -> Transform {
        Transform {
            translation: Vec3::from(self.first_person_translation),
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                self.first_person_rotation[0].to_radians(),
                self.first_person_rotation[1].to_radians(),
                self.first_person_rotation[2].to_radians(),
            ),
            scale: Vec3::splat(self.first_person_scale),
        }
    }

    /// 默认方块手持配置
    pub fn default_block() -> Self {
        Self {
            render: HeldRenderDefinition::Block,
            first_person_translation: [0.30, -0.10, -0.50],
            first_person_rotation: [-15.0, 25.0, 0.0],
            first_person_scale: 0.2,
            animations: AnimationConfig { swing: true, ..default() },
        }
    }

    /// 默认工具手持配置
    pub fn default_tool(thickness: f32) -> Self {
        Self {
            render: HeldRenderDefinition::FlatItem { thickness },
            first_person_translation: [0.30, -0.10, -0.50],
            first_person_rotation: [0.0, -60.0, 60.0],
            first_person_scale: 0.50,
            animations: AnimationConfig { swing: true, ..default() },
        }
    }

    /// 默认普通物品手持配置
    pub fn default_flat(thickness: f32) -> Self {
        Self {
            render: HeldRenderDefinition::FlatItem { thickness },
            first_person_translation: [0.30, -0.10, -0.50],
            first_person_rotation: [0.0, -60.0, 60.0],
            first_person_scale: 0.50,
            animations: AnimationConfig::default(),
        }
    }

    /// 模型物品手持配置
    pub fn default_model(path: &str) -> Self {
        Self {
            render: HeldRenderDefinition::Model { path: path.to_string() },
            first_person_translation: [0.25, -0.10, -0.40],
            first_person_rotation: [0.0, -60.0, 60.0],
            first_person_scale: 0.60,
            animations: AnimationConfig::default(),
        }
    }
}
