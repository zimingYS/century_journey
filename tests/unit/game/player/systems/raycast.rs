use super::*;

#[test]
fn interaction_ray_starts_at_player_and_uses_player_facing() {
    let player = Transform::from_xyz(10.0, 20.0, 30.0)
        .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
    let (origin, direction) = player_interaction_ray(&player, 0.0);

    assert!((origin.y - (20.0 + PLAYER_EYE_HEIGHT)).abs() < 0.0001);
    assert!(direction.x < -0.99);
    assert!(origin.x < 10.0);
}

#[test]
fn interaction_ray_pitch_changes_direction_without_moving_to_camera() {
    let player = Transform::from_xyz(2.0, 3.0, 4.0);
    let (origin, level) = player_interaction_ray(&player, 0.0);
    let (pitched_origin, pitched) = player_interaction_ray(&player, 0.5);

    assert_eq!(origin, pitched_origin);
    assert!(level.y.abs() < 0.0001);
    assert!(pitched.y > 0.0);
}
