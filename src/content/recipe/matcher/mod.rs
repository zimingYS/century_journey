use crate::content::recipe::definition::Ingredient;
use crate::shared::item_id::ItemId;

pub fn ingredient_matches(ingredient: &Ingredient, id: &ItemId, _tag_registry: &()) -> bool {
    match ingredient {
        Ingredient::Item { item } => item == id,
        Ingredient::Tag { .. } => {
            // TODO: Item Tag matching requires Item RuntimeId system
            false
        }
    }
}
