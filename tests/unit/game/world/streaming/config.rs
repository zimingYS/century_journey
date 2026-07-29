use super::*;

#[test]
fn data_window_is_horizontal_circle_with_limited_vertical_range() {
    let config = WorldStreamingConfig::default();
    let (ordered, expected) = config.rebuild_expected_chunks(IVec3::ZERO, Vec2::Y);

    assert_eq!(ordered.len(), expected.len());
    assert!(expected.len() < 19usize.pow(3));
    assert!(expected.contains(&IVec3::new(9, 0, 0)));
    assert!(!expected.contains(&IVec3::new(10, 0, 0)));
    assert!(expected.contains(&IVec3::new(0, 2, 0)));
    assert!(expected.contains(&IVec3::new(0, -3, 0)));
    assert!(!expected.contains(&IVec3::new(0, 3, 0)));
    assert!(!expected.contains(&IVec3::new(0, -4, 0)));
}

#[test]
fn chunks_in_front_are_prioritized_over_chunks_behind() {
    let config = WorldStreamingConfig::default();
    let (ordered, _) = config.rebuild_expected_chunks(IVec3::ZERO, Vec2::NEG_Y);

    let front = ordered
        .iter()
        .position(|&pos| pos == IVec3::new(0, 0, -1))
        .unwrap();
    let back = ordered
        .iter()
        .position(|&pos| pos == IVec3::new(0, 0, 1))
        .unwrap();
    assert!(front < back);
}
