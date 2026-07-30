//! 菜单交互到应用流程命令的适配系统。

use bevy::prelude::*;

use super::components::{
    CreateButton, DeleteButton, DialogCancelButton, DialogConfirmButton, MainSettingsButton,
    PauseSettingsButton, PlayButton, QuitButton, ResumeButton, SaveQuitButton, SettingButton,
    SettingsBackButton, WorldEntryButton,
};
use super::resources::WorldNameDraft;
use crate::app::flow::FlowCommand;

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
