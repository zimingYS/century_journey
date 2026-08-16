use super::*;

#[test]
fn player_visual_action_curve_has_windup_strike_and_recovery() {
    assert!(action_swing(0.15) < 0.0);
    assert!(action_swing(0.56) > 0.95);
    assert_eq!(action_swing(1.0), 0.0);
}

#[test]
fn player_visual_walking_pose_tracks_model_animation_keyframes() {
    // 模型 walk 动画 t=0：左腿前摆 +30°（quat.x=sin15°，全角 2×15°）、左膝弯曲 -30°。
    let pose0 = lower_pose(PlayerLocomotionState::Walk, 0.0);
    assert!(
        (pose0.thigh_l - 0.5236).abs() < 0.02,
        "thigh_l={}",
        pose0.thigh_l
    );
    assert!(
        (pose0.calf_l + 0.5236).abs() < 0.02,
        "calf_l={}",
        pose0.calf_l
    );
    // 右腿为左腿反相（模型未导出 right_leg，按左腿相位 +0.5 生成）：
    // t=0 时右腿后摆、相位 +0.5 周期（π）后前摆，两者之和为零。
    let pose_half = lower_pose(PlayerLocomotionState::Walk, std::f32::consts::PI);
    assert!((pose0.thigh_r + pose_half.thigh_r).abs() < 0.001);
    assert!((pose0.thigh_r + pose0.thigh_l).abs() < 0.001);
}

#[test]
fn player_visual_fall_pose_matches_model_animation() {
    // 模型 fall 动画：左腿前抬 +70°（quat.x=sin35°）、右小腿后屈 -70°。
    let pose = lower_pose(PlayerLocomotionState::Fall, 0.0);
    assert!(
        (pose.thigh_l - 1.2217).abs() < 0.02,
        "thigh_l={}",
        pose.thigh_l
    );
    assert!(
        (pose.calf_r + 1.2217).abs() < 0.02,
        "calf_r={}",
        pose.calf_r
    );
    // 右大腿保持伸直（模型未导出该通道）。
    assert!(pose.thigh_r.abs() < 0.001);
}
