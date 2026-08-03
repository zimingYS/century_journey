use super::*;

#[test]
fn underwater_depth_steps_toward_target_and_clamps() {
    let stepped = underwater_depth_step(0.0, 1.0, 6.0, 1.0 / 60.0);
    assert!(stepped > 0.0 && stepped < 1.0);
    let again = underwater_depth_step(stepped, 1.0, 6.0, 1.0 / 60.0);
    assert!(again > stepped && again <= 1.0);
    let back = underwater_depth_step(0.8, 0.0, 6.0, 1.0 / 60.0);
    assert!((0.0..0.8).contains(&back));
    assert_eq!(underwater_depth_step(4.0, -2.0, 10.0, 1.0), 0.0);
}

#[test]
fn water_flow_offset_advances_and_wraps_by_tile() {
    assert!((water_flow_offset(1.0, 0.12, 1.0) - 0.12).abs() < 1e-5);
    let wrapped = water_flow_offset(10.0, 0.12, 1.0);
    assert!((0.0..1.0).contains(&wrapped));
    assert_eq!(water_flow_offset(100.0, 0.0, 1.0), 0.0);
    assert_eq!(water_flow_offset(1.0, 1.0, 0.0), 0.0);
}

#[test]
fn underwater_alpha_maps_depth_to_max_alpha() {
    assert_eq!(compute_underwater_alpha(0.0, 0.42), 0.0);
    assert!((compute_underwater_alpha(1.0, 0.42) - 0.42).abs() < 1e-5);
    assert!((compute_underwater_alpha(0.5, 0.42) - 0.21).abs() < 1e-5);
    assert_eq!(compute_underwater_alpha(2.0, 0.42), 0.42);
    assert_eq!(compute_underwater_alpha(-1.0, 0.42), 0.0);
}

#[test]
fn water_depth_factor_tracks_shallow_and_deep_edges() {
    assert_eq!(water_depth_factor(0.0, 6.0), 0.0);
    assert!((water_depth_factor(3.0, 6.0) - 0.5).abs() < 1e-5);
    assert_eq!(water_depth_factor(8.0, 6.0), 1.0);
    assert_eq!(water_depth_factor(1.0, 0.0), 1.0);
}
