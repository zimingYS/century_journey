use super::*;

#[test]
fn stage_seven_horizontal_velocity_accelerates_and_decelerates_gradually() {
    let accelerated = approach_velocity(Vec3::ZERO, Vec3::X * 10.0, 2.0);
    assert_eq!(accelerated, Vec3::X * 2.0);

    let decelerated = approach_velocity(accelerated, Vec3::ZERO, 0.5);
    assert_eq!(decelerated, Vec3::X * 1.5);
    assert_ne!(decelerated, Vec3::ZERO);
}

#[test]
fn ground_deceleration_stops_sprint_without_ice_like_sliding() {
    let movement = PlayerMovement::default();
    let dt = 1.0 / 60.0;
    let mut velocity = Vec3::X * movement.movement_speed * movement.sprint_factor;

    for _ in 0..5 {
        velocity = approach_velocity(velocity, Vec3::ZERO, movement.deceleration * dt);
    }

    assert_eq!(velocity, Vec3::ZERO);
}
