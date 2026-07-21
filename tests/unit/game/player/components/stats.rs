use super::*;

#[test]
fn hunger_saturation_absorbs_exhaustion_at_the_exact_boundary() {
    let mut hunger = Hunger {
        current: 10.0,
        max: 20.0,
        saturation: 2.0,
    };

    hunger.exhaust(2.0);
    assert_eq!(hunger.current, 10.0);
    assert_eq!(hunger.saturation, 0.0);

    hunger.exhaust(2.5);
    assert_eq!(hunger.current, 7.5);
    assert_eq!(hunger.saturation, 0.0);
}

#[test]
fn hunger_and_health_ignore_invalid_deltas_and_clamp_valid_ones() {
    let mut health = Health::default();
    health.apply_damage(f32::NAN);
    health.apply_damage(-3.0);
    assert_eq!(health.current, health.max);
    health.apply_damage(health.max);
    assert_eq!(health.current, 0.0);

    let mut hunger = Hunger {
        current: 19.0,
        max: 20.0,
        saturation: 0.0,
    };
    hunger.eat(4.0, 8.0);
    assert_eq!(hunger.current, 20.0);
    assert_eq!(hunger.saturation, 8.0);
    hunger.exhaust(f32::INFINITY);
    assert_eq!(hunger.current, 20.0);
}
