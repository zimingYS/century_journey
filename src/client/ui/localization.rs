//! 菜单静态文本的本地化标记与语言切换刷新。

use bevy::prelude::*;

use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::common::{UiControlKind, spawn_text_button};
use crate::engine::localization::Localization;

/// 标记一个静态 `Text` 实体显示某个本地化键。
///
/// 实体创建时按当前语言填充初始文本；语言切换后由
/// [`refresh_localized_text_system`] 统一重写。动态文本
/// （如设置值、世界名）由各自的同步系统渲染，不使用本组件。
///
/// 标记支持两种布局：与 `Text` 同实体（纯标签），或位于组合控件
/// 父实体、`Text` 在其子实体上（按钮）。
#[derive(Component, Debug, Clone)]
pub struct LocalizedText {
    /// 点号分隔的翻译键。
    pub key: String,
    /// 键缺失时的兜底文本；数据驱动的动态键（如创造分类）使用。
    pub fallback: Option<String>,
}

impl LocalizedText {
    /// 用翻译键创建标记。
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            fallback: None,
        }
    }

    /// 用翻译键与兜底文本创建标记；查不到键时显示兜底而非键名。
    pub fn with_fallback(key: impl Into<String>, fallback: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            fallback: Some(fallback.into()),
        }
    }
}

/// 生成 `(LocalizedText, Text)` 初始组件对；语言切换后由刷新系统维护。
pub fn localized_text(key: &str, localization: &Localization) -> (LocalizedText, Text) {
    (LocalizedText::new(key), Text::new(localization.get(key)))
}

/// 语言变化后重写全部静态本地化文本。
///
/// 标记与 `Text` 同实体时直接重写；标记在按钮等组合控件父实体上时，
/// 重写其子实体上的文案。`Text` 只保留一个可变查询避免访问冲突。
pub fn refresh_localized_text_system(
    localization: Res<Localization>,
    markers: Query<(Entity, &LocalizedText, Option<&Children>)>,
    mut texts: Query<&mut Text>,
) {
    if !localization.is_changed() {
        return;
    }
    for (entity, tag, children) in &markers {
        let text = match tag.fallback.as_deref() {
            Some(fallback) => localization.get_or(&tag.key, fallback),
            None => localization.get(&tag.key),
        };
        if let Ok(mut t) = texts.get_mut(entity) {
            *t = Text::new(text);
        }
        if let Some(children) = children {
            for child in children {
                if let Ok(mut t) = texts.get_mut(*child) {
                    *t = Text::new(text);
                }
            }
        }
    }
}

/// 生成带本地化标记的文本按钮：初始取当前语言，切换语言时自动刷新。
pub fn spawn_localized_button<M: Bundle>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    key: &str,
    kind: UiControlKind,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) -> Entity {
    let entity = spawn_text_button(parent, marker, localization.get(key), kind, theme, ui_font);
    parent
        .commands()
        .entity(entity)
        .insert(LocalizedText::new(key));
    entity
}

#[cfg(test)]
#[path = "../../../tests/unit/client/ui/localization.rs"]
mod tests;
