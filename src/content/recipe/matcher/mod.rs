use crate::content::recipe::definition::Ingredient;
use crate::content::tag::runtime::ItemTagIndex;
use crate::shared::item_id::ItemId;

pub fn ingredient_matches(ingredient: &Ingredient, id: &ItemId, tag_index: &ItemTagIndex) -> bool {
    match ingredient {
        Ingredient::Item { item } => item == id,
        Ingredient::Tag { tag } => tag_index.contains(tag, id),
    }
}
