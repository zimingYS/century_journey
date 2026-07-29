use super::*;
use std::collections::HashSet;

#[test]
fn required_support_accepts_only_members_of_the_declared_tag() {
    let tag = TagId::new("century_journey", "sapling_supports");
    let mut tags = RuntimeTagRegistry::default();
    tags.insert(tag.clone(), HashSet::from([3]));

    assert!(has_required_support(&tag, 3, &tags));
    assert!(!has_required_support(&tag, 4, &tags));
}
