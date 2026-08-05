//! 定义云层表现参数的数据格式。

use serde::{Deserialize, Serialize};

use crate::shared::identifier::Identifier;

/// 云层定义集合。
///
/// 一份定义包含若干层云和可选的近景云片配置，全部字段为表现参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudDefinition {
    /// 云场的稳定内容标识，供世界或天气配置选择云场。
    pub identifier: Identifier,
    /// 云纹理噪声密度阈值，值越大云覆盖越稀疏。
    pub density: f32,
    /// 云纹理生成的固定种子，保证每次启动云形一致。
    #[serde(default = "default_cloud_seed")]
    pub seed: u32,
    /// 云层列表，按数组顺序从低到高排列。
    pub layers: Vec<CloudLayerDefinition>,
    /// 近景 billboard 云片配置。
    #[serde(default)]
    pub patches: CloudPatchDefinition,
}

/// 单层云的渲染参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudLayerDefinition {
    /// 云层世界高度。
    pub height: f32,
    /// 云层平面世界尺寸（边长），作为纹理重复周期。
    pub size: f32,
    /// 漂移速度（世界单位/秒），方向由风向决定。
    pub speed: f32,
    /// 水平风向向量，运行时会归一化；允许使用负坐标表达反向风。
    #[serde(default = "default_wind_direction")]
    pub wind_direction: [f32; 2],
    /// 白天色调 (R, G, B)，范围 0-1。
    pub tint_day: [f32; 3],
    /// 夜晚色调 (R, G, B)，范围 0-1。
    pub tint_night: [f32; 3],
    /// 黄昏叠加色调 (R, G, B)，范围 0-1。
    pub tint_sunset: [f32; 3],
    /// 不透明度 0-1。
    pub opacity: f32,
}

/// 近景 billboard 云片配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CloudPatchDefinition {
    /// 是否启用云片。
    pub enabled: bool,
    /// 云片数量。
    pub count: u32,
    /// 云片环绕相机的半径。
    pub spawn_radius: f32,
    /// 云片最小尺寸。
    pub scale_min: f32,
    /// 云片最大尺寸。
    pub scale_max: f32,
    /// 云片不透明度。
    pub opacity: f32,
}

impl Default for CloudPatchDefinition {
    fn default() -> Self {
        Self {
            enabled: false,
            count: 10,
            spawn_radius: 120.0,
            scale_min: 8.0,
            scale_max: 18.0,
            opacity: 0.35,
        }
    }
}

/// 云纹理默认固定种子。
fn default_cloud_seed() -> u32 {
    20260803
}

/// 兼容旧云定义的默认水平风向。
fn default_wind_direction() -> [f32; 2] {
    [1.0, 0.0]
}

#[cfg(test)]
#[path = "../../../tests/unit/content/cloud/definition.rs"]
mod tests;
