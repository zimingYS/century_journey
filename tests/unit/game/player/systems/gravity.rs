use super::*;

#[test]
fn stage_seven_fall_damage_has_safe_distance_and_scales_after_it() {
    assert_eq!(fall_damage_from_distance(3.9), 0.0);
    assert_eq!(fall_damage_from_distance(4.0), 1.0);
    assert_eq!(fall_damage_from_distance(8.8), 5.0);
}
