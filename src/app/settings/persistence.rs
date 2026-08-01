//! 使用命名 MessagePack 文档持久化设置，并兼容读取历史 JSON 文件。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::model::GameSettings;
use crate::engine::{document, persistence};

/// 当前设置文档的外层编码格式；普通字段变化不得递增此值。
pub const SETTINGS_DOCUMENT_FORMAT: u32 = 1;
const SETTINGS_MAGIC: [u8; 4] = *b"CJST";
const MAX_SETTINGS_DOCUMENT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct SettingsDocument {
    game_version: String,
    settings: GameSettings,
}

impl Default for SettingsDocument {
    fn default() -> Self {
        Self {
            game_version: env!("CARGO_PKG_VERSION").to_string(),
            settings: GameSettings::default(),
        }
    }
}

/// 冻结的历史 JSON 顶层布局，仅用于迁移已有设置。
#[derive(Debug, Serialize, Deserialize)]
struct LegacyJsonSettings {
    format_version: u32,
    game_version: String,
    settings: GameSettings,
}

/// 返回当前设置文档路径。
pub fn settings_path() -> PathBuf {
    PathBuf::from("config").join("settings.dat")
}

fn legacy_settings_path() -> PathBuf {
    PathBuf::from("config").join("settings.json")
}

/// 判断当前或历史设置主文件是否存在。
pub fn settings_file_exists() -> bool {
    settings_path().exists() || legacy_settings_path().exists()
}

/// 从默认路径读取设置；首次读取旧 JSON 后会写出当前文档。
pub fn load_settings() -> Result<GameSettings, String> {
    let current = settings_path();
    if current.exists() {
        return load_settings_from(&current);
    }

    let legacy = legacy_settings_path();
    let settings = load_legacy_settings_from(&legacy)?;
    save_settings(&settings)?;
    Ok(settings)
}

/// 将设置原子写入默认路径，并保留可恢复备份。
pub fn save_settings(settings: &GameSettings) -> Result<(), String> {
    save_settings_to(&settings_path(), settings)
}

/// 从指定路径读取当前设置文档，供运行时和持久化测试复用。
pub fn load_settings_from(path: &Path) -> Result<GameSettings, String> {
    let bytes = persistence::read_verified(path, validate_settings_bytes)
        .map_err(|error| error.to_string())?;
    decode_settings(&bytes)
}

/// 将设置原子写入指定路径，供运行时和持久化测试复用。
pub fn save_settings_to(path: &Path, settings: &GameSettings) -> Result<(), String> {
    let file = SettingsDocument {
        game_version: env!("CARGO_PKG_VERSION").to_string(),
        settings: normalize_settings(settings.clone()),
    };
    let bytes = document::encode_named(SETTINGS_MAGIC, SETTINGS_DOCUMENT_FORMAT, &file)?;
    persistence::atomic_write_verified(path, &bytes, validate_settings_bytes)
        .map_err(|error| error.to_string())
}

/// 判断当前文档或历史 JSON 是否存在可恢复的有效备份。
pub fn settings_backup_available() -> bool {
    persistence::has_valid_backup(&settings_path(), validate_settings_bytes)
        || persistence::has_valid_backup(&legacy_settings_path(), validate_legacy_settings_bytes)
}

/// 使用有效备份恢复设置；历史 JSON 备份会直接升级为当前文档。
pub fn restore_settings_backup() -> Result<(), String> {
    let current = settings_path();
    if persistence::has_valid_backup(&current, validate_settings_bytes) {
        return persistence::restore_backup(&current, validate_settings_bytes)
            .map_err(|error| error.to_string());
    }

    let legacy = legacy_settings_path();
    let bytes = persistence::read_backup_verified(&legacy, validate_legacy_settings_bytes)
        .map_err(|error| error.to_string())?;
    let settings = decode_legacy_settings(&bytes)?;
    save_settings(&settings)
}

fn decode_settings(bytes: &[u8]) -> Result<GameSettings, String> {
    let file: SettingsDocument = document::decode_named(
        bytes,
        SETTINGS_MAGIC,
        SETTINGS_DOCUMENT_FORMAT,
        MAX_SETTINGS_DOCUMENT_BYTES,
    )?;
    Ok(normalize_settings(file.settings))
}

fn load_legacy_settings_from(path: &Path) -> Result<GameSettings, String> {
    let bytes = persistence::read_verified(path, validate_legacy_settings_bytes)
        .map_err(|error| error.to_string())?;
    decode_legacy_settings(&bytes)
}

fn decode_legacy_settings(bytes: &[u8]) -> Result<GameSettings, String> {
    let file: LegacyJsonSettings =
        serde_json::from_slice(bytes).map_err(|error| format!("旧设置文件格式无效: {error}"))?;
    if file.format_version > 1 {
        return Err(format!(
            "旧设置文件版本 {} 高于最后支持版本 1",
            file.format_version
        ));
    }
    Ok(normalize_settings(file.settings))
}

fn normalize_settings(mut settings: GameSettings) -> GameSettings {
    settings.render_distance = settings.render_distance.clamp(2, 24);
    settings.master_volume = finite_or(settings.master_volume, 1.0).clamp(0.0, 1.0);
    settings.mouse_sensitivity = finite_or(settings.mouse_sensitivity, 1.0).clamp(0.2, 3.0);
    settings.ui_scale = finite_or(settings.ui_scale, 1.0).clamp(0.6, 1.6);
    settings
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

fn validate_settings_bytes(bytes: &[u8]) -> Result<(), String> {
    decode_settings(bytes).map(|_| ())
}

fn validate_legacy_settings_bytes(bytes: &[u8]) -> Result<(), String> {
    decode_legacy_settings(bytes).map(|_| ())
}

#[cfg(test)]
#[path = "../../../tests/unit/app/settings/persistence.rs"]
mod tests;
