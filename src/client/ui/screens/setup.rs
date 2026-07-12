use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui_widgets::SelectAllOnFocus;

use crate::client::ui::components::{
    CreativeCategoryPanel, CreativeCloseButton, CreativeHotbarPanel, CreativeInventoryOverlay,
    CreativeInventoryRoot, CreativeItemGrid, CreativeRecentPanel, CreativeSearchBox,
    CreativeSearchPlaceholder, CreativeTitleIcon,
};
use crate::client::ui::navigation::{UiScreenAudience, UiScreenRoot};
use crate::client::ui::resources::frame_assets::UiFrameKind;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::CreativeSearchInput;

/// 创造物品栏位于 HUD 之上，避免被底部快捷栏盖住。
const CREATIVE_OVERLAY_Z: i32 = 1000;
/// 面板上限与内部尺寸共同保证 9 列槽位能够等宽排布。
const CREATIVE_PANEL_WIDTH: f32 = 1124.0;
const CREATIVE_PANEL_HEIGHT: f32 = 680.0;
const CREATIVE_SIDEBAR_WIDTH: f32 = 160.0;
const CREATIVE_RECENT_WIDTH: f32 = 164.0;
const CREATIVE_GRID_COLUMNS: u16 = 9;
const CREATIVE_SLOT_SIZE: f32 = 74.0;
const CREATIVE_SLOT_GAP: f32 = 6.0;
const CREATIVE_GRID_PADDING: f32 = 12.0;
const CREATIVE_HOTBAR_HEIGHT: f32 = 92.0;

/// 构造创造模式物品栏 UI。
pub fn spawn_creative_inventory_system(
    mut commands: Commands,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
) {
    commands
        .spawn((
            CreativeInventoryOverlay,
            UiScreenRoot::inventory(UiScreenAudience::Creative),
            Name::new("CreativeOverlay"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(18.0)),
                ..default()
            },
            // 明确提高创造物品栏层级，保证它渲染在 HUD 快捷栏之上。
            ZIndex(CREATIVE_OVERLAY_Z),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.46)),
            Visibility::Hidden,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    CreativeInventoryRoot,
                    UiFrameKind::Creative,
                    Name::new("CreativeRoot"),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        max_width: Val::Px(CREATIVE_PANEL_WIDTH),
                        max_height: Val::Px(CREATIVE_PANEL_HEIGHT),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.055, 0.058, 0.062, 0.98)),
                    BorderColor::all(Color::srgba(0.52, 0.43, 0.32, 1.0)),
                ))
                .with_children(|root| {
                    build_header(root, &ui_font, &theme);
                    build_inventory_frame(root, &ui_font, &theme);
                });
        });
}

/// 构造标题栏、搜索框与关闭按钮。
fn build_header(root: &mut ChildSpawnerCommands, ui_font: &UiFont, theme: &UiTheme) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(60.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(14.0)),
            border: UiRect::all(Val::Px(1.0)),
            column_gap: Val::Px(16.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.075, 0.078, 0.084, 0.96)),
        BorderColor::all(Color::srgba(0.18, 0.18, 0.19, 1.0)),
    ))
    .with_children(|header| {
        header
            .spawn((Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..default()
            },))
            .with_children(|title| {
                title
                    .spawn((
                        CreativeTitleIcon,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.11, 0.16, 0.11, 1.0)),
                        BorderColor::all(Color::srgba(0.32, 0.38, 0.30, 1.0)),
                    ))
                    .with_children(|icon| {
                        icon.spawn((
                            Text::new("■"),
                            TextFont {
                                font: FontSource::from(ui_font.default.clone()),
                                font_size: FontSize::Px(24.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.42, 0.78, 0.25)),
                        ));
                    });
                title.spawn((
                    Text::new("创造模式"),
                    TextFont {
                        font: FontSource::from(ui_font.default.clone()),
                        font_size: FontSize::Px(28.0),
                        ..default()
                    },
                    TextColor(theme.text_primary),
                ));
            });

        header
            .spawn((Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(18.0),
                ..default()
            },))
            .with_children(|right| {
                build_search_box(right, ui_font, theme);
                build_close_button(right, ui_font);
            });
    });
}

/// 构造搜索框。占位文字是单独节点，不会污染真实搜索文本。
fn build_search_box(parent: &mut ChildSpawnerCommands, ui_font: &UiFont, theme: &UiTheme) {
    parent
        .spawn((
            CreativeSearchBox,
            Name::new("CreativeSearchBox"),
            Node {
                width: Val::Px(320.0),
                height: Val::Px(40.0),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                column_gap: Val::Px(10.0),
                overflow: Overflow::clip_x(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.035, 0.037, 0.041, 0.96)),
            BorderColor::all(Color::srgba(0.23, 0.23, 0.24, 1.0)),
        ))
        .with_children(|search| {
            search.spawn((
                Text::new("⌕"),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(28.0),
                    ..default()
                },
                TextColor(Color::srgba(0.72, 0.72, 0.72, 1.0)),
                Pickable::IGNORE,
            ));
            search.spawn((
                CreativeSearchPlaceholder,
                Text::new("搜索物品..."),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(theme.search_font_size + 3.0),
                    ..default()
                },
                TextColor(Color::srgba(0.58, 0.58, 0.60, 1.0)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(56.0),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            search.spawn((
                CreativeSearchInput,
                Name::new("CreativeSearchInput"),
                EditableText {
                    visible_width: Some(18.0),
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
                    font_size: FontSize::Px(theme.search_font_size + 3.0),
                    ..default()
                },
                TextColor(theme.text_primary),
                Node {
                    flex_grow: 1.0,
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    overflow: Overflow::clip_x(),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));
        });
}

/// 构造右上角关闭按钮。
fn build_close_button(parent: &mut ChildSpawnerCommands, ui_font: &UiFont) {
    parent
        .spawn((
            CreativeCloseButton,
            Button,
            Pickable::default(),
            Node {
                width: Val::Px(40.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.42, 0.39, 0.34, 1.0)),
            BorderColor::all(Color::srgba(0.12, 0.12, 0.12, 1.0)),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new("×"),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(28.0),
                    ..default()
                },
                TextColor(Color::BLACK),
            ));
        });
}

