//! 定义跨 Content、Game 与 Client 使用的物品标识值。

use crate::shared::identifier::Identifier;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 物品唯一标识符
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemId(Identifier);

impl ItemId {
    /// 使用给定参数创建新实例。
    pub fn new(id: Identifier) -> Self {
        Self(id)
    }

    /// 解析并校验带命名空间的标识文本。
    pub fn parse(raw: &str) -> Result<Self, crate::shared::identifier::IdentifierError> {
        Identifier::parse(raw).map(Self)
    }

    /// 返回表示空物品的规范标识。
    pub fn air() -> Self {
        Self(Identifier::new("century_journey", "air"))
    }

    /// 判断该标识是否表示空物品。
    pub fn is_air(&self) -> bool {
        self.0 == Identifier::new("century_journey", "air")
    }

    /// 返回物品的稳定内容标识。
    pub fn identifier(&self) -> &Identifier {
        &self.0
    }

    /// 返回适合界面展示的无命名空间名称。
    pub fn display_name(&self) -> &str {
        self.0.path()
    }

    /// 从方块标识构造对应的物品标识。
    pub fn block(id: impl AsRef<str>) -> Self {
        Self::parse(id.as_ref()).unwrap_or_else(|e| panic!("非法方块标识符: {e}"))
    }
    /// 从物品标识文本构造物品 ID。
    pub fn item(id: impl AsRef<str>) -> Self {
        Self::parse(id.as_ref()).unwrap_or_else(|e| panic!("非法物品标识符: {e}"))
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ItemId {
    fn default() -> Self {
        Self::air()
    }
}
