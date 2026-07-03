use crate::content::recipe::definition::Ingredient;
use crate::shared::item_id::ItemId;
use crate::shared::tag::identifier::TagRegistryType;
use crate::shared::tag::registry::TagRegistry;

pub fn ingredient_matches(
    ingredient: &Ingredient,
    id: &ItemId,
    tag_registry: &TagRegistry,
) -> bool {
    match ingredient {
        Ingredient::Item { item } => item == id,
        Ingredient::Tag { tag } => tag_registry
            .get_registry(&TagRegistryType::Item)
            .map(|typed| typed.is_tagged(&id.to_string(), tag))
            .unwrap_or(false),
    }
}
