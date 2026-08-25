//! 语言文件解析与目录加载的镜像测试：拍平规则、元数据校验、排序与去重。

use super::*;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// 构造最小合法语言文件文本。
fn locale_text(id: &str, native_name: &str) -> String {
    format!("language = \"{id}\"\nnative-name = \"{native_name}\"\n")
}

/// 生成进程内唯一的临时目录，供目录扫描类测试隔离使用。
fn temp_directory(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("cj_locale_{name}_{}_{unique}", std::process::id()))
}

/// 直接构造语言文件结构，供组装函数测试使用。
fn language_file(id: &str, native_name: &str, entries: &[(&str, &str)]) -> LanguageFile {
    LanguageFile {
        info: LanguageInfo {
            id: LanguageId::new(id),
            native_name: native_name.to_string(),
        },
        entries: entries
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect(),
    }
}

#[test]
fn parse_flattens_nested_tables_into_dot_separated_keys() {
    let file = parse_locale_toml(
        "language = \"zh-CN\"\nnative-name = \"简体中文\"\n\
         [menu]\ntitle = \"世纪之旅\"\n[menu.row]\nlanguage = \"语言\"\n",
    )
    .unwrap();

    assert_eq!(file.info.id.as_str(), "zh-CN");
    assert_eq!(file.info.native_name, "简体中文");
    assert_eq!(
        file.entries.get("menu.title").map(String::as_str),
        Some("世纪之旅")
    );
    assert_eq!(
        file.entries.get("menu.row.language").map(String::as_str),
        Some("语言")
    );
}

#[test]
fn metadata_keys_do_not_enter_the_translation_table() {
    let file = parse_locale_toml(&locale_text("zh-CN", "简体中文")).unwrap();
    assert!(!file.entries.contains_key("language"));
    assert!(!file.entries.contains_key("native-name"));
    assert!(file.entries.is_empty());
}

#[test]
fn metadata_named_keys_in_deeper_tables_are_normal_entries() {
    let file = parse_locale_toml(
        "language = \"zh-CN\"\nnative-name = \"简体中文\"\n\
         [menu]\nlanguage = \"语言选项\"\n",
    )
    .unwrap();
    assert_eq!(
        file.entries.get("menu.language").map(String::as_str),
        Some("语言选项")
    );
}

#[test]
fn parse_fails_without_required_metadata() {
    assert!(parse_locale_toml("native-name = \"简体中文\"\n").is_err());
    assert!(parse_locale_toml("language = \"zh-CN\"\n").is_err());
    assert!(parse_locale_toml("language = 1\nnative-name = \"x\"\n").is_err());
}

#[test]
fn parse_fails_on_non_string_values() {
    let result = parse_locale_toml(
        "language = \"zh-CN\"\nnative-name = \"简体中文\"\n\
         [menu]\ncount = 3\n",
    );
    assert!(result.is_err());
}

#[test]
fn invalid_toml_syntax_is_rejected() {
    assert!(parse_locale_toml("language = ").is_err());
}

#[test]
fn load_returns_empty_list_for_missing_directory() {
    let missing = temp_directory("missing");
    let files = load_locales_from_dir(&missing).unwrap();
    assert!(files.is_empty());
}

#[test]
fn load_skips_non_toml_files_and_sorts_by_language_id() {
    let directory = temp_directory("scan");
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("zh-CN.toml"),
        locale_text("zh-CN", "简体中文"),
    )
    .unwrap();
    fs::write(
        directory.join("en-US.toml"),
        locale_text("en-US", "English"),
    )
    .unwrap();
    fs::write(directory.join("readme.txt"), "not a locale").unwrap();

    let files = load_locales_from_dir(&directory).unwrap();
    let ids: Vec<_> = files.iter().map(|file| file.info.id.as_str()).collect();
    assert_eq!(ids, vec!["en-US", "zh-CN"]);

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn load_reports_the_offending_file_on_parse_error() {
    let directory = temp_directory("broken");
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("broken.toml"), "language = ").unwrap();

    let result = load_locales_from_dir(&directory);
    let error = result.expect_err("残缺语言文件应当整体失败");
    assert!(
        error.contains("broken.toml"),
        "错误信息应定位到文件: {error}"
    );

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn build_sorts_languages_and_deduplicates_by_id() {
    let files = vec![
        language_file("zh-CN", "简体中文", &[("menu.play", "进入世界")]),
        language_file("en-US", "English", &[("menu.play", "Play World")]),
        // 同标识的重复文件：列表去重，翻译表由后加载者覆盖。
        language_file("zh-CN", "简体中文", &[("menu.quit", "退出游戏")]),
    ];
    let localization = build_localization(files);

    let ids: Vec<_> = localization
        .languages()
        .iter()
        .map(|info| info.id.as_str())
        .collect();
    assert_eq!(ids, vec!["en-US", "zh-CN"]);
    assert_eq!(
        localization.get_in(&LanguageId::new("zh-CN"), "menu.quit"),
        "退出游戏"
    );
    assert_eq!(
        localization.get_in(&LanguageId::new("zh-CN"), "menu.play"),
        "menu.play"
    );
}
