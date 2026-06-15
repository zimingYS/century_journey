use serde::{Deserialize, Serialize};

/// 工具类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    /// 镐
    Pickaxe,
    /// 斧
    Axe,
    /// 铲
    Shovel,
    /// 锄
    Hoe,
}

/// 工具等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolTier{
    /// 木质
    Wood = 0,
    /// 石质
    Stone = 1,
    /// 铁质
    Iron = 2,
    /// 钻石
    Diamond = 3,
}

/// 工具属性数据
/// 挂载在 ItemDefinition.tool 上，描述工具的性能参数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolData {
    /// 工具类型
    pub tool_type: ToolType,
    /// 等级
    pub tier: ToolTier,
    /// 最大耐久度 (能破坏多少个方块)
    pub max_durability: u32,
    /// 基础挖掘效率 (倍率，1.0为空手速度)
    pub efficiency: f32,
}

impl ToolData {
    /// 创建工具数据
    pub fn new(tool_type: ToolType, tier: ToolTier, max_durability: u32, efficiency: f32) -> Self {
        Self { tool_type, tier, max_durability, efficiency }
    }
}
