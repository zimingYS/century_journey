use crate::shared::tag::identifier::{TagId, TagRegistryType};
use crate::shared::tag::registry::TagRegistry;

pub trait Taggable {
    fn identifier(&self) -> &str;
    fn tags(&self) -> &[String];
}

pub fn populate_tags<T: Taggable>(
    tag_registry: &mut TagRegistry,
    registry_type: TagRegistryType,
    entries: impl Iterator<Item = T>,
) -> usize {
    let typed = tag_registry.get_or_create_registry(registry_type);
    let mut added = 0usize;
    for entry in entries {
        for tag_str in entry.tags() {
            let tag_id = tag_str
                .contains('/')
                .then(|| tag_str.split_once('/'))
                .flatten()
                .map(|(ns, p)| TagId::new(ns, p))
                .unwrap_or_else(|| TagId::new("century_journey", tag_str.as_str()));
            typed.insert(tag_id, entry.identifier().to_string());
            added += 1;
        }
    }
    added
}
