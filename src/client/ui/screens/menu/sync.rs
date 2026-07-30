//! 应用流程资源到菜单表现实体的单向同步系统。

use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::text::EditableText;

use super::components::{
    DialogCancelButton, DialogMessage, DialogRoot, DialogTitle, LoadingDetail, LoadingTitle,
    SettingValue, SettingsRoot, WorldEntryButton, WorldList, WorldNameInput,
};
use super::resources::WorldNameDraft;
use super::style::body_font;
use crate::app::flow::{
    DialogKind, DialogState, GameSettings, LoadingStatus, MenuPage, WorldCatalog,
};
use crate::client::ui::navigation::{UiScreen, UiScreenRoot, UiScreenStack};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::common::{UiControl, UiControlKind, spawn_text_button};
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

/// 在世界目录变化时重建世界列表，并保留当前选中条目的视觉状态。
pub(crate) fn populate_world_list_system(
    catalog: Res<WorldCatalog>,
    list_query: Query<Entity, With<WorldList>>,
    children_query: Query<&Children>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
) {
    if !catalog.is_changed() {
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
                Text::new("还没有世界"),
                body_font(&ui_font, 15.0),
                TextColor(theme.text_hint),
            ));
            return;
        }

        for world in &catalog.worlds {
            let label = format!("{}    种子 {}", world.id, world.seed);
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

/// 在设置资源变化后刷新所有设置值文本。
pub(crate) fn sync_setting_values_system(
    settings: Res<GameSettings>,
    mut query: Query<(&SettingValue, &mut Text)>,
) {
    if !settings.is_changed() {
        return;
    }
    for (value, mut text) in &mut query {
        *text = Text::new(match value {
            SettingValue::RenderDistance => settings.render_distance.to_string(),
            SettingValue::MasterVolume => format!("{:.0}%", settings.master_volume * 100.0),
            SettingValue::MouseSensitivity => format!("{:.1}x", settings.mouse_sensitivity),
            SettingValue::UiScale => format!("{:.1}x", settings.ui_scale),
            SettingValue::Fullscreen => if settings.fullscreen {
                "开启"
            } else {
                "关闭"
            }
            .to_string(),
            SettingValue::Vsync => if settings.vsync { "开启" } else { "关闭" }.to_string(),
        });
    }
}
