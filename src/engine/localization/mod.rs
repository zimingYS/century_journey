//! 本地化基础设施：语言文件加载、翻译查询与回退链。
//!
//! 语言资源位于 `assets/locales/`，每个 `.toml` 文件自描述一种语言
//! （顶层 `language` 与 `native-name` 元数据 + 分节译文表）。新增语言
//! 只需放置新文件，无需修改代码。查询回退链为
//! 「激活语言 -> 简体中文 -> 键本身」。

mod loader;
mod store;

pub use loader::{
    LOCALES_DIR, LanguageFile, build_localization, load_locales_from_dir, parse_locale_toml,
};
pub use store::{FALLBACK_LANGUAGE, LanguageId, LanguageInfo, Localization};

use std::path::Path;

use bevy::prelude::*;

/// 注册语言加载与查询资源。
pub struct LocalizationPlugin;

impl Plugin for LocalizationPlugin {
    fn build(&self, app: &mut App) {
        // Bevy 的初始状态 OnEnter（如 Boot）先于 PreStartup 触发，
        // 因此在插件构建期同步加载语言，保证任何调度阶段都能查询译文。
        app.insert_resource(load_localization());
    }
}

/// 从语言目录加载全部语言并构建查询资源。
///
/// 加载失败时返回空资源并告警：查询退化为返回键本身，
/// 界面构建不被阻断，漏翻条目在屏幕上直接可见。
fn load_localization() -> Localization {
    match load_locales_from_dir(Path::new(LOCALES_DIR)) {
        Ok(files) if files.is_empty() => {
            log::warn!("[本地化] 语言目录 {LOCALES_DIR} 为空，界面文本将显示为键名");
            build_localization(Vec::new())
        }
        Ok(files) => build_localization(files),
        Err(error) => {
            log::warn!("[本地化] {error}，界面文本将显示为键名");
            build_localization(Vec::new())
        }
    }
}
