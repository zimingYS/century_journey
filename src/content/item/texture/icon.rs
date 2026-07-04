use crate::shared::identifier::Identifier;
use serde::{Deserialize, Serialize};

/// 物品图标定义
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum IconDefinition {
    /// 方块图标
    /// 使用方块注册表中方块的纹理
    Block(Identifier),
    /// 独立纹理
    /// 使用独立的纹理路径
    Texture(String),
}

impl IconDefinition {
    /// 从方块标识符创建方块图标
    pub fn block(id: impl Into<String>) -> Self {
        IconDefinition::Block(Identifier::parse(&id.into()).unwrap_or_default())
    }

    /// 从路径创建图标
    pub fn texture(path: impl Into<String>) -> Self {
        IconDefinition::Texture(path.into())
    }

    /// 获取用于方块注册表纹理查找的方块标识符
    pub fn as_block_id(&self) -> Option<&Identifier> {
        match self {
            IconDefinition::Block(id) => Some(id),
            IconDefinition::Texture(_) => None,
        }
    }

    /// 获取独立纹理引用的路径
    pub fn texture_path(&self) -> Option<&str> {
        match self {
            IconDefinition::Block(_) => None,
            IconDefinition::Texture(path) => Some(path.as_str()),
        }
    }
}

impl Default for IconDefinition {
    fn default() -> Self {
        IconDefinition::Block(Identifier::new("century_journey", "air"))
    }
}
