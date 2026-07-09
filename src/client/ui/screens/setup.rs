use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui_widgets::SelectAllOnFocus;

use crate::client::ui::components::{
    CreativeCategoryPanel, CreativeHotbarPanel, CreativeInventoryOverlay, CreativeInventoryRoot,
    CreativeItemGrid, CreativeRecentPanel, CreativeSearchBox,
};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::CreativeSearchInput;

/// 构造创造模式物品栏 UI。
pub fn spawn_creative_inventory_system(
    mut commands: Commands,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
) {
    commands
        .spawn((
            CreativeInventoryOverlay,
            Name::new("CreativeOverlay"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            Visibility::Hidden,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    CreativeInventoryRoot,
                    Name::new("CreativeRoot"),
                    Node {
                        width: Val::Px(theme.panel_width),
                        height: Val::Px(theme.panel_height),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(theme.bg_panel),
                    BorderColor::all(theme.border_default),
                ))
                .with_children(|root| {
                    build_header(root, &ui_font, &theme);
                    build_body(root, &ui_font, &theme);
                    build_recent_panel(root, &ui_font, &theme);
                    build_hotbar_panel(root, &theme);
                });
        });
}

/// 构造标题栏和 Bevy 0.19 EditableText 搜索框。
fn build_header(root: &mut ChildSpawnerCommands, ui_font: &UiFont, theme: &UiTheme) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(theme.panel_header_h),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(theme.panel_padding)),
            border: UiRect::bottom(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(theme.border_default),
    ))
    .with_children(|header| {
        header.spawn((
            Text::new("创造模式"),
            TextFont {
                font: FontSource::from(ui_font.default.clone()),
                font_size: FontSize::Px(theme.title_font_size),
                ..default()
            },
            TextColor(theme.text_primary),
        ));
        header.spawn((
            CreativeSearchBox,
            CreativeSearchInput,
            Name::new("CreativeSearchInput"),
            EditableText {
                visible_width: Some(16.0),
                max_characters: Some(64),
                allow_newlines: false,
                ..default()
            },
            TextCursorStyle {
                color: theme.text_primary,
                selection_color: theme.border_selected,
                unfocused_selection_color: theme.border_hover,
                selected_text_color: Some(Color::BLACK),
            },
            SelectAllOnFocus,
            TextLayout::no_wrap(),
            TextFont {
                font: FontSource::from(ui_font.default.clone()),
                font_size: FontSize::Px(theme.search_font_size),
                ..default()
            },
            TextColor(theme.text_primary),
            Node {
                width: Val::Px(theme.search_width),
                height: Val::Px(theme.search_height),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip_x(),
                ..default()
            },
            BackgroundColor(theme.search_bg),
            BorderColor::all(theme.search_border),
        ));
    });
}

/// 构造分类栏和物品网格。
fn build_body(root: &mut ChildSpawnerCommands, _ui_font: &UiFont, theme: &UiTheme) {
    root.spawn((Node {
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        flex_direction: FlexDirection::Row,
        ..default()
    },))
        .with_children(|body| {
            body.spawn((
                CreativeCategoryPanel,
                Node {
                    width: Val::Px(theme.tab_sidebar_width),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::right(Val::Px(1.0)),
                    padding: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(theme.bg_sidebar),
                BorderColor::all(theme.border_default),
            ));

            body.spawn((
                CreativeItemGrid,
                Node {
                    flex_grow: 1.0,
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(theme.grid_columns as u16, 1.0),
                    grid_auto_rows: vec![GridTrack::px(theme.slot_size + theme.slot_gap)],
                    column_gap: Val::Px(theme.slot_gap),
                    row_gap: Val::Px(theme.slot_gap),
                    padding: UiRect::all(Val::Px(theme.grid_padding)),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(theme.bg_content),
            ));
        });
}

/// 构造最近使用面板。
fn build_recent_panel(root: &mut ChildSpawnerCommands, ui_font: &UiFont, theme: &UiTheme) {
    root.spawn((
        CreativeRecentPanel,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(theme.recent_height),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(theme.panel_padding)),
            border: UiRect::top(Val::Px(1.0)),
            column_gap: Val::Px(theme.slot_gap),
            ..default()
        },
        BorderColor::all(theme.border_default),
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new("最近使用:"),
            TextFont {
                font: FontSource::from(ui_font.default.clone()),
                font_size: FontSize::Px(theme.small_font_size),
                ..default()
            },
            TextColor(theme.text_secondary),
            Node {
                margin: UiRect::right(Val::Px(8.0)),
                ..default()
            },
        ));
    });
}

/// 构造底部快捷栏面板。
fn build_hotbar_panel(root: &mut ChildSpawnerCommands, theme: &UiTheme) {
    root.spawn((
        CreativeHotbarPanel,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(theme.creative_hotbar_h),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::top(Val::Px(1.0)),
            padding: UiRect::vertical(Val::Px(6.0)),
            column_gap: Val::Px(4.0),
            ..default()
        },
        BorderColor::all(theme.border_default),
    ));
}
