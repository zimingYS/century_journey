//! 物品分类定义。

use serde::{Deserialize, Serialize};

/// 物品分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemCategory {
    /// 方块类（由方块注册表自动生成）
    Block,
    /// 材料类（矿石、锭、宝石等）
    Material,
    /// 工具类（镐、斧、铲等）
    Tool,
    /// 武器类（剑、弓等）
    Weapon,
    /// 盔甲类（头、胸、腿、脚）
    Armor,
    /// 饰品类（戒指、项链等）
    Accessory,
    /// 消耗品类（食物、药水等）
    #[serde(rename = "consumable")]
    Consumable,
}
