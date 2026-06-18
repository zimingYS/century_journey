use serde::{Deserialize, Serialize};
use crate::inventory::item::icon::IconDefinition;
use crate::inventory::item::id::ItemId;
use crate::inventory::item::tool::ToolData;
use crate::rendering::held_render::{HeldRenderDefinition, AnimationConfig};

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

/// 物品定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDefinition {
    /// 唯一标识符
    pub identifier: String,
    /// 显示名称
    pub display_name: String,
    /// 物品分类
    pub category: ItemCategory,

    /// 最大堆叠数（默认 64）
    #[serde(default = "default_max_stack")]
    pub max_stack: u32,

    /// 标签列表
    #[serde(default)]
    pub tags: Vec<String>,

    /// 图标定义
    #[serde(default)]
    pub icon: IconDefinition,

    /// 可放置的方块 ID (仅 Block 物品)
    #[serde(default)]
    pub placeable_block: Option<String>,

    /// 工具数据 (仅 Tool 物品)
    #[serde(default)]
    pub tool: Option<ToolData>,

    /// 手持渲染配置 (用于第一人称 ViewModel)
    #[serde(default)]
    pub held_render: HeldRenderDefinition,

    /// 动画配置
    #[serde(default)]
    pub animations: AnimationConfig,

    /// 运行时 ItemId — 不参与 serde
    #[serde(skip, default = "ItemId::air")]
    pub id: ItemId,
}

fn default_max_stack() -> u32 { 64 }

impl ItemDefinition {
    /// 从方块属性自动创建 Block Item (保留兼容 bridge 系统)
    pub fn from_block(identifier: &str, display_name: &str) -> Self {
        Self {
            identifier: identifier.to_string(),
            id: ItemId::block(identifier),
            display_name: display_name.to_string(),
            category: ItemCategory::Block,
            max_stack: 64,
            tags: Vec::new(),
            icon: IconDefinition::block(identifier),
            placeable_block: Some(identifier.to_string()),
            tool: None,
            held_render: HeldRenderDefinition::Block,
            animations: AnimationConfig::default(),
        }
    }

    /// 加载时: 从 identifier + category 自动赋值 id
    pub fn finalize_id(&mut self) {
        self.id = match self.category {
            ItemCategory::Block => ItemId::block(&self.identifier),
            _ => ItemId::item(&self.identifier),
        };
    }

    /// 是否为工具
    pub fn is_tool(&self) -> bool {
        self.tool.is_some()
    }

    /// 获取工具数据引用
    pub fn tool_data(&self) -> Option<&ToolData> {
        self.tool.as_ref()
    }

    /// 是否为可放置的方块
    pub fn is_placeable(&self) -> bool {
        self.placeable_block.is_some()
    }

    /// 获取用于渲染的纹理标识符
    pub fn texture_key(&self) -> Option<&str> {
        match &self.icon {
            IconDefinition::Block(id) => Some(id.as_str()),
            IconDefinition::Texture(path) => Some(path.as_str()),
        }
    }
}
