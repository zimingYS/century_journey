//! 提供配方原料与物品、标签的纯匹配规则。

use crate::content::recipe::definition::Ingredient;
use crate::content::tag::runtime::ItemTagIndex;
use crate::shared::item_id::ItemId;

/// 判断物品是否满足物品或标签形式的原料约束。
pub fn ingredient_matches(ingredient: &Ingredient, id: &ItemId, tag_index: &ItemTagIndex) -> bool {
    match ingredient {
        Ingredient::Item { item } => item == id,
        Ingredient::Tag { tag } => tag_index.contains(tag, id),
    }
}
