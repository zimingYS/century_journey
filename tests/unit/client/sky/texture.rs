use super::*;

#[test]
fn sun_has_a_round_opaque_core_and_transparent_corners() {
    let texture = generate_sun_texture(128);
    assert_eq!(texture.get_pixel(64, 64).0[3], 255);
    assert_eq!(texture.get_pixel(0, 0).0[3], 0);
    assert_eq!(texture.get_pixel(64, 0).0[3], 0);
}
