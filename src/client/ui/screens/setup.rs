use bevy::prelude::*;

use crate::client::ui::components::{
    CreativeCategoryPanel, CreativeHotbarPanel, CreativeInventoryOverlay,
    CreativeInventoryRoot, CreativeItemGrid, CreativeRecentPanel, CreativeSearchBox,
};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::CreativeSearchInput;

/// 构造创造模式物品栏UI
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
    )).with_children(|header| {
        header.spawn((
            Text::new("创造模式"),
            TextFont {
                font: FontSource::from(ui_font.default.clone()),
                font_size: FontSize::Px(theme.title_font_size),
                ..default()
            },
            TextColor(theme.text_primary),
        ));
        header
            .spawn((
                CreativeSearchBox,
                CreativeSearchInput,
                Button,
                Pickable::default(),
                Node {
                    width: Val::Px(theme.search_width),
                    height: Val::Px(theme.search_height),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(theme.search_bg),
                BorderColor::all(theme.search_border),
            )).with_children(|s| {
                s.spawn((
                    Text::new(""),
                    TextFont {
                        font: FontSource::from(ui_font.default.clone()),
                        font_size: FontSize::Px(theme.search_font_size),
                        ..default()
                    },
                    TextColor(theme.text_hint),
                ));
            });
        });
}

fn build_body(root: &mut ChildSpawnerCommands, _ui_font: &UiFont, theme: &UiTheme) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    ))
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
                    grid_template_columns: RepeatedGridTrack::flex(
                        theme.grid_columns as u16,
                        1.0,
                    ),
                    grid_auto_rows: vec![GridTrack::px(
                        theme.slot_size + theme.slot_gap,
                    )],
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
