use std::fmt;
use serde::{Deserialize, Serialize};

/// 物品唯一标识符
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemId{
    /// 方块类型
    Block(String),
}

impl ItemId {
    /// 从方块标识符创建
    pub fn block(id: impl Into<String>) -> Self {
        Self::Block(id.into())
    }

    /// 空气方块
    pub fn air() -> Self {
        Self::Block("century_journey:air".to_string())
    }

    /// 是否为空气 (空槽位)
    pub fn is_air(&self) -> bool {
        match self {
            ItemId::Block(id) => id == "century_journey:air",
        }
    }

    /// 获取方块标识符引用，非 Block 变体返回 None
    pub fn as_block_id(&self) -> Option<&str> {
        match self {
            ItemId::Block(id) => Some(id),
        }
    }

    /// 转为显示文本
    pub fn display_name(&self) -> &str {
        match self {
            ItemId::Block(id) => id.as_str(),
        }
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemId::Block(id) => write!(f, "{}", id),
        }
    }
}

impl Default for ItemId {
    fn default() -> Self {
        Self::air()
    }
}
