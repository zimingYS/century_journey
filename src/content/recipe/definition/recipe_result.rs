use crate::shared::item_id::ItemId;
use serde::{Deserialize, Serialize};

/// 配方输出。
///
/// 后续可以扩展经验、NBT、组件、副产物等。
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct RecipeResult {
    /// 输出物品
    pub item: ItemId,

    /// 输出数量
    #[serde(default = "default_count")]
    pub count: u32,
}

pub fn default_count() -> u32 {
    1
}
