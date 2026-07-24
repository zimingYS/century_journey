use super::*;

fn simulate_particle(fps: u32) -> (Vec3, Vec3) {
    let mut particle = FeedbackParticle {
        velocity: Vec3::new(2.0, 3.0, -1.0),
        age: 0.0,
        lifetime: 2.0,
        initial_scale: 0.1,
        spin: 0.0,
    };
    let mut transform = Transform::default();
    let delta = 1.0 / fps as f32;
    for _ in 0..fps {
        advance_particle_motion(&mut particle, &mut transform, delta);
    }
    (transform.translation, particle.velocity)
}

#[test]
fn particle_motion_is_stable_across_render_rates() {
    let at_10 = simulate_particle(10);
    let at_20 = simulate_particle(20);
    let at_60 = simulate_particle(60);
    let at_144 = simulate_particle(144);

    assert!(at_10.0.distance(at_20.0) < 0.01);
    assert!(at_20.0.distance(at_60.0) < 0.01);
    assert!(at_60.0.distance(at_144.0) < 0.01);
    assert!(at_10.1.distance(at_20.1) < 0.01);
    assert!(at_20.1.distance(at_60.1) < 0.01);
    assert!(at_60.1.distance(at_144.1) < 0.01);
}
