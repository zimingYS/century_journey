use crate::shared::identifier::Identifier;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 物品唯一标识符
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ItemId(Identifier);

impl ItemId {
    pub fn new(id: Identifier) -> Self {
        Self(id)
    }

    pub fn parse(raw: &str) -> Result<Self, crate::shared::identifier::IdentifierError> {
        Identifier::parse(raw).map(Self)
    }

    pub fn air() -> Self {
        Self(Identifier::new("century_journey", "air"))
    }

    pub fn is_air(&self) -> bool {
        self.0 == Identifier::new("century_journey", "air")
    }

    pub fn identifier(&self) -> &Identifier {
        &self.0
    }

    pub fn display_name(&self) -> &str {
        self.0.path()
    }

    pub fn block(id: impl AsRef<str>) -> Self {
        Self::parse(id.as_ref()).unwrap_or_else(|e| panic!("非法方块标识符: {e}"))
    }
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
impl Serialize for ItemId {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}
impl<'de> Deserialize<'de> for ItemId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Identifier::deserialize(d).map(Self)
    }
}
