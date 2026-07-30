//! 设置文件的版本、校验、备份与迁移。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::model::GameSettings;
use crate::engine::persistence;

/// 当前支持的设置文件格式版本。
pub const SETTINGS_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct SettingsFile {
    format_version: u32,
    game_version: String,
    settings: GameSettings,
}

/// 返回默认设置文件路径。
pub fn settings_path() -> PathBuf {
    PathBuf::from("config").join("settings.json")
}

/// 从默认路径读取并校验设置。
pub fn load_settings() -> Result<GameSettings, String> {
    load_settings_from(&settings_path())
}

/// 将设置原子写入默认路径，并保留可恢复备份。
pub fn save_settings(settings: &GameSettings) -> Result<(), String> {
    save_settings_to(&settings_path(), settings)
}

/// 从指定路径读取设置，供运行时和持久化测试复用。
pub fn load_settings_from(path: &Path) -> Result<GameSettings, String> {
    let bytes = persistence::read_verified(path, validate_settings_bytes)
        .map_err(|error| error.to_string())?;
    decode_settings(&bytes)
}

/// 将设置原子写入指定路径，供运行时和持久化测试复用。
pub fn save_settings_to(path: &Path, settings: &GameSettings) -> Result<(), String> {
    let file = SettingsFile {
        format_version: SETTINGS_FORMAT_VERSION,
        game_version: env!("CARGO_PKG_VERSION").to_string(),
        settings: normalize_settings(settings.clone()),
    };
    let bytes =
        serde_json::to_vec_pretty(&file).map_err(|error| format!("设置序列化失败: {error}"))?;
    persistence::atomic_write_verified(path, &bytes, validate_settings_bytes)
        .map_err(|error| error.to_string())
}

/// 判断默认设置文件是否存在通过校验的备份。
pub fn settings_backup_available() -> bool {
    persistence::has_valid_backup(&settings_path(), validate_settings_bytes)
}

/// 使用有效备份恢复默认设置文件。
pub fn restore_settings_backup() -> Result<(), String> {
    persistence::restore_backup(&settings_path(), validate_settings_bytes)
        .map_err(|error| error.to_string())
}

fn decode_settings(bytes: &[u8]) -> Result<GameSettings, String> {
    let file: SettingsFile =
        serde_json::from_slice(bytes).map_err(|error| format!("设置文件格式无效: {error}"))?;
    migrate_settings(file)
}

/// 所有格式升级都从这里进入，避免版本判断散落在读写代码中。
fn migrate_settings(mut file: SettingsFile) -> Result<GameSettings, String> {
    match file.format_version {
        0 => {
            file.format_version = SETTINGS_FORMAT_VERSION;
            file.game_version = env!("CARGO_PKG_VERSION").to_string();
        }
        SETTINGS_FORMAT_VERSION => {}
        found => {
            return Err(format!(
                "设置文件版本 {found} 高于当前支持版本 {SETTINGS_FORMAT_VERSION}"
            ));
        }
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

#[cfg(test)]
#[path = "../../../tests/unit/app/settings/persistence.rs"]
mod tests;
