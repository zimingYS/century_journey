//! 应用流程各资源与调度阶段的统一注册入口。

use bevy::prelude::*;

use super::catalog::refresh_world_catalog_system;
use super::commands::handle_flow_commands_system;
use super::contracts::{
    DialogState, FlowCommand, GameSession, LoadingStatus, MenuPage, PendingWorld,
    SaveAndQuitRequest, WorldCatalog,
};
use super::settings_runtime::{
    KeybindsPersistenceState, SettingsPersistenceState, apply_settings_system,
    load_keybinds_system, load_settings_system, persist_keybinds_system, persist_settings_system,
};
use super::world_session::{
    enter_boot_system, finish_fresh_session_system, log_enter_world_system, log_exit_world_system,
    pause_virtual_time_system, prepare_world_system, request_content_reload_system,
    resume_virtual_time_system, save_and_quit_system, show_content_errors_system,
    sync_pause_state_system,
};
use crate::app::settings::{GameSettings, Keybinds};
use crate::content::lifecycle::ContentReloadSet::{Consumers, Request};
use crate::shared::states::AppState;

/// App 层的菜单、世界会话与设置运行时插件。
pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSession>()
            .init_resource::<WorldCatalog>()
            .init_resource::<PendingWorld>()
            .init_resource::<LoadingStatus>()
            .init_resource::<DialogState>()
            .init_resource::<MenuPage>()
            .init_resource::<GameSettings>()
            .init_resource::<SettingsPersistenceState>()
            .init_resource::<Keybinds>()
            .init_resource::<KeybindsPersistenceState>()
            .init_resource::<SaveAndQuitRequest>()
            .add_message::<FlowCommand>()
            .add_systems(Startup, (load_settings_system, load_keybinds_system))
            .add_systems(OnEnter(AppState::Boot), enter_boot_system)
            .add_systems(
                OnEnter(AppState::MainMenu),
                (refresh_world_catalog_system, show_content_errors_system).chain(),
            )
            .add_systems(OnEnter(AppState::WorldLoading), prepare_world_system)
            .add_systems(
                OnEnter(AppState::InGame),
                request_content_reload_system.in_set(Request),
            )
            .add_systems(
                OnEnter(AppState::InGame),
                log_enter_world_system.after(Consumers),
            )
            .add_systems(OnExit(AppState::InGame), log_exit_world_system)
            .add_systems(OnEnter(AppState::Paused), pause_virtual_time_system)
            .add_systems(OnExit(AppState::Paused), resume_virtual_time_system)
            .add_systems(
                Update,
                (
                    handle_flow_commands_system,
                    sync_pause_state_system,
                    save_and_quit_system,
                    apply_settings_system,
                    persist_settings_system,
                    persist_keybinds_system,
                    finish_fresh_session_system,
                )
                    .chain(),
            );
    }
}
