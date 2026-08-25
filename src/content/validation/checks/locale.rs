//! 语言资源文件的可解析性与各语言键集合一致性校验。

use std::collections::BTreeSet;

use super::super::ContentCheckReport;
use crate::engine::asset::AssetFiles;
use crate::engine::localization::{FALLBACK_LANGUAGE, parse_locale_toml};

/// 每种语言最多逐条报告的键差异数量，超出部分汇总为一条提示。
const MAX_REPORTED_KEYS: usize = 20;

/// 校验语言目录：文件可解析、语言标识唯一、回退语言存在、各语言键集合与回退语言一致。
///
/// 键集合不一致会造成部分界面在切换语言后回退到中文或显示键名，
/// 因此在离线检查阶段就按错误处理。
pub(in crate::content::validation) fn validate_locales(
    files: &AssetFiles<'_>,
    report: &mut ContentCheckReport,
) {
    let locale_files = files.resolved_files("locales", "toml");
    report.checked_files += locale_files.len();

    let mut languages: Vec<(String, BTreeSet<String>)> = Vec::new();
    for file in &locale_files {
        let text = match std::fs::read_to_string(&file.full_path) {
            Ok(text) => text,
            Err(error) => {
                report.errors.push(format!(
                    "{}:locale.data: cannot read locale file: {error}",
                    file.full_path.display()
                ));
                continue;
            }
        };
        match parse_locale_toml(&text) {
            Ok(locale) => {
                // 同名文件会被来源覆盖合并；这里只对合并结果检查标识唯一性。
                let id = locale.info.id.0.clone();
                if languages.iter().any(|(known, _)| *known == id) {
                    report.errors.push(format!(
                        "locales/{id}:locale.language: duplicate language {id}"
                    ));
                    continue;
                }
                languages.push((id, locale.entries.keys().cloned().collect()));
            }
            Err(error) => report.errors.push(format!(
                "{}:locale.format: {error}",
                file.full_path.display()
            )),
        }
    }

    let Some(fallback_keys) = languages
        .iter()
        .find(|(id, _)| id == FALLBACK_LANGUAGE)
        .map(|(_, keys)| keys.clone())
    else {
        report.errors.push(format!(
            "locales:locale.language: missing fallback language {FALLBACK_LANGUAGE}"
        ));
        return;
    };

    for (id, keys) in &languages {
        if id == FALLBACK_LANGUAGE {
            continue;
        }
        let missing: Vec<_> = fallback_keys.difference(keys).collect();
        report_key_diff(report, id, "missing", &missing);
        let extra: Vec<_> = keys.difference(&fallback_keys).collect();
        report_key_diff(report, id, "extra", &extra);
    }
}

/// 逐条报告键差异，超过上限后汇总剩余数量，避免单个残缺文件淹没报告。
fn report_key_diff(report: &mut ContentCheckReport, language: &str, kind: &str, keys: &[&String]) {
    for key in keys.iter().take(MAX_REPORTED_KEYS) {
        report.errors.push(format!(
            "locales/{language}:locale.keys: {kind} key {key} compared with {FALLBACK_LANGUAGE}"
        ));
    }
    let remaining = keys.len().saturating_sub(MAX_REPORTED_KEYS);
    if remaining > 0 {
        report.errors.push(format!(
            "locales/{language}:locale.keys: ... and {remaining} more {kind} keys"
        ));
    }
}
