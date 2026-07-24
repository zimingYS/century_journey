use super::*;

#[test]
fn scale_covers_supported_screenshot_resolutions() {
    let settings = UiScaleSettings::default();
    assert!((settings.resolved_scale(Vec2::new(1280.0, 720.0)) - 0.67).abs() < 0.001);
    assert!((settings.resolved_scale(Vec2::new(1920.0, 1080.0)) - 1.0).abs() < 0.001);
    assert!((settings.resolved_scale(Vec2::new(2560.0, 1440.0)) - 4.0 / 3.0).abs() < 0.001);
}
