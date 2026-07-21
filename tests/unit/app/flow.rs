use super::*;

#[test]
fn world_names_are_safe_and_non_empty() {
    assert_eq!(sanitize_world_name(" My World "), "my_world");
    assert!(valid_world_id(&sanitize_world_name("../../")));
    assert!(!valid_world_id("../unsafe"));
}

#[test]
fn settings_are_clamped_to_supported_ranges() {
    let mut settings = GameSettings::default();
    adjust_setting(&mut settings, SettingAction::RenderDistance(-100));
    adjust_setting(&mut settings, SettingAction::MasterVolume(-5.0));
    adjust_setting(&mut settings, SettingAction::UiScale(5.0));
    assert_eq!(settings.render_distance, 2);
    assert_eq!(settings.master_volume, 0.0);
    assert_eq!(settings.ui_scale, 1.6);
}
