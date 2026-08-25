//! 应用流程资源到菜单表现实体的单向同步系统。

use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::text::EditableText;

use super::components::{
    DialogCancelButton, DialogMessage, DialogRoot, DialogTitle, KeybindFilterButton,
    KeybindKeyButton, KeybindList, KeybindRow, LoadingDetail, LoadingTitle, SettingValue,
    SettingsGeneralPage, SettingsKeybindsPage, SettingsRoot, SettingsTabButton, WorldEntryButton,
    WorldList, WorldNameInput,
};
use super::resources::{KeybindsUiState, WorldNameDraft};
use super::style::body_font;
use crate::app::flow::{
    DialogKind, DialogState, GameSettings, LoadingStatus, MenuPage, WorldCatalog,
};
use crate::app::settings::{
    KEY_ACTIONS, KeyAction, Keybinds, action_label_localized, binding_display_localized,
};
use crate::client::input::RebindCapture;
use crate::client::ui::navigation::{UiScreen, UiScreenRoot, UiScreenStack};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::common::{UiControl, UiControlKind, spawn_text_button};
use crate::engine::localization::Localization;
use crate::shared::states::{AppState, InputContextState};

/// 在应用状态变化时重建顶层 UI 栈与输入上下文。
///
/// 只有状态真正变化时才清除焦点，避免每帧打断文本输入。
pub(crate) fn sync_flow_screen_stack_system(
    state: Res<State<AppState>>,
    mut previous: Local<Option<AppState>>,
    mut stack: ResMut<UiScreenStack>,
    mut context: ResMut<InputContextState>,
    mut menu_page: ResMut<MenuPage>,
    mut focus: ResMut<InputFocus>,
) {
    if previous.as_ref() == Some(state.get()) {
        return;
    }
    *previous = Some(state.get().clone());
    focus.clear();
    *menu_page = MenuPage::Worlds;
    match state.get() {
        AppState::Boot | AppState::Loading | AppState::WorldLoading => {
            stack.clear();
            stack.open(UiScreen::Loading);
            context.set_menu_open(true);
        }
        AppState::MainMenu => {
            stack.clear();
            stack.open(UiScreen::MainMenu);
            context.set_menu_open(true);
        }
        AppState::InGame => {
            stack.clear();
            context.set_menu_open(false);
        }
        AppState::Paused => {
            stack.close(UiScreen::Settings);
            stack.open(UiScreen::PauseMenu);
            context.set_menu_open(true);
        }
    }
}

