use super::*;

#[test]
fn feedback_fix_deep_night_uses_readable_exposure() {
    let night_ev100 = visibility_exposure_ev100(0.0, 1.0, 1.0);
    let noon_ev100 = visibility_exposure_ev100(1.0, -1.0, 0.0);

    assert_eq!(night_ev100, NIGHT_EXPOSURE_EV100);
    assert!(night_ev100 < noon_ev100);
}
