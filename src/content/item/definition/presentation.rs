//! 定义物品数据中与展示方式有关的兼容配置。
//!
//! 新内容优先使用 item model；这里保留的字段只负责旧定义迁移和动画能力声明，
//! 不持有 Mesh、Material 或 Transform 等客户端运行时对象。

use serde::{Deserialize, Serialize};

/// 旧物品定义使用的手持几何来源。
///
/// 未显式指定 item model 时，Content 会据此推导等价的模型定义。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HeldRenderDefinition {
    /// 不指定几何来源，由物品分类和图标推导。
    #[serde(rename = "empty")]
    #[default]
    Empty,
    /// 使用对应方块的立方体模型。
    #[serde(rename = "block")]
    Block,
    /// 从二维贴图生成具有指定厚度的挤出模型。
    #[serde(rename = "flat_item")]
    FlatItem {
        /// 挤出模型沿深度方向的厚度。
        #[serde(default = "default_thickness")]
        thickness: f32,
    },
    /// 使用指定路径的自定义模型。
    #[serde(rename = "model")]
    Model {
        /// 相对于内容根目录的模型路径。
        path: String,
    },
}

/// 旧物品定义可请求的第一人称动画能力。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationConfig {
    /// 是否允许挥动反馈。
    #[serde(default)]
    pub swing: bool,
    /// 是否允许进食动画。
    #[serde(default)]
    pub eat: bool,
    /// 是否允许通用使用动画。
    #[serde(default)]
    pub use_anim: bool,
    /// 是否允许望远镜式观察动画。
    #[serde(default)]
    pub spyglass: bool,
}

fn default_thickness() -> f32 {
    0.05
}
