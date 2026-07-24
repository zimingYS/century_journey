use super::*;

#[test]
fn player_visual_action_curve_has_windup_strike_and_recovery() {
    assert!(action_swing(0.15) < 0.0);
    assert!(action_swing(0.56) > 0.95);
    assert_eq!(action_swing(1.0), 0.0);
}

#[test]
fn player_visual_walking_pose_uses_knees_and_ankles_for_foot_planting() {
    let pose = lower_pose(PlayerLocomotionState::Walk, std::f32::consts::FRAC_PI_2);
    assert!(pose.calf_l > 0.0);
    assert!(pose.foot_l < 0.0);
    assert_ne!(pose.foot_r, pose.foot_l);
}
