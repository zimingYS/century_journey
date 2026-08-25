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
use crate::client::ui::localization::{localized_text, spawn_localized_button};
use crate::client::ui::resources::frame_assets::UiFrameKind;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::common::{
    UiControl, UiControlKind, spawn_scroll_area, spawn_text_button,
};
use crate::engine::localization::Localization;

/// 创建设置页和流程对话框；两者由应用流程资源控制可见性。
pub(super) fn spawn_settings_screens(
    commands: &mut Commands,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
    spawn_settings(commands, theme, ui_font, localization);
    spawn_dialog(commands, theme, ui_font, localization);
}

fn spawn_settings(
    commands: &mut Commands,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
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
                    localized_text("settings.title", localization),
                    title_font(ui_font, 28.0),
                    TextColor(theme.text_primary),
                ));
                spawn_settings_tab_bar(panel, theme, ui_font, localization);
                spawn_general_page(panel, theme, ui_font, localization);
                spawn_keybinds_page(panel, theme, ui_font, localization);
                spawn_localized_button(
                    panel,
                    SettingsBackButton,
                    "settings.back",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                    localization,
                );
            });
        });
}

/// 创建“通用 / 键位”页签行；选中态由同步系统按 `KeybindsUiState` 刷新。
fn spawn_settings_tab_bar(
    parent: &mut ChildSpawnerCommands,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|bar| {
            for (tab, key) in [
                (SettingsTab::General, "settings.tab-general"),
                (SettingsTab::Keybinds, "settings.tab-keybinds"),
            ] {
                spawn_localized_button(
                    bar,
                    SettingsTabButton { tab },
                    key,
                    UiControlKind::Tab,
                    theme,
                    ui_font,
                    localization,
                );
            }
        });
}

/// 通用设置页：渲染、音频、控制、界面与语言设置值。
fn spawn_general_page(
    panel: &mut ChildSpawnerCommands,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
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
            let ctx = SettingRowContext {
                theme,
                ui_font,
                localization,
            };
            spawn_setting_row(
                page,
                "settings.row.render-distance",
                SettingValue::RenderDistance,
                SettingAction::RenderDistance(-1),
                SettingAction::RenderDistance(1),
                &ctx,
            );
            spawn_setting_row(
                page,
                "settings.row.master-volume",
                SettingValue::MasterVolume,
                SettingAction::MasterVolume(-0.1),
                SettingAction::MasterVolume(0.1),
                &ctx,
            );
            spawn_setting_row(
                page,
                "settings.row.mouse-sensitivity",
                SettingValue::MouseSensitivity,
                SettingAction::MouseSensitivity(-0.1),
                SettingAction::MouseSensitivity(0.1),
                &ctx,
            );
            spawn_setting_row(
                page,
                "settings.row.ui-scale",
                SettingValue::UiScale,
                SettingAction::UiScale(-0.1),
                SettingAction::UiScale(0.1),
                &ctx,
            );
            spawn_setting_row(
                page,
                "settings.row.language",
                SettingValue::Language,
                SettingAction::CycleLanguage(-1),
                SettingAction::CycleLanguage(1),
                &ctx,
            );
            spawn_toggle_row(
                page,
                "settings.row.fullscreen",
                SettingValue::Fullscreen,
                SettingAction::ToggleFullscreen,
                &ctx,
            );
            spawn_toggle_row(
                page,
                "settings.row.vsync",
                SettingValue::Vsync,
                SettingAction::ToggleVsync,
                &ctx,
            );
        });
}

/// 键位设置页：搜索、过滤开关、重置按钮与可滚动的键位列表。
fn spawn_keybinds_page(
    panel: &mut ChildSpawnerCommands,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
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
                localized_text("settings.keybinds.rebind-hint", localization),
                body_font(ui_font, 12.0),
                TextColor(theme.text_hint),
            ));
            page.spawn((
                localized_text("settings.keybinds.fixed-hint", localization),
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
                let conflict_button = spawn_localized_button(
                    toolbar,
                    KeybindFilterButton {
                        filter: KeybindFilter::Conflicts,
                    },
                    "settings.keybinds.filter-conflicts",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                    localization,
                );
                toolbar
                    .commands()
                    .entity(conflict_button)
                    .insert(UiControl {
                        kind: UiControlKind::Button,
                        selected: false,
                        disabled: false,
                    });
                let unbound_button = spawn_localized_button(
                    toolbar,
                    KeybindFilterButton {
                        filter: KeybindFilter::Unbound,
                    },
                    "settings.keybinds.filter-unbound",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                    localization,
                );
                toolbar.commands().entity(unbound_button).insert(UiControl {
                    kind: UiControlKind::Button,
                    selected: false,
                    disabled: false,
                });
                spawn_localized_button(
                    toolbar,
                    KeybindResetButton,
                    "settings.keybinds.reset-defaults",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                    localization,
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

/// 设置行构建的公共依赖：主题、字体与本地化查询总是成组传递。
struct SettingRowContext<'a> {
    theme: &'a UiTheme,
    ui_font: &'a UiFont,
    localization: &'a Localization,
}

fn spawn_setting_row(
    parent: &mut ChildSpawnerCommands,
    label_key: &str,
    value: SettingValue,
    decrease: SettingAction,
    increase: SettingAction,
    ctx: &SettingRowContext<'_>,
) {
    let (theme, ui_font, localization) = (ctx.theme, ctx.ui_font, ctx.localization);
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
                localized_text(label_key, localization),
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
    label_key: &str,
    value: SettingValue,
    action: SettingAction,
    ctx: &SettingRowContext<'_>,
) {
    let (theme, ui_font, localization) = (ctx.theme, ctx.ui_font, ctx.localization);
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
                localized_text(label_key, localization),
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
            spawn_localized_button(
                row,
                SettingButton(action),
                "settings.toggle",
                UiControlKind::Toggle,
                theme,
                ui_font,
                localization,
            );
        });
}

fn spawn_dialog(
    commands: &mut Commands,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
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
                    Text::new(""),
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
                        spawn_localized_button(
                            actions,
                            DialogCancelButton,
                            "settings.cancel",
                            UiControlKind::Button,
                            theme,
                            ui_font,
                            localization,
                        );
                        spawn_localized_button(
                            actions,
                            DialogConfirmButton,
                            "settings.confirm",
                            UiControlKind::Button,
                            theme,
                            ui_font,
                            localization,
                        );
                    });
            });
        });
}
