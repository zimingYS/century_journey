//! 物品定义的数据结构及其便捷查询方法。

use serde::{Deserialize, Serialize};

use crate::content::item::definition::category::ItemCategory;
use crate::content::item::definition::nutrition::{DrinkData, FoodData};
use crate::content::item::definition::presentation::{AnimationConfig, HeldRenderDefinition};
use crate::content::item::definition::tool::ToolData;
use crate::content::item::texture::icon::IconDefinition;
use crate::shared::identifier::Identifier;

/// 物品定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDefinition {
    /// 唯一标识符
    pub identifier: Identifier,
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

    /// 可选的物品模型 ID；未配置时会根据分类、图标和旧 held_renderer 自动推导 fallback 模型。
    #[serde(default)]
    pub model: Option<Identifier>,

    /// 可放置的方块 ID (仅 Block 物品)
    #[serde(default)]
    pub placeable_block: Option<Identifier>,

    /// 工具数据 (仅 Tool 物品)
    #[serde(default)]
    pub tool: Option<ToolData>,

    /// 食物数据；存在时该物品可以通过“使用”恢复饥饿值。
    #[serde(default)]
    pub food: Option<FoodData>,

    /// 饮品数据；存在时该物品可以通过"使用"恢复口渴值。
    #[serde(default)]
    pub drink: Option<DrinkData>,

    /// 手持渲染配置 (用于第一人称 ViewModel)
    #[serde(default)]
    pub held_renderer: HeldRenderDefinition,

    /// 动画配置
    #[serde(default)]
    pub animations: AnimationConfig,
}

fn default_max_stack() -> u32 {
    64
}

impl ItemDefinition {
    /// 从方块属性自动创建 Block Item (保留兼容 bridge 系统)
    /// 此部分后续应迁移到别的模块
    pub fn from_block(identifier: &Identifier, display_name: &str) -> Self {
        Self {
            identifier: identifier.clone(),
            display_name: display_name.to_string(),
            category: ItemCategory::Block,
            max_stack: 64,
            tags: Vec::new(),
            icon: IconDefinition::block(identifier.to_string()),
            model: None,
            placeable_block: Some(identifier.clone()),
            tool: None,
            food: None,
            drink: None,
            held_renderer: HeldRenderDefinition::Block,
            animations: AnimationConfig::default(),
        }
    }

    /// 返回该物品名称的本地化键（`item.<命名空间>.<路径>`）。
    ///
    /// 键由标识符推导，语言文件缺失该键时调用方应回退到 `display_name`。
    pub fn name_key(&self) -> String {
        format!(
            "item.{}.{}",
            self.identifier.namespace(),
            self.identifier.path()
        )
    }

    /// 是否为工具
    pub fn is_tool(&self) -> bool {
        self.tool.is_some()
    }

    /// 获取工具数据引用
    pub fn tool_data(&self) -> Option<&ToolData> {
        self.tool.as_ref()
    }

    /// 获取食物属性。
    pub fn food_data(&self) -> Option<&FoodData> {
        self.food.as_ref()
    }

    /// 获取饮品属性。
    pub fn drink_data(&self) -> Option<&DrinkData> {
        self.drink.as_ref()
    }

    /// 是否为可放置的方块
    pub fn is_placeable(&self) -> bool {
        self.placeable_block.is_some()
    }

    /// 获取用于渲染的纹理标识符
    pub fn texture_key(&self) -> Option<&Identifier> {
        match &self.icon {
            IconDefinition::Block(id) => Some(id),
            IconDefinition::Texture(_) => None,
        }
    }
}