/// 构造三栏主体：左分类、中间物品、右最近使用。
fn build_inventory_frame(root: &mut ChildSpawnerCommands, ui_font: &UiFont, theme: &UiTheme) {
    root.spawn((Node {
        width: Val::Percent(100.0),
        height: Val::Px(0.0),
        min_height: Val::Px(0.0),
        flex_grow: 1.0,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Stretch,
        column_gap: Val::Px(8.0),
        overflow: Overflow::clip_y(),
        ..default()
    },))
        .with_children(|frame| {
            build_category_sidebar(frame, ui_font, theme);
            build_center_panel(frame, theme);
            build_recent_panel(frame, ui_font, theme);
        });
}

/// 构造左侧分类栏和页码区。
fn build_category_sidebar(parent: &mut ChildSpawnerCommands, ui_font: &UiFont, theme: &UiTheme) {
    parent
        .spawn((
            Node {
                width: Val::Px(CREATIVE_SIDEBAR_WIDTH),
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                row_gap: Val::Px(6.0),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.065, 0.068, 0.072, 0.98)),
            BorderColor::all(Color::srgba(0.18, 0.19, 0.20, 1.0)),
        ))
        .with_children(|sidebar| {
            sidebar.spawn((
                CreativeCategoryPanel,
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(0.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
            ));

            sidebar
                .spawn((Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(52.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    border: UiRect::top(Val::Px(1.0)),
                    padding: UiRect::top(Val::Px(8.0)),
                    ..default()
                },))
                .with_children(|pager| {
                    build_pager_button(pager, ui_font, "‹");
                    pager.spawn((
                        Text::new("1 / 1"),
                        TextFont {
                            font: FontSource::from(ui_font.default.clone()),
                            font_size: FontSize::Px(theme.body_font_size + 4.0),
                            ..default()
                        },
                        TextColor(theme.text_primary),
                    ));
                    build_pager_button(pager, ui_font, "›");
                });
        });
}

/// 构造页码箭头，暂时只作为视觉占位。
fn build_pager_button(parent: &mut ChildSpawnerCommands, ui_font: &UiFont, label: &str) {
    parent
        .spawn((
            Node {
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.16, 0.16, 0.16, 1.0)),
            BorderColor::all(Color::srgba(0.32, 0.32, 0.32, 1.0)),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::srgba(0.78, 0.78, 0.78, 1.0)),
            ));
        });
}

/// 构造中间物品网格和面板内快捷栏。
fn build_center_panel(parent: &mut ChildSpawnerCommands, theme: &UiTheme) {
    parent
        .spawn((
            Node {
                min_height: Val::Px(0.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                row_gap: Val::Px(8.0),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.075, 0.079, 0.083, 0.98)),
            BorderColor::all(Color::srgba(0.18, 0.22, 0.24, 1.0)),
        ))
        .with_children(|center| {
            center.spawn((
                CreativeItemGrid,
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(0.0),
                    flex_grow: 1.0,
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(CREATIVE_GRID_COLUMNS, 1.0),
                    grid_auto_rows: vec![GridTrack::px(CREATIVE_SLOT_SIZE)],
                    column_gap: Val::Px(CREATIVE_SLOT_GAP),
                    row_gap: Val::Px(CREATIVE_SLOT_GAP),
                    padding: UiRect::all(Val::Px(CREATIVE_GRID_PADDING)),
                    overflow: Overflow::scroll_y(),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.09, 0.095, 0.10, 1.0)),
                BorderColor::all(Color::srgba(0.16, 0.18, 0.19, 1.0)),
            ));
            build_hotbar_panel(center, theme);
        });
}

/// 构造右侧最近使用栏。槽位由同步系统固定补齐为空槽。
fn build_recent_panel(parent: &mut ChildSpawnerCommands, _ui_font: &UiFont, _theme: &UiTheme) {
    parent.spawn((
        CreativeRecentPanel,
        Node {
            width: Val::Px(CREATIVE_RECENT_WIDTH),
            min_height: Val::Px(0.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::FlexStart,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(2.0)),
            column_gap: Val::Px(6.0),
            row_gap: Val::Px(6.0),
            overflow: Overflow::clip_y(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.065, 0.068, 0.072, 0.98)),
        BorderColor::all(Color::srgba(0.18, 0.19, 0.20, 1.0)),
    ));
}

/// 构造创造物品栏内部快捷栏。
fn build_hotbar_panel(parent: &mut ChildSpawnerCommands, _theme: &UiTheme) {
    parent.spawn((
        CreativeHotbarPanel,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(CREATIVE_HOTBAR_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::top(Val::Px(1.0)),
            padding: UiRect::vertical(Val::Px(8.0)),
            column_gap: Val::Px(4.0),
            ..default()
        },
        BorderColor::all(Color::srgba(0.20, 0.20, 0.20, 1.0)),
    ));
}
