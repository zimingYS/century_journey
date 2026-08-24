//! 菜单交互到应用流程命令的适配系统。

use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use super::components::{
    CreateButton, DeleteButton, DialogCancelButton, DialogConfirmButton, KeybindFilter,
    KeybindFilterButton, KeybindKeyButton, KeybindResetButton, MainSettingsButton,
    PauseSettingsButton, PlayButton, QuitButton, ResumeButton, SaveQuitButton, SettingButton,
    SettingsBackButton, SettingsTabButton, WorldEntryButton,
};
use super::resources::{KeybindsUiState, WorldNameDraft};
use crate::app::flow::{FlowCommand, MenuPage};
use crate::client::input::RebindCapture;

/// 把本帧按下的菜单控件转换为应用流程命令。
///
/// 本系统不直接创建世界、切换状态或修改设置，以保持 Client 到 App 的单向命令边界。
#[allow(clippy::type_complexity)]
pub(crate) fn menu_button_system(
    static_query: Query<
        (
            &Interaction,
            Option<&PlayButton>,
            Option<&CreateButton>,
            Option<&DeleteButton>,
            Option<&MainSettingsButton>,
            Option<&QuitButton>,
            Option<&ResumeButton>,
            Option<&PauseSettingsButton>,
            Option<&SaveQuitButton>,
            Option<&SettingsBackButton>,
        ),
        Changed<Interaction>,
    >,
    world_query: Query<(&Interaction, &WorldEntryButton), Changed<Interaction>>,
    setting_query: Query<(&Interaction, &SettingButton), Changed<Interaction>>,
    dialog_query: Query<
        (
            &Interaction,
            Option<&DialogConfirmButton>,
            Option<&DialogCancelButton>,
        ),
        Changed<Interaction>,
    >,
    draft: Res<WorldNameDraft>,
    mut writer: MessageWriter<FlowCommand>,
) {
    for (
        interaction,
        play,
        create,
        delete,
        main_settings,
        quit,
        resume,
        pause_settings,
        save_quit,
        back,
    ) in &static_query
    {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let command = if play.is_some() {
            FlowCommand::PlaySelected
        } else if create.is_some() {
            FlowCommand::CreateWorld(draft.0.clone())
        } else if delete.is_some() {
            FlowCommand::RequestDeleteSelected
        } else if main_settings.is_some() || pause_settings.is_some() {
            FlowCommand::OpenSettings
        } else if quit.is_some() {
            FlowCommand::QuitApplication
        } else if resume.is_some() {
            FlowCommand::Resume
        } else if save_quit.is_some() {
            FlowCommand::SaveAndQuit
        } else if back.is_some() {
            FlowCommand::CloseSettings
        } else {
            continue;
        };
        writer.write(command);
    }

    for (interaction, entry) in &world_query {
        if *interaction == Interaction::Pressed {
            writer.write(FlowCommand::SelectWorld(entry.id.clone()));
        }
    }

    for (interaction, button) in &setting_query {
        if *interaction == Interaction::Pressed {
            writer.write(FlowCommand::AdjustSetting(button.0));
        }
    }

    for (interaction, confirm, cancel) in &dialog_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if confirm.is_some() {
            writer.write(FlowCommand::ConfirmDialog);
        } else if cancel.is_some() {
            writer.write(FlowCommand::CancelDialog);
        }
    }
}

/// 处理设置页键位界面的本地交互：页签、过滤、进入重绑定与恢复默认。
///
/// 页签和过滤只改变 Client 本地界面状态；重置默认通过 FlowCommand
/// 走应用层修改 `Keybinds`，保持与设置值相同的单向命令边界。
#[allow(clippy::too_many_arguments)]
pub(crate) fn keybind_ui_system(
    tab_query: Query<(&Interaction, &SettingsTabButton), Changed<Interaction>>,
    key_button_query: Query<(&Interaction, &KeybindKeyButton), Changed<Interaction>>,
    filter_query: Query<(&Interaction, &KeybindFilterButton), Changed<Interaction>>,
    reset_query: Query<&Interaction, (Changed<Interaction>, With<KeybindResetButton>)>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<SettingsBackButton>)>,
    mut ui_state: ResMut<KeybindsUiState>,
    mut capture: ResMut<RebindCapture>,
    mut focus: ResMut<InputFocus>,
    mut writer: MessageWriter<FlowCommand>,
) {
    for (interaction, button) in &tab_query {
        if *interaction == Interaction::Pressed {
            ui_state.tab = button.tab;
            if button.tab != super::components::SettingsTab::Keybinds {
                capture.listening = None;
                focus.clear();
            }
            ui_state.list_dirty = true;
        }
    }

    for (interaction, button) in &key_button_query {
        if *interaction == Interaction::Pressed {
            // 进入捕获前释放文本焦点，避免新按键被输入进搜索框。
            focus.clear();
            capture.listening = Some(button.action);
            ui_state.list_dirty = true;
        }
    }

    for (interaction, button) in &filter_query {
        if *interaction == Interaction::Pressed {
            match button.filter {
                KeybindFilter::Conflicts => {
                    ui_state.conflicts_only = !ui_state.conflicts_only;
                }
                KeybindFilter::Unbound => {
                    ui_state.unbound_only = !ui_state.unbound_only;
                }
            }
            ui_state.list_dirty = true;
        }
    }

    for interaction in &reset_query {
        if *interaction == Interaction::Pressed {
            writer.write(FlowCommand::ResetKeybinds);
            ui_state.list_dirty = true;
        }
    }

    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            capture.listening = None;
        }
    }
}

/// 设置页关闭时退出捕获状态并清空搜索，避免残留监听影响后续输入。
pub(crate) fn reset_keybind_listening_system(
    page: Res<MenuPage>,
    mut capture: ResMut<RebindCapture>,
    mut ui_state: ResMut<KeybindsUiState>,
) {
    if *page != MenuPage::Settings && capture.listening.is_some() {
        capture.listening = None;
    }
    if *page != MenuPage::Settings && !ui_state.search.is_empty() {
        ui_state.search.clear();
        ui_state.list_dirty = true;
    }
}
