use super::*;

#[test]
fn new_world_start_time_is_daytime() {
    let time = TimeOfDay::default();
    assert_eq!(time.current_time, NEW_WORLD_START_TIME);
    assert_eq!(time.phase(), TimePhase::Day);
}
