//! 菜单页面共享的局部布局与字体辅助。

use bevy::prelude::*;

use crate::client::ui::resources::ui_font::UiFont;

/// 创建覆盖整个窗口并居中子节点的菜单根布局。
pub(super) fn overlay_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(24.0)),
        ..default()
    }
}

/// 创建菜单标题使用的字体组件。
pub(super) fn title_font(ui_font: &UiFont, size: f32) -> TextFont {
    TextFont {
        font: FontSource::from(ui_font.default.clone()),
        font_size: FontSize::Px(size),
        ..default()
    }
}

/// 创建菜单正文使用的字体组件。
pub(super) fn body_font(ui_font: &UiFont, size: f32) -> TextFont {
    TextFont {
        font: FontSource::from(ui_font.default.clone()),
        font_size: FontSize::Px(size),
        ..default()
    }
}
