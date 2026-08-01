//! 定义树种、生长节奏和树形参数的数据格式。

use crate::shared::identifier::Identifier;
use serde::{Deserialize, Serialize};

const DEFAULT_SAPLING_DURATION_GAME_MINUTES: u64 = 24 * 60;
const DEFAULT_YOUNG_DURATION_GAME_MINUTES: u64 = 3 * 24 * 60;
const DEFAULT_RETRY_INTERVAL_GAME_MINUTES: u64 = 5;

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
    /// 低频生命周期阶段时长与受阻重试规则。
    pub growth: TreeGrowthDefinition,
    /// 幼树阶段使用的较小树形；缺失时由成熟尺寸确定性派生。
    #[serde(default)]
    pub young_blueprint: Option<TreeBlueprintDefinition>,
    /// 当前树形蓝图使用的尺寸范围。
    pub blueprint: TreeBlueprintDefinition,
}

/// 定义树苗、幼树的阶段时长和空间受阻后的重试间隔。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TreeGrowthDefinition {
    /// 树苗成为幼树前至少经过的游戏分钟数。
    #[serde(default = "default_sapling_duration_game_minutes")]
    pub sapling_duration_game_minutes: u64,
    /// 幼树成为成熟树前至少经过的游戏分钟数。
    #[serde(default = "default_young_duration_game_minutes")]
    pub young_duration_game_minutes: u64,
    /// 区块未加载或空间被占用后延迟多久再次检查。
    #[serde(
        default = "default_retry_interval_game_minutes",
        alias = "attempt_interval_game_minutes"
    )]
    pub retry_interval_game_minutes: u64,
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

const fn default_sapling_duration_game_minutes() -> u64 {
    DEFAULT_SAPLING_DURATION_GAME_MINUTES
}

const fn default_young_duration_game_minutes() -> u64 {
    DEFAULT_YOUNG_DURATION_GAME_MINUTES
}

const fn default_retry_interval_game_minutes() -> u64 {
    DEFAULT_RETRY_INTERVAL_GAME_MINUTES
}

#[cfg(test)]
#[path = "../../../tests/unit/content/vegetation/definition.rs"]
mod tests;