/// 根据应用状态、菜单页和对话框状态同步各菜单根实体的可见性。
#[allow(clippy::type_complexity)]
pub(crate) fn sync_menu_visibility_system(
    state: Res<State<AppState>>,
    page: Res<MenuPage>,
    dialog: Res<DialogState>,
    mut settings_query: Query<&mut Visibility, (With<SettingsRoot>, Without<DialogRoot>)>,
    mut dialog_query: Query<&mut Visibility, (With<DialogRoot>, Without<SettingsRoot>)>,
    mut screen_query: Query<
        (&UiScreenRoot, &mut Visibility),
        (Without<SettingsRoot>, Without<DialogRoot>),
    >,
) {
    let settings_open =
        *page == MenuPage::Settings && matches!(state.get(), AppState::MainMenu | AppState::Paused);
    if let Ok(mut visibility) = settings_query.single_mut() {
        *visibility = if settings_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (root, mut visibility) in &mut screen_query {
        match root.screen {
            UiScreen::MainMenu if *state.get() == AppState::MainMenu => {
                *visibility = if settings_open {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
            UiScreen::PauseMenu if *state.get() == AppState::Paused => {
                *visibility = if settings_open {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
            _ => {}
        }
    }

    if let Ok(mut visibility) = dialog_query.single_mut() {
        *visibility = if dialog.kind.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// 在加载状态变化时刷新加载界面的标题和详情。
pub(crate) fn sync_loading_text_system(
    status: Res<LoadingStatus>,
    mut title_query: Query<&mut Text, (With<LoadingTitle>, Without<LoadingDetail>)>,
    mut detail_query: Query<&mut Text, (With<LoadingDetail>, Without<LoadingTitle>)>,
) {
    if !status.is_changed() {
        return;
    }
    if let Ok(mut text) = title_query.single_mut() {
        *text = Text::new(status.title.clone());
    }
    if let Ok(mut text) = detail_query.single_mut() {
        *text = Text::new(status.detail.clone());
    }
}

/// 刷新流程对话框文本，并按对话框语义决定是否显示取消按钮。
pub(crate) fn sync_dialog_text_system(
    dialog: Res<DialogState>,
    mut title_query: Query<&mut Text, (With<DialogTitle>, Without<DialogMessage>)>,
    mut message_query: Query<&mut Text, (With<DialogMessage>, Without<DialogTitle>)>,
    mut cancel_query: Query<&mut Visibility, With<DialogCancelButton>>,
) {
    if !dialog.is_changed() {
        return;
    }
    if let Ok(mut text) = title_query.single_mut() {
        *text = Text::new(dialog.title.clone());
    }
    if let Ok(mut text) = message_query.single_mut() {
        *text = Text::new(dialog.message.clone());
    }
    if let Ok(mut visibility) = cancel_query.single_mut() {
        *visibility = if dialog
            .kind
            .as_ref()
            .is_some_and(DialogKind::requires_confirmation)
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// 在世界目录或语言变化时重建世界列表，并保留当前选中条目的视觉状态。
pub(crate) fn populate_world_list_system(
    catalog: Res<WorldCatalog>,
    localization: Res<Localization>,
    list_query: Query<Entity, With<WorldList>>,
    children_query: Query<&Children>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
) {
    if !catalog.is_changed() && !localization.is_changed() {
        return;
    }
    let Ok(list) = list_query.single() else {
        return;
    };
    if let Ok(children) = children_query.get(list) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    commands.entity(list).with_children(|parent| {
        if catalog.worlds.is_empty() {
            parent.spawn((
                Text::new(localization.get("menu.world-empty")),
                body_font(&ui_font, 15.0),
                TextColor(theme.text_hint),
            ));
            return;
        }

        for world in &catalog.worlds {
            let label = localization.format(
                "menu.world-entry",
                &[("id", world.id.as_str()), ("seed", &world.seed.to_string())],
            );
            let entity = spawn_text_button(
                parent,
                WorldEntryButton {
                    id: world.id.clone(),
                },
                &label,
                UiControlKind::Tab,
                &theme,
                &ui_font,
            );
            parent.commands().entity(entity).insert(UiControl {
                kind: UiControlKind::Tab,
                selected: catalog.selected.as_deref() == Some(world.id.as_str()),
                disabled: false,
            });
        }
    });
}

/// 把可编辑文本组件中的最新内容同步到创建世界名称草稿。
pub(crate) fn sync_world_name_draft_system(
    query: Query<&EditableText, (With<WorldNameInput>, Changed<EditableText>)>,
    mut draft: ResMut<WorldNameDraft>,
) {
    let Ok(editable) = query.single() else {
        return;
    };
    draft.0 = editable.value().to_string();
}

/// 在设置资源或语言变化后刷新所有设置值文本。
pub(crate) fn sync_setting_values_system(
    settings: Res<GameSettings>,
    localization: Res<Localization>,
    mut query: Query<(&SettingValue, &mut Text)>,
) {
    if !settings.is_changed() && !localization.is_changed() {
        return;
    }
    for (value, mut text) in &mut query {
        *text = Text::new(match value {
            SettingValue::RenderDistance => settings.render_distance.to_string(),
            SettingValue::MasterVolume => format!("{:.0}%", settings.master_volume * 100.0),
            SettingValue::MouseSensitivity => format!("{:.1}x", settings.mouse_sensitivity),
            SettingValue::UiScale => format!("{:.1}x", settings.ui_scale),
            SettingValue::Language => {
                // 语言值始终用该语言自身书写的名称展示（业界惯例）。
                // 展示激活语言而非设置值：设置语言无效时运行时保持既有
                // 激活语言，界面实际使用的正是后者。
                localization
                    .native_name_of(localization.active())
                    .unwrap_or(localization.active().as_str())
                    .to_string()
            }
            SettingValue::Fullscreen => if settings.fullscreen {
                localization.get("common.on")
            } else {
                localization.get("common.off")
            }
            .to_string(),
            SettingValue::Vsync => if settings.vsync {
                localization.get("common.on")
            } else {
                localization.get("common.off")
            }
            .to_string(),
        });
    }
}

/// 同步设置页页签：页面可见性、页签选中态和过滤开关选中态。
///
/// 页面查询必须限定到两个页面容器；无界查询会把相机与整个主菜单
/// 一并隐藏，导致设置页之外的画面全部消失（黑屏）。
pub(crate) fn sync_settings_tabs_system(
    ui_state: Res<KeybindsUiState>,
    mut general_page: Query<&mut Node, (With<SettingsGeneralPage>, Without<SettingsKeybindsPage>)>,
    mut keybinds_page: Query<&mut Node, (With<SettingsKeybindsPage>, Without<SettingsGeneralPage>)>,
    mut tab_query: Query<(&SettingsTabButton, &mut UiControl), Without<KeybindFilterButton>>,
    mut filter_query: Query<(&KeybindFilterButton, &mut UiControl), Without<SettingsTabButton>>,
) {
    if !ui_state.is_changed() {
        return;
    }
    let on_keybinds = ui_state.tab == super::components::SettingsTab::Keybinds;
    // 两页为兄弟节点并列，用 display 切换而非 Visibility：
    // Visibility::Hidden 仍参与布局，隐藏页会占据高度，把当前页挤出可视区域。
    if let Ok(mut node) = general_page.single_mut() {
        node.display = if on_keybinds {
            Display::None
        } else {
            Display::Flex
        };
    }
    if let Ok(mut node) = keybinds_page.single_mut() {
        node.display = if on_keybinds {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (button, mut control) in &mut tab_query {
        control.selected = button.tab == ui_state.tab;
    }
    for (button, mut control) in &mut filter_query {
        control.selected = match button.filter {
            super::components::KeybindFilter::Conflicts => ui_state.conflicts_only,
            super::components::KeybindFilter::Unbound => ui_state.unbound_only,
        };
    }
}

/// 把键位搜索框内容同步到界面状态，并触发列表重建。
pub(crate) fn sync_keybinds_search_system(
    query: Query<
        &EditableText,
        (
            With<super::components::KeybindSearchInput>,
            Changed<EditableText>,
        ),
    >,
    mut ui_state: ResMut<KeybindsUiState>,
) {
    let Ok(editable) = query.single() else {
        return;
    };
    let text = editable.value().to_string();
    if text != ui_state.search {
        ui_state.search = text;
        ui_state.list_dirty = true;
    }
}

/// 在搜索、过滤、绑定或语言变化时重建键位列表。
#[allow(clippy::too_many_arguments)]
pub(crate) fn populate_keybind_list_system(
    keybinds: Res<Keybinds>,
    ui_state: Res<KeybindsUiState>,
    capture: Res<RebindCapture>,
    localization: Res<Localization>,
    list_query: Query<Entity, With<KeybindList>>,
    children_query: Query<&Children>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
) {
    let needs_rebuild = ui_state.list_dirty
        || keybinds.is_changed()
        || capture.is_changed()
        || localization.is_changed();
    if !needs_rebuild {
        return;
    }
    let Ok(list) = list_query.single() else {
        return;
    };
    if let Ok(children) = children_query.get(list) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    let visible = KEY_ACTIONS
        .iter()
        .filter(|spec| {
            let label = action_label_localized(spec.action, &localization);
            let key_label = binding_display_localized(keybinds.binding(spec.action), &localization);
            keybinds.matches_filter(
                spec.action,
                &label,
                &key_label,
                &ui_state.search,
                ui_state.conflicts_only,
                ui_state.unbound_only,
            )
        })
        .collect::<Vec<_>>();

    commands.entity(list).with_children(|parent| {
        if visible.is_empty() {
            parent.spawn((
                Text::new(localization.get("settings.keybinds.list-empty")),
                body_font(&ui_font, 14.0),
                TextColor(theme.text_hint),
            ));
            return;
        }
        for spec in visible {
            spawn_keybind_row(
                parent,
                spec.action,
                &keybinds,
                capture.listening,
                &theme,
                &ui_font,
                &localization,
            );
        }
    });
}

/// 生成一行键位条目：动作名、冲突提示与键位按钮。
#[allow(clippy::too_many_arguments)]
fn spawn_keybind_row(
    parent: &mut ChildSpawnerCommands,
    action: KeyAction,
    keybinds: &Keybinds,
    listening: Option<KeyAction>,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &Localization,
) {
    let partners = keybinds.conflict_partners(action);
    let listening_here = listening == Some(action);
    let key_label = if listening_here {
        localization.get("settings.keybinds.listening").to_string()
    } else {
        binding_display_localized(keybinds.binding(action), localization)
    };

    parent
        .spawn((
            KeybindRow,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(action_label_localized(action, localization)),
                body_font(ui_font, 14.0),
                TextColor(theme.text_secondary),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            if !partners.is_empty() {
                let names = partners
                    .iter()
                    .map(|partner| action_label_localized(*partner, localization))
                    .collect::<Vec<_>>()
                    .join("、");
                let hint = localization.format("settings.keybinds.conflict", &[("keys", &names)]);
                row.spawn((
                    Text::new(hint),
                    body_font(ui_font, 12.0),
                    TextColor(theme.warning),
                    Node {
                        width: Val::Px(220.0),
                        ..default()
                    },
                ));
            }
            let entity = spawn_text_button(
                row,
                KeybindKeyButton { action },
                &key_label,
                UiControlKind::Button,
                theme,
                ui_font,
            );
            row.commands().entity(entity).insert(UiControl {
                kind: UiControlKind::Button,
                selected: listening_here,
                disabled: false,
            });
        });
}
