//! 本地化查询资源的镜像测试：回退链、插值、语言切换与空资源退化。

use super::*;

/// 构造带两种语言的查询资源：zh-CN 是回退语言且多一个仅中文的键。
fn localization_with_two_languages() -> Localization {
    let languages = vec![
        LanguageInfo {
            id: LanguageId::new("en-US"),
            native_name: "English".to_string(),
        },
        LanguageInfo {
            id: LanguageId::new("zh-CN"),
            native_name: "简体中文".to_string(),
        },
    ];
    let mut zh_table = BTreeMap::new();
    zh_table.insert("menu.play".to_string(), "进入世界".to_string());
    zh_table.insert("menu.only_zh".to_string(), "仅中文".to_string());
    let mut en_table = BTreeMap::new();
    en_table.insert("menu.play".to_string(), "Play World".to_string());
    let mut tables = BTreeMap::new();
    tables.insert(LanguageId::new("zh-CN"), zh_table);
    tables.insert(LanguageId::new("en-US"), en_table);
    Localization::new(languages, tables)
}

#[test]
fn active_language_defaults_to_the_fallback_language() {
    let localization = localization_with_two_languages();
    assert_eq!(localization.active().as_str(), FALLBACK_LANGUAGE);
    assert_eq!(localization.get("menu.play"), "进入世界");
}

#[test]
fn missing_key_in_active_language_falls_back() {
    let mut localization = localization_with_two_languages();
    assert!(localization.set_active(&LanguageId::new("en-US")));
    // en-US 缺少该键，回退到 zh-CN 的译文而不是键本身。
    assert_eq!(localization.get("menu.only_zh"), "仅中文");
    assert_eq!(localization.get("menu.play"), "Play World");
}

#[test]
fn missing_key_in_both_languages_returns_the_key_itself() {
    let mut localization = localization_with_two_languages();
    localization.set_active(&LanguageId::new("en-US"));
    assert_eq!(localization.get("menu.absent"), "menu.absent");
}

#[test]
fn format_interpolates_known_placeholders_and_keeps_unknown() {
    let mut localization = localization_with_two_languages();
    let mut zh_table = BTreeMap::new();
    zh_table.insert(
        "menu.entry".to_string(),
        "{id} 种子 {seed} 未知 {unknown}".to_string(),
    );
    let mut tables = BTreeMap::new();
    tables.insert(LanguageId::new("zh-CN"), zh_table);
    localization = Localization::new(localization.languages().to_vec(), tables);

    let text = localization.format("menu.entry", &[("id", "world-1"), ("seed", "42")]);
    assert_eq!(text, "world-1 种子 42 未知 {unknown}");
}

#[test]
fn set_active_rejects_unregistered_languages() {
    let mut localization = localization_with_two_languages();
    assert!(!localization.set_active(&LanguageId::new("ja-JP")));
    assert_eq!(localization.active().as_str(), FALLBACK_LANGUAGE);
}

#[test]
fn empty_registry_degrades_to_returning_keys() {
    let localization = Localization::new(Vec::new(), BTreeMap::new());
    assert!(localization.languages().is_empty());
    assert_eq!(localization.active().as_str(), FALLBACK_LANGUAGE);
    assert_eq!(localization.get("menu.play"), "menu.play");
}

#[test]
fn first_language_is_active_when_fallback_is_missing() {
    let languages = vec![LanguageInfo {
        id: LanguageId::new("ja-JP"),
        native_name: "日本語".to_string(),
    }];
    let mut ja_table = BTreeMap::new();
    ja_table.insert("menu.play".to_string(), "ワールドに入る".to_string());
    let mut tables = BTreeMap::new();
    tables.insert(LanguageId::new("ja-JP"), ja_table);

    let localization = Localization::new(languages, tables);
    assert_eq!(localization.active().as_str(), "ja-JP");
    assert_eq!(localization.get("menu.play"), "ワールドに入る");
    // 回退表不存在时直接返回键本身。
    assert_eq!(localization.get("menu.absent"), "menu.absent");
}

#[test]
fn get_in_follows_the_same_fallback_chain_for_any_language() {
    let localization = localization_with_two_languages();
    let en = LanguageId::new("en-US");
    let ja = LanguageId::new("ja-JP");
    assert_eq!(localization.get_in(&en, "menu.play"), "Play World");
    assert_eq!(localization.get_in(&en, "menu.only_zh"), "仅中文");
    assert_eq!(localization.get_in(&ja, "menu.play"), "进入世界");
    assert_eq!(localization.get_in(&ja, "menu.absent"), "menu.absent");
}

#[test]
fn native_name_of_resolves_registered_languages_only() {
    let localization = localization_with_two_languages();
    assert_eq!(
        localization.native_name_of(&LanguageId::new("zh-CN")),
        Some("简体中文")
    );
    assert_eq!(localization.native_name_of(&LanguageId::new("ja-JP")), None);
}

#[test]
fn keys_of_lists_entries_and_is_empty_for_unknown_languages() {
    let localization = localization_with_two_languages();
    let keys: Vec<_> = localization.keys_of(&LanguageId::new("en-US")).collect();
    assert_eq!(keys, vec!["menu.play"]);
    assert_eq!(localization.keys_of(&LanguageId::new("ja-JP")).count(), 0);
}
