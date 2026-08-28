//! 构建并更新界面使用的标签页切换控件。

use crate::client::ui::components::{CreativeTabIcon, CreativeTabLabel};
use crate::client::ui::localization::LocalizedText;
use crate::client::ui::resources::creative_assets::{
    CreativeUiAssets, TAB_SLICE, sliced_image_node,
};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::common::{UiControl, UiControlKind};
use crate::client::ui::widgets::slot::CategoryTab;
use crate::engine::localization::Localization;
use crate::game::inventory::container::creative::CreativeCategory;
use bevy::prelude::*;

const CREATIVE_TAB_HEIGHT: f32 = 53.0;
const CREATIVE_TAB_FONT_SIZE: f32 = 16.0;
const CREATIVE_TAB_ICON_SIZE: f32 = 32.0;

/// 生成创造物品栏左侧分类按钮。
///
/// 分类名来自本地化键；数据驱动标签可能没有对应键，此时显示兜底名。
/// 按钮背景为标签九宫格皮肤，选中态在生成时即选定初始纹理，
/// 后续由高亮系统随选中状态切换。
#[allow(clippy::too_many_arguments)]
pub fn spawn_category_tab(
    parent: &mut ChildSpawnerCommands,
    category: &CreativeCategory,
    category_index: usize,
    is_active: bool,
    ui_font: &UiFont,
    theme: &UiTheme,
    localization: &Localization,
    assets: &CreativeUiAssets,
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
                position_type: PositionType::Relative,
                width: Val::Percent(100.0),
                height: Val::Px(CREATIVE_TAB_HEIGHT),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                padding: UiRect::left(Val::Px(10.0)),
                column_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            sliced_image_node(
                if is_active {
                    assets.tab_active.clone()
                } else {
                    assets.tab_inactive.clone()
                },
                TAB_SLICE,
            ),
        ))
        .with_children(|btn| {
            btn.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                sliced_image_node(
                    if is_active {
                        assets.tab_active.clone()
                    } else {
                        assets.tab_inactive.clone()
                    },
                    TAB_SLICE,
                ),
                Pickable::IGNORE,
            ));
            // 图标节点先占位；代表物品的 3D 图标由皮肤系统待渲染资源就绪后填充，
            // 未配置代表物品的分类保留原有文字符号。
            if category_icon_item_path(category).is_some() {
                btn.spawn((
                    CreativeTabIcon { category_index },
                    Node {
                        width: Val::Px(CREATIVE_TAB_ICON_SIZE),
                        height: Val::Px(CREATIVE_TAB_ICON_SIZE),
                        ..default()
                    },
                ));
            } else {
                btn.spawn((
                    Text::new(icon_label.to_string()),
                    TextFont {
                        font: FontSource::from(ui_font.default.clone()),
                        font_size: FontSize::Px(CREATIVE_TAB_FONT_SIZE + 4.0),
                        ..default()
                    },
                    TextColor(Color::srgba(0.84, 0.84, 0.82, 1.0)),
                    Node {
                        width: Val::Px(CREATIVE_TAB_ICON_SIZE),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            }
            btn.spawn((
                CreativeTabLabel,
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
                Pickable::IGNORE,
            ));
        });
}

/// 分类标签图标对应的代表物品路径表。
///
/// 键为分类本地化键，值为默认命名空间下的物品路径；
/// 未收录的分类沿用文字符号图标，内容扩充后补条目即可。
const CATEGORY_ICON_ITEMS: &[(&str, &str)] = &[
    ("creative.category.all", "grass"),
    ("creative.category.solid", "stone"),
    ("creative.category.crop", "sapling"),
    ("creative.category.natural", "leaves"),
    ("creative.category.tools", "iron_pickaxe"),
    ("creative.category.decor", "glowstone"),
    ("creative.category.misc", "water_bottle"),
    ("creative.category.favorites", "apple"),
];

/// 查询分类对应的代表物品路径；未映射时返回 `None`。
pub fn category_icon_item_path(category: &CreativeCategory) -> Option<&'static str> {
    CATEGORY_ICON_ITEMS
        .iter()
        .find(|(key, _)| *key == category.label_key.as_str())
        .map(|(_, path)| *path)
}

/// 生成分类标签之间的横向分隔线。
pub fn spawn_category_separator(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Name::new("CreativeCategorySeparator"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(3.0),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(Color::srgba(0.25, 0.25, 0.26, 0.65)),
        Pickable::IGNORE,
    ));
}
