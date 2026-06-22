use serde::{Deserialize, Serialize};
use std::fmt;

/// 物品唯一标识符
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemId {
    /// 方块类型
    Block(String),
    /// 物品类型
    Item(String),
}

impl ItemId {
    /// 从方块标识符创建
    pub fn block(id: impl Into<String>) -> Self {
        Self::Block(id.into())
    }

    /// 空气方块 (空槽位)
    pub fn air() -> Self {
        Self::Block("century_journey:air".to_string())
    }

    /// 从纯物品标识符创建
    pub fn item(id: impl Into<String>) -> Self {
        ItemId::Item(id.into())
    }

    /// 是否为空气 (空槽位)
    pub fn is_air(&self) -> bool {
        match self {
            ItemId::Block(id) => id == "century_journey:air",
            ItemId::Item(_) => false,
        }
    }

    /// 是否为可放置的方块类型
    pub fn is_block(&self) -> bool {
        matches!(self, ItemId::Block(_))
    }

    /// 是否为纯物品类型
    pub fn is_pure_item(&self) -> bool {
        matches!(self, ItemId::Item(_))
    }

    /// 获取通用标识符引用（Block 和 Item 都适用）
    pub fn as_identifier(&self) -> Option<&str> {
        match self {
            ItemId::Block(id) | ItemId::Item(id) => Some(id.as_str()),
        }
    }

    /// 获取方块标识符引用，非Block变体返回None
    pub fn as_block_id(&self) -> Option<&str> {
        match self {
            ItemId::Block(id) => Some(id),
            ItemId::Item(_) => None,
        }
    }

    /// 获取物品标识符引用
    pub fn as_item_id(&self) -> Option<&str> {
        match self {
            ItemId::Block(_) => None,
            ItemId::Item(id) => Some(id),
        }
    }

    /// 转为可读名称
    pub fn display_name(&self) -> &str {
        match self {
            ItemId::Block(id) | ItemId::Item(id) => {
                id.split(':').next_back().unwrap_or(id.as_str())
            }
        }
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemId::Block(id) => write!(f, "block:{}", id),
            ItemId::Item(id) => write!(f, "item:{}", id),
        }
    }
}

impl Default for ItemId {
    fn default() -> Self {
        Self::air()
    }
}
