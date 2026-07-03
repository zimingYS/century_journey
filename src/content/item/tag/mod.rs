use crate::content::item::ItemRegistry;
use crate::content::tag::populate::populate_tags;
use crate::shared::tag::identifier::TagRegistryType;
use crate::shared::tag::registry::TagRegistry;

pub fn auto_populate_from_item_tags(tag_registry: &mut TagRegistry, item_registry: &ItemRegistry) {
    let added = populate_tags(
        tag_registry,
        TagRegistryType::Item,
        item_registry.all_items().cloned(),
    );
    log::info!("[Tag] 从 ItemDefinition.tags 自动构建 {} 条标签映射", added);
}
