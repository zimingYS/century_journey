use crate::shared::item_id::ItemId;
use crate::shared::tag::identifier::TagId;
use serde::{Deserialize, Serialize};

/// 合成材料表达式。
///
/// 后续可扩展 Any / All / Not 等组合表达式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Ingredient {
    Item { item: ItemId },
    Tag { tag: TagId },
}
