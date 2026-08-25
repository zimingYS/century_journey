//! 构建并更新界面使用的标签页切换控件。

use crate::client::ui::localization::LocalizedText;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::common::{UiControl, UiControlKind};
use crate::client::ui::widgets::slot::CategoryTab;
use crate::engine::localization::Localization;
use crate::game::inventory::container::creative::CreativeCategory;
use bevy::prelude::*;

const CREATIVE_TAB_HEIGHT: f32 = 46.0;
const CREATIVE_TAB_FONT_SIZE: f32 = 16.0;

/// 生成创造物品栏左侧分类按钮。
///
/// 分类名来自本地化键；数据驱动标签可能没有对应键，此时显示兜底名。
pub fn spawn_category_tab(
    parent: &mut ChildSpawnerCommands,
    category: &CreativeCategory,
    category_index: usize,
    is_active: bool,
    ui_font: &UiFont,
    theme: &UiTheme,
    localization: &Localization,
) {
    let text_color = if is_active {
        theme.text_primary
    } else {
        theme.tab_inactive_text
    };
    let icon_label = if category.icon.is_empty() {
        "□"
    } else {
        category.icon.as_str()
    };

    parent
        .spawn((
            CategoryTab { category_index },
            UiControl {
                kind: UiControlKind::Tab,
                selected: is_active,
                disabled: false,
            },
            Button,
            Pickable::default(),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(CREATIVE_TAB_HEIGHT),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                column_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(if is_active {
                theme.accent
            } else {
                theme.tab_active_bg
            }),
            BorderColor::all(if is_active {
                theme.border_selected
            } else {
                theme.border_default
            }),
        ))
        .with_children(|btn| {
            // 分类图标使用文本符号，后续可替换成真实物品图标。
            btn.spawn((
                Text::new(icon_label.to_string()),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(CREATIVE_TAB_FONT_SIZE + 4.0),
                    ..default()
                },
                TextColor(Color::srgba(0.84, 0.84, 0.82, 1.0)),
                Node {
                    width: Val::Px(24.0),
                    ..default()
                },
            ));
            btn.spawn((
                LocalizedText::with_fallback(
                    category.label_key.as_str(),
                    category.label_fallback.as_str(),
                ),
                Text::new(localization.get_or(
                    category.label_key.as_str(),
                    category.label_fallback.as_str(),
                )),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(CREATIVE_TAB_FONT_SIZE),
                    ..default()
                },
                TextColor(text_color),
            ));
        });
}
