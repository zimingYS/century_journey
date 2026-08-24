//! 设置页与通用流程对话框的实体构建。

use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle, TextLayout};

use super::components::{
    DialogCancelButton, DialogConfirmButton, DialogMessage, DialogRoot, DialogTitle, KeybindFilter,
    KeybindFilterButton, KeybindList, KeybindResetButton, KeybindSearchInput, SettingButton,
    SettingValue, SettingsBackButton, SettingsGeneralPage, SettingsKeybindsPage, SettingsRoot,
    SettingsTab, SettingsTabButton,
};
use super::style::{body_font, overlay_node, title_font};
use crate::app::flow::SettingAction;
use crate::client::ui::resources::frame_assets::UiFrameKind;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::common::{
    UiControl, UiControlKind, spawn_scroll_area, spawn_text_button,
};

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
                    width: Val::Px(720.0),
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
                spawn_settings_tab_bar(panel, theme, ui_font);
                spawn_general_page(panel, theme, ui_font);
                spawn_keybinds_page(panel, theme, ui_font);
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

/// 创建“通用 / 键位”页签行；选中态由同步系统按 `KeybindsUiState` 刷新。
fn spawn_settings_tab_bar(parent: &mut ChildSpawnerCommands, theme: &UiTheme, ui_font: &UiFont) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|bar| {
            for (tab, label) in [
                (SettingsTab::General, "通用"),
                (SettingsTab::Keybinds, "键位"),
            ] {
                spawn_text_button(
                    bar,
                    SettingsTabButton { tab },
                    label,
                    UiControlKind::Tab,
                    theme,
                    ui_font,
                );
            }
        });
}

/// 通用设置页：现有六项设置值。
fn spawn_general_page(panel: &mut ChildSpawnerCommands, theme: &UiTheme, ui_font: &UiFont) {
    panel
        .spawn((
            SettingsGeneralPage,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
        ))
        .with_children(|page| {
            spawn_setting_row(
                page,
                "画面 / 渲染距离",
                SettingValue::RenderDistance,
                SettingAction::RenderDistance(-1),
                SettingAction::RenderDistance(1),
                theme,
                ui_font,
            );
            spawn_setting_row(
                page,
                "音频 / 主音量",
                SettingValue::MasterVolume,
                SettingAction::MasterVolume(-0.1),
                SettingAction::MasterVolume(0.1),
                theme,
                ui_font,
            );
            spawn_setting_row(
                page,
                "控制 / 鼠标灵敏度",
                SettingValue::MouseSensitivity,
                SettingAction::MouseSensitivity(-0.1),
                SettingAction::MouseSensitivity(0.1),
                theme,
                ui_font,
            );
            spawn_setting_row(
                page,
                "界面 / UI 缩放",
                SettingValue::UiScale,
                SettingAction::UiScale(-0.1),
                SettingAction::UiScale(0.1),
                theme,
                ui_font,
            );
            spawn_toggle_row(
                page,
                "画面 / 全屏",
                SettingValue::Fullscreen,
                SettingAction::ToggleFullscreen,
                theme,
                ui_font,
            );
            spawn_toggle_row(
                page,
                "画面 / 垂直同步",
                SettingValue::Vsync,
                SettingAction::ToggleVsync,
                theme,
                ui_font,
            );
        });
}

/// 键位设置页：搜索、过滤开关、重置按钮与可滚动的键位列表。
fn spawn_keybinds_page(panel: &mut ChildSpawnerCommands, theme: &UiTheme, ui_font: &UiFont) {
    panel
        .spawn((
            SettingsKeybindsPage,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                // 初始隐藏用 display None：兄弟页并列布局下 None 不占高度，
                // 与通用页的切换由 sync_settings_tabs_system 统一用 display 驱动。
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|page| {
            page.spawn((
                Text::new("点击键位后按下新按键：Esc 取消，Backspace 解除绑定"),
                body_font(ui_font, 12.0),
                TextColor(theme.text_hint),
            ));
            page.spawn((
                Text::new(
                    "滚轮切换快捷栏、鼠标视角与 Ctrl+F5 存档等调试组合为固定输入，不参与重映射",
                ),
                body_font(ui_font, 12.0),
                TextColor(theme.text_hint),
            ));

            page.spawn(Node {
                width: Val::Percent(100.0),
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|toolbar| {
                toolbar.spawn((
                    KeybindSearchInput,
                    EditableText {
                        visible_width: Some(16.0),
                        max_characters: Some(32),
                        allow_newlines: false,
                        ..default()
                    },
                    TextCursorStyle::default(),
                    TextLayout::no_wrap(),
                    body_font(ui_font, 14.0),
                    TextColor(theme.text_primary),
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(36.0),
                        padding: UiRect::horizontal(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(theme.search_bg),
                    BorderColor::all(theme.search_border),
                ));
                let conflict_button = spawn_text_button(
                    toolbar,
                    KeybindFilterButton {
                        filter: KeybindFilter::Conflicts,
                    },
                    "仅冲突",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                );
                toolbar
                    .commands()
                    .entity(conflict_button)
                    .insert(UiControl {
                        kind: UiControlKind::Button,
                        selected: false,
                        disabled: false,
                    });
                let unbound_button = spawn_text_button(
                    toolbar,
                    KeybindFilterButton {
                        filter: KeybindFilter::Unbound,
                    },
                    "仅未绑定",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                );
                toolbar.commands().entity(unbound_button).insert(UiControl {
                    kind: UiControlKind::Button,
                    selected: false,
                    disabled: false,
                });
                spawn_text_button(
                    toolbar,
                    KeybindResetButton,
                    "重置默认",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                );
            });

            spawn_scroll_area(
                page,
                KeybindList,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(330.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            );
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
            // 与步进行的「−」按钮列等宽的占位，保证数值列与上方各行纵向对齐。
            row.spawn(Node {
                width: Val::Px(56.0),
                height: Val::Px(40.0),
                ..default()
            });
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
                SettingButton(action),
                "切换",
                UiControlKind::Toggle,
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
