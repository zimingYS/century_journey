//! 矿脉定义：把"矿石方块"与"世界生成参数"解耦成独立内容。

use crate::shared::identifier::Identifier;
use serde::{Deserialize, Serialize};

/// 一条矿脉的稳定内容定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OreVeinDefinition {
    /// 矿脉的稳定标识符。
    pub identifier: Identifier,
    /// 面向玩家和开发工具显示的矿脉名称。
    pub display_name: String,
    /// 生成时放置的矿石方块。
    pub block: Identifier,
    /// 检查优先级：数值大者先检查，重叠区域由高优先级矿脉获胜。
    #[serde(default)]
    pub priority: u32,
    /// 世界高度带（含）。
    pub min_y: i32,
    pub max_y: i32,
    /// 3D 噪声阈值：低于阈值出现矿（值越低越稀有）。
    pub threshold: f64,
    /// 噪声世界坐标缩放（越小矿团越大）。
    pub scale: f64,
}

#[cfg(test)]
#[path = "../../../tests/unit/content/ore_vein/definition.rs"]
mod tests;
