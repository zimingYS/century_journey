use crate::inventory::item::id::ItemId;

/// 物品分类
///
/// 用于创造模式分类标签、合成配方分组、掉落表筛选等。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemCategory {
    /// 方块类（由BlockRegistry自动生成）
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
    Consumable,

    // ── 未来扩展 ──
    // Food,
    // Quest,
    // Magic,
}


/// 物品定义:物品在背包/UI 中的行为（堆叠、分类、显示名）
#[derive(Debug, Clone)]
pub struct ItemDefinition {
    /// 物品唯一标识
    pub id: ItemId,
    /// 显示名称
    pub display_name: String,
    /// 最大堆叠数（默认 64）
    pub max_stack: u32,
    /// 物品分类
    pub category: ItemCategory,
}

impl ItemDefinition {
    /// 从方块自动创建 ItemDefinition
    pub fn from_block(identifier: &str, display_name: &str) -> Self {
        Self {
            id: ItemId::block(identifier),
            display_name: display_name.to_string(),
            max_stack: 64,
            category: ItemCategory::Block,
        }
    }
}
