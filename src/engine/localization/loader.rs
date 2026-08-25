//! 语言目录扫描与 TOML 解析：把分节译文表拍平成点号分隔的扁平键。

use std::collections::BTreeMap;
use std::path::Path;

use super::store::{LanguageId, LanguageInfo, Localization};

/// 语言文件所在目录；新增语言只需在该目录放置一个自描述 TOML 文件。
pub const LOCALES_DIR: &str = "assets/locales";

/// 语言文件顶层的元数据键，不进入翻译表。
const METADATA_KEYS: &[&str] = &["language", "native-name"];

/// 单个语言文件的解析结果。
#[derive(Debug, Clone)]
pub struct LanguageFile {
    /// 语言标识与展示名。
    pub info: LanguageInfo,
    /// 扁平键到译文的映射。
    pub entries: BTreeMap<String, String>,
}

/// 解析语言文件文本；缺少元数据或存在非字符串译文时整体失败。
pub fn parse_locale_toml(text: &str) -> Result<LanguageFile, String> {
    let table: toml::Table =
        toml::from_str(text).map_err(|error| format!("语言文件语法无效: {error}"))?;
    let id = table
        .get("language")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| "缺少有效的 language 元数据".to_string())?
        .to_string();
    let native_name = table
        .get("native-name")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| "缺少有效的 native-name 元数据".to_string())?
        .to_string();
    let mut entries = BTreeMap::new();
    flatten_table(&table, "", &mut entries)?;
    Ok(LanguageFile {
        info: LanguageInfo {
            id: LanguageId::new(id),
            native_name,
        },
        entries,
    })
}

/// 递归拍平 TOML 表：字符串值是译文，子表继续下钻并拼接键前缀。
///
/// 仅顶层跳过元数据键；其他层级的同名键视为普通译文。
fn flatten_table(
    table: &toml::Table,
    prefix: &str,
    entries: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    for (key, value) in table {
        if prefix.is_empty() && METADATA_KEYS.contains(&key.as_str()) {
            continue;
        }
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        match value {
            toml::Value::String(text) => {
                entries.insert(full_key, text.clone());
            }
            toml::Value::Table(sub) => flatten_table(sub, &full_key, entries)?,
            other => {
                return Err(format!(
                    "条目 {full_key} 的值必须是字符串，实际为 {other:?}"
                ));
            }
        }
    }
    Ok(())
}

/// 扫描语言目录并解析全部 `.toml` 文件；目录缺失返回空列表。
///
/// 单个文件解析失败整体失败：残缺语言表会造成界面文本静默漏翻，
/// 不如启动时显式报错。
pub fn load_locales_from_dir(dir: &Path) -> Result<Vec<LanguageFile>, String> {
    let mut files = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(files),
        Err(error) => return Err(format!("读取语言目录失败: {error}")),
    };
    for entry in entries {
        let entry = entry.map_err(|error| format!("遍历语言目录失败: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("读取语言文件 {} 失败: {error}", path.display()))?;
        let file = parse_locale_toml(&text)
            .map_err(|error| format!("语言文件 {} {error}", path.display()))?;
        files.push(file);
    }
    files.sort_by(|a, b| a.info.id.cmp(&b.info.id));
    Ok(files)
}

/// 把语言文件集合组装成查询资源；重复语言以后加载者覆盖，列表保持去重有序。
pub fn build_localization(files: Vec<LanguageFile>) -> Localization {
    let mut languages = Vec::new();
    let mut tables = BTreeMap::new();
    for file in files {
        if !languages.contains(&file.info) {
            languages.push(file.info.clone());
        }
        tables.insert(file.info.id.clone(), file.entries);
    }
    languages.sort_by(|a, b| a.id.cmp(&b.id));
    Localization::new(languages, tables)
}

#[cfg(test)]
#[path = "../../../tests/unit/engine/localization/loader.rs"]
mod tests;
