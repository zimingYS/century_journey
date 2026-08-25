use super::*;
use crate::engine::localization::{build_localization, parse_locale_toml};

#[test]
fn settings_are_clamped_to_supported_ranges() {
    let localization = build_localization(Vec::new());
    let mut settings = GameSettings::default();
    adjust_setting(
        &mut settings,
        SettingAction::RenderDistance(-100),
        &localization,
    );
    adjust_setting(
        &mut settings,
        SettingAction::MasterVolume(-5.0),
        &localization,
    );
    adjust_setting(&mut settings, SettingAction::UiScale(5.0), &localization);
    assert_eq!(settings.render_distance, 2);
    assert_eq!(settings.master_volume, 0.0);
    assert_eq!(settings.ui_scale, 1.6);
}

/// 构造含 zh-CN 与 en-US 两种语言的查询资源；注册顺序按标识排序为 en-US、zh-CN。
fn localization_with_two_languages() -> Localization {
    build_localization(vec![
        parse_locale_toml("language = \"zh-CN\"\nnative-name = \"简体中文\"\n").unwrap(),
        parse_locale_toml("language = \"en-US\"\nnative-name = \"English\"\n").unwrap(),
    ])
}

#[test]
fn language_cycles_forward_and_backward_with_wraparound() {
    let localization = localization_with_two_languages();
    let mut settings = GameSettings {
        language: "zh-CN".to_string(),
        ..GameSettings::default()
    };

    adjust_setting(
        &mut settings,
        SettingAction::CycleLanguage(1),
        &localization,
    );
    assert_eq!(settings.language, "en-US");
    // 越过末尾后回到首个语言。
    adjust_setting(
        &mut settings,
        SettingAction::CycleLanguage(1),
        &localization,
    );
    assert_eq!(settings.language, "zh-CN");
    // 负向步进越过开头后回到末尾语言。
    adjust_setting(
        &mut settings,
        SettingAction::CycleLanguage(-1),
        &localization,
    );
    assert_eq!(settings.language, "en-US");
    assert_eq!(
        localization.native_name_of(&LanguageId::new(&settings.language)),
        Some("English")
    );
}

#[test]
fn language_cycle_resets_to_the_first_step_when_current_is_unregistered() {
    let localization = localization_with_two_languages();
    let mut settings = GameSettings {
        language: "fr-FR".to_string(),
        ..GameSettings::default()
    };

    adjust_setting(
        &mut settings,
        SettingAction::CycleLanguage(1),
        &localization,
    );
    assert_eq!(settings.language, "zh-CN");
}

#[test]
fn language_cycle_keeps_the_current_value_without_registered_languages() {
    let localization = build_localization(Vec::new());
    let mut settings = GameSettings {
        language: "zh-CN".to_string(),
        ..GameSettings::default()
    };

    adjust_setting(
        &mut settings,
        SettingAction::CycleLanguage(1),
        &localization,
    );
    assert_eq!(settings.language, "zh-CN");
}
