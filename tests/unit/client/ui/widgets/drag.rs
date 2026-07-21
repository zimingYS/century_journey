use super::cursor_ui_position;
use bevy::math::Vec2;

#[test]
fn cursor_offset_stays_constant_across_ui_scales() {
    let cursor = Vec2::new(900.0, 480.0);
    for scale in [0.67, 1.0, 4.0 / 3.0] {
        let rendered_position = cursor_ui_position(cursor, scale) * scale;
        std::assert!((rendered_position - cursor - Vec2::splat(12.0)).length() < 0.001);
    }
}
