//! 加载界面、主菜单与暂停菜单的实体构建。

use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui_widgets::SelectAllOnFocus;

use super::components::{
    CreateButton, DeleteButton, LoadingDetail, LoadingTitle, MainSettingsButton,
    PauseSettingsButton, PlayButton, QuitButton, ResumeButton, SaveQuitButton, WorldList,
    WorldNameInput,
};
use super::settings::spawn_settings_screens;
use super::style::{body_font, overlay_node, title_font};
use crate::client::ui::navigation::{UiScreen, UiScreenRoot};
use crate::client::ui::resources::frame_assets::UiFrameKind;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::common::{UiControlKind, UiScrollArea, spawn_text_button};

/// 在启动阶段一次性创建菜单相关界面；后续系统只切换可见性并同步文本。
pub(crate) fn spawn_menu_screens_system(
    mut commands: Commands,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
) {
    spawn_loading_screen(&mut commands, &theme, &ui_font);
    spawn_main_menu(&mut commands, &theme, &ui_font);
    spawn_pause_menu(&mut commands, &theme, &ui_font);
    spawn_settings_screens(&mut commands, &theme, &ui_font);
}

fn spawn_loading_screen(commands: &mut Commands, theme: &UiTheme, ui_font: &UiFont) {
    commands
        .spawn((
            UiScreenRoot::new(UiScreen::Loading),
            Name::new("LoadingScreen"),
            overlay_node(),
            BackgroundColor(Color::srgb(0.025, 0.03, 0.035)),
            GlobalZIndex(3000),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                UiFrameKind::Generic,
                Node {
                    width: Val::Px(520.0),
                    min_height: Val::Px(180.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(28.0)),
                    row_gap: Val::Px(14.0),
                    ..default()
                },
            ))
            .with_children(|panel| {
                panel.spawn((
                    LoadingTitle,
                    Text::new("正在启动"),
                    title_font(ui_font, 28.0),
                    TextColor(theme.text_primary),
                ));
                panel.spawn((
                    LoadingDetail,
                    Text::new("正在加载内容资源..."),
                    body_font(ui_font, 15.0),
                    TextColor(theme.text_secondary),
                ));
            });
        });
}

fn spawn_main_menu(commands: &mut Commands, theme: &UiTheme, ui_font: &UiFont) {
    commands
        .spawn((
            UiScreenRoot::new(UiScreen::MainMenu),
            Name::new("MainMenu"),
            overlay_node(),
            BackgroundColor(Color::srgba(0.02, 0.025, 0.03, 0.97)),
            GlobalZIndex(2000),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                UiFrameKind::Generic,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    max_width: Val::Px(1080.0),
                    max_height: Val::Px(700.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(22.0)),
                    row_gap: Val::Px(16.0),
                    ..default()
                },
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("CENTURY JOURNEY"),
                    title_font(ui_font, 34.0),
                    TextColor(theme.text_primary),
                ));
                panel.spawn((
                    Text::new("世界"),
                    title_font(ui_font, 20.0),
                    TextColor(theme.text_secondary),
                ));
                panel
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(0.0),
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(14.0),
                        ..default()
                    })
                    .with_children(|body| {
                        body.spawn((
                            WorldList,
                            UiScrollArea,
                            Interaction::None,
                            Pickable::default(),
                            ScrollPosition::default(),
                            Node {
                                width: Val::Percent(64.0),
                                min_height: Val::Px(0.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(10.0)),
                                row_gap: Val::Px(6.0),
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            BackgroundColor(theme.bg_content),
                            BorderColor::all(theme.border_default),
                        ));
                        body.spawn((Node {
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },))
                            .with_children(|actions| {
                                actions.spawn((
                                    Text::new("新世界名称"),
                                    body_font(ui_font, 14.0),
                                    TextColor(theme.text_secondary),
                                ));
                                actions.spawn((
                                    WorldNameInput,
                                    EditableText::new("new_world"),
                                    TextCursorStyle::default(),
                                    SelectAllOnFocus,
                                    TextLayout::no_wrap(),
                                    body_font(ui_font, 15.0),
                                    TextColor(theme.text_primary),
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(42.0),
                                        padding: UiRect::all(Val::Px(10.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        overflow: Overflow::clip_x(),
                                        ..default()
                                    },
                                    BackgroundColor(theme.search_bg),
                                    BorderColor::all(theme.search_border),
                                ));
                                spawn_text_button(
                                    actions,
                                    CreateButton,
                                    "创建世界",
                                    UiControlKind::Button,
                                    theme,
                                    ui_font,
                                );
                                spawn_text_button(
                                    actions,
                                    PlayButton,
                                    "进入世界",
                                    UiControlKind::Button,
                                    theme,
                                    ui_font,
                                );
                                spawn_text_button(
                                    actions,
                                    DeleteButton,
                                    "删除世界",
                                    UiControlKind::Button,
                                    theme,
                                    ui_font,
                                );
                                actions.spawn(Node {
                                    flex_grow: 1.0,
                                    ..default()
                                });
                                spawn_text_button(
                                    actions,
                                    MainSettingsButton,
                                    "设置",
                                    UiControlKind::Button,
                                    theme,
                                    ui_font,
                                );
                                spawn_text_button(
                                    actions,
                                    QuitButton,
                                    "退出游戏",
                                    UiControlKind::Button,
                                    theme,
                                    ui_font,
                                );
                            });
                    });
            });
        });
}

fn spawn_pause_menu(commands: &mut Commands, theme: &UiTheme, ui_font: &UiFont) {
    commands
        .spawn((
            UiScreenRoot::new(UiScreen::PauseMenu),
            Name::new("PauseMenu"),
            overlay_node(),
            BackgroundColor(theme.modal_scrim),
            GlobalZIndex(2500),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                UiFrameKind::Generic,
                Node {
                    width: Val::Px(420.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(24.0)),
                    row_gap: Val::Px(12.0),
                    ..default()
                },
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("游戏已暂停"),
                    title_font(ui_font, 28.0),
                    TextColor(theme.text_primary),
                ));
                spawn_text_button(
                    panel,
                    ResumeButton,
                    "继续游戏",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                );
                spawn_text_button(
                    panel,
                    PauseSettingsButton,
                    "设置",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                );
                spawn_text_button(
                    panel,
                    SaveQuitButton,
                    "保存并退出",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                );
            });
        });
}
