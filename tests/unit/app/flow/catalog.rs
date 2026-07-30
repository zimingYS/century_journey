use super::*;

#[test]
fn world_names_are_safe_and_non_empty() {
    assert_eq!(sanitize_world_name(" My World "), "my_world");
    assert!(valid_world_id(&sanitize_world_name("../../")));
    assert!(!valid_world_id("../unsafe"));
}
