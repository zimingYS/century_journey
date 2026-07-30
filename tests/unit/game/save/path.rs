use super::*;

#[test]
fn world_root_is_relative_to_the_repository_save_directory() {
    assert_eq!(
        world_save_root("isolated_world"),
        PathBuf::from("saves").join("isolated_world")
    );
}
