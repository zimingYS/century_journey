use super::*;

#[test]
fn beginning_world_session_discards_previous_save_runtime_state() {
    let mut manager = PlayerSaveManager::default();
    manager.set_dirty(SaveDirtySource::Inventory);
    manager.total_saves = 7;
    manager.last_save_time = 12.0;
    manager.last_saved_position = Vec3::splat(9.0);
    manager.auto_save_timer = 0.5;

    manager.begin_session();

    assert!(!manager.dirty);
    assert!(manager.last_dirty_source.is_none());
    assert_eq!(manager.total_saves, 0);
    assert_eq!(manager.last_save_time, 0.0);
    assert_eq!(manager.last_saved_position, Vec3::ZERO);
    assert_eq!(
        manager.auto_save_timer,
        PlayerSaveManager::AUTO_SAVE_INTERVAL
    );
}
