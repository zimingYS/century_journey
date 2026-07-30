//! 定义树种、生长节奏和树形参数的数据格式。

use crate::shared::identifier::Identifier;
use serde::{Deserialize, Serialize};

/// 描述一种树木从树苗到体素结构所需的稳定内容数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeSpeciesDefinition {
    /// 树种的稳定标识符。
    pub identifier: Identifier,
    /// 面向玩家和开发工具显示的树种名称。
    pub display_name: String,
    /// 代表该树种幼苗的方块。
    pub sapling_block: Identifier,
    /// 生长后使用的树干方块。
    pub trunk_block: Identifier,
    /// 生长后使用的树叶方块。
    pub leaves_block: Identifier,
    /// 低频生长尝试规则。
    pub growth: TreeGrowthDefinition,
    /// 当前树形蓝图使用的尺寸范围。
    pub blueprint: TreeBlueprintDefinition,
}

/// 定义树苗参与权威生长判定的时间间隔和成功概率。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TreeGrowthDefinition {
    /// 两次生长机会之间相隔的游戏分钟数。
    pub attempt_interval_game_minutes: u64,
    /// 每次满足环境和空间约束后的成功概率。
    pub chance_per_attempt: f32,
}

/// 定义小树蓝图的树干高度和树冠半径范围。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TreeBlueprintDefinition {
    /// 树干高度的闭区间。
    pub trunk_height: TreeSizeRange,
    /// 球形树冠半径的闭区间。
    pub crown_radius: TreeSizeRange,
}

/// 用紧凑整数表达树形尺寸的闭区间。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TreeSizeRange {
    /// 最小尺寸，包含在范围内。
    pub min: u8,
    /// 最大尺寸，包含在范围内。
    pub max: u8,
}

#[cfg(test)]
#[path = "../../../tests/unit/content/vegetation/definition.rs"]
mod tests;
