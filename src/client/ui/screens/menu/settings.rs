//! 设置页与通用流程对话框的实体构建。

use bevy::prelude::*;

use super::components::{
    DialogCancelButton, DialogConfirmButton, DialogMessage, DialogRoot, DialogTitle, SettingButton,
    SettingValue, SettingsBackButton, SettingsRoot,
};
use super::style::{body_font, overlay_node, title_font};
use crate::app::flow::SettingAction;
use crate::client::ui::resources::frame_assets::UiFrameKind;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::common::{UiControlKind, spawn_text_button};

/// 创建设置页和流程对话框；两者由应用流程资源控制可见性。
pub(super) fn spawn_settings_screens(commands: &mut Commands, theme: &UiTheme, ui_font: &UiFont) {
    spawn_settings(commands, theme, ui_font);
    spawn_dialog(commands, theme, ui_font);
}

fn spawn_settings(commands: &mut Commands, theme: &UiTheme, ui_font: &UiFont) {
    commands
        .spawn((
            SettingsRoot,
            Name::new("SettingsMenu"),
            overlay_node(),
            BackgroundColor(theme.modal_scrim),
            GlobalZIndex(4000),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                UiFrameKind::Generic,
                Node {
                    width: Val::Px(650.0),
                    max_width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(24.0)),
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(theme.bg_panel),
                BorderColor::all(theme.border_default),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("设置"),
                    title_font(ui_font, 28.0),
                    TextColor(theme.text_primary),
                ));
                spawn_setting_row(
                    panel,
                    "画面 / 渲染距离",
                    SettingValue::RenderDistance,
                    SettingAction::RenderDistance(-1),
                    SettingAction::RenderDistance(1),
                    theme,
                    ui_font,
                );
                spawn_setting_row(
                    panel,
                    "音频 / 主音量",
                    SettingValue::MasterVolume,
                    SettingAction::MasterVolume(-0.1),
                    SettingAction::MasterVolume(0.1),
                    theme,
                    ui_font,
                );
                spawn_setting_row(
                    panel,
                    "控制 / 鼠标灵敏度",
                    SettingValue::MouseSensitivity,
                    SettingAction::MouseSensitivity(-0.1),
                    SettingAction::MouseSensitivity(0.1),
                    theme,
                    ui_font,
                );
                spawn_setting_row(
                    panel,
                    "界面 / UI 缩放",
                    SettingValue::UiScale,
                    SettingAction::UiScale(-0.1),
                    SettingAction::UiScale(0.1),
                    theme,
                    ui_font,
                );
                spawn_toggle_row(
                    panel,
                    "画面 / 全屏",
                    SettingValue::Fullscreen,
                    SettingAction::ToggleFullscreen,
                    theme,
                    ui_font,
                );
                spawn_toggle_row(
                    panel,
                    "画面 / 垂直同步",
                    SettingValue::Vsync,
                    SettingAction::ToggleVsync,
                    theme,
                    ui_font,
                );
                spawn_text_button(
                    panel,
                    SettingsBackButton,
                    "返回",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                );
            });
        });
}

fn spawn_setting_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: SettingValue,
    decrease: SettingAction,
    increase: SettingAction,
    theme: &UiTheme,
    ui_font: &UiFont,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(46.0),
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label.to_string()),
                body_font(ui_font, 14.0),
                TextColor(theme.text_secondary),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            spawn_text_button(
                row,
                SettingButton(decrease),
                "-",
                UiControlKind::IconButton,
                theme,
                ui_font,
            );
            row.spawn((
                value,
                Text::new(""),
                body_font(ui_font, 14.0),
                TextColor(theme.text_primary),
                Node {
                    width: Val::Px(92.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));
            spawn_text_button(
                row,
                SettingButton(increase),
                "+",
                UiControlKind::IconButton,
                theme,
                ui_font,
            );
        });
}

fn spawn_toggle_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: SettingValue,
    action: SettingAction,
    theme: &UiTheme,
    ui_font: &UiFont,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(44.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label.to_string()),
                body_font(ui_font, 14.0),
                TextColor(theme.text_secondary),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            row.spawn((
                value,
                Text::new(""),
                body_font(ui_font, 14.0),
                TextColor(theme.text_primary),
                Node {
                    width: Val::Px(72.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));
            spawn_text_button(
                row,
                SettingButton(action),
                "切换",
                UiControlKind::Button,
                theme,
                ui_font,
            );
        });
}

fn spawn_dialog(commands: &mut Commands, theme: &UiTheme, ui_font: &UiFont) {
    commands
        .spawn((
            DialogRoot,
            Name::new("Dialog"),
            overlay_node(),
            BackgroundColor(theme.modal_scrim),
            GlobalZIndex(10_000),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                UiFrameKind::Modal,
                Node {
                    width: Val::Px(520.0),
                    max_width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(24.0)),
                    row_gap: Val::Px(14.0),
                    ..default()
                },
            ))
            .with_children(|panel| {
                panel.spawn((
                    DialogTitle,
                    Text::new("提示"),
                    title_font(ui_font, 24.0),
                    TextColor(theme.text_primary),
                ));
                panel.spawn((
                    DialogMessage,
                    Text::new(""),
                    body_font(ui_font, 15.0),
                    TextColor(theme.text_secondary),
                ));
                panel
                    .spawn(Node {
                        justify_content: JustifyContent::End,
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|actions| {
                        spawn_text_button(
                            actions,
                            DialogCancelButton,
                            "取消",
                            UiControlKind::Button,
                            theme,
                            ui_font,
                        );
                        spawn_text_button(
                            actions,
                            DialogConfirmButton,
                            "确认",
                            UiControlKind::Button,
                            theme,
                            ui_font,
                        );
                    });
            });
        });
}
