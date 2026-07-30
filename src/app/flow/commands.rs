//! 菜单命令分发与确认对话框操作。

use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

use super::catalog::{refresh_world_catalog, sanitize_world_name, unique_world_id, valid_world_id};
use super::contracts::{
    DialogKind, DialogState, FlowCommand, MenuPage, PendingWorld, SaveAndQuitRequest, WorldCatalog,
};
use super::settings_runtime::{SettingsPersistenceState, adjust_setting};
use crate::app::settings::{GameSettings, load_settings, restore_settings_backup};
use crate::content::block::registry::BlockRegistry;
use crate::content::validation::ContentCompilation;
use crate::game::save::player::{player_save_path, restore_player_backup};
use crate::game::save::world::chunk::region::RegionManager;
use crate::game::save::world::metadata::io;
use crate::game::world::generation::pipeline::CURRENT_GENERATION_VERSION;
use crate::game::world::time::WorldSimulationClock;
use crate::shared::states::{AppState, InputContextState};

/// 顺序消费菜单命令，并把跨帧操作转换为应用状态或待处理请求。
///
/// 参数较多来自同一应用流程的资源边界；拆分后仍保持原有单系统顺序，避免改变
/// 同一帧多条菜单命令的处理语义。
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_flow_commands_system(
    mut reader: MessageReader<FlowCommand>,
    mut catalog: ResMut<WorldCatalog>,
    mut pending: ResMut<PendingWorld>,
    mut dialog: ResMut<DialogState>,
    mut menu_page: ResMut<MenuPage>,
    mut settings: ResMut<GameSettings>,
    mut settings_persistence: ResMut<SettingsPersistenceState>,
    compilation: Option<Res<ContentCompilation>>,
    block_registry: Option<Res<BlockRegistry>>,
    mut save_quit: ResMut<SaveAndQuitRequest>,
    mut next_state: ResMut<NextState<AppState>>,
    mut context: ResMut<InputContextState>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for command in reader.read() {
        match command {
            FlowCommand::RefreshWorlds => refresh_world_catalog(&mut catalog),
            FlowCommand::SelectWorld(id) => catalog.selected = Some(id.clone()),
            FlowCommand::CreateWorld(name) => {
                let Some(registry) = block_registry.as_deref() else {
                    dialog.error("创建失败", "方块注册表尚未加载完成");
                    continue;
                };
                let id = unique_world_id(&sanitize_world_name(name), &catalog);
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                match io::save_level(
                    &id,
                    seed,
                    CURRENT_GENERATION_VERSION,
                    &WorldSimulationClock::default(),
                    Vec3::new(0.0, 70.0, 0.0),
                    registry,
                ) {
                    Ok(()) => {
                        refresh_world_catalog(&mut catalog);
                        catalog.selected = Some(id);
                    }
                    Err(error) => dialog.error("创建失败", error.to_string()),
                }
            }
            FlowCommand::PlaySelected => {
                if let Some(compilation) = compilation
                    .as_deref()
                    .filter(|compilation| !compilation.is_valid())
                {
                    dialog.error(
                        "无法进入世界",
                        format!("内容编译失败：\n{}", compilation.error_summary(12)),
                    );
                    continue;
                }
                if let Some(selected) = catalog.selected.clone() {
                    pending.0 = Some(selected);
                    next_state.set(AppState::WorldLoading);
                } else {
                    dialog.error("无法进入世界", "请先创建或选择一个世界");
                }
            }
            FlowCommand::RequestDeleteSelected => {
                if let Some(world_id) = catalog.selected.clone() {
                    dialog.kind = Some(DialogKind::ConfirmDelete {
                        world_id: world_id.clone(),
                    });
                    dialog.title = "删除世界".into();
                    dialog.message = format!("确定永久删除世界“{world_id}”吗？此操作无法撤销。");
                }
            }
            FlowCommand::ConfirmDialog => {
                if let Some(DialogKind::ConfirmRecoverWorld { world_id }) = dialog.kind.clone() {
                    if let Err(error) = io::restore_level_backup(&world_id) {
                        dialog.error("世界恢复失败", error.to_string());
                        continue;
                    }
                    pending.0 = Some(world_id);
                    next_state.set(AppState::WorldLoading);
                }
                if let Some(DialogKind::ConfirmRecoverPlayer { world_id }) = dialog.kind.clone() {
                    if let Err(error) = restore_player_backup(&player_save_path(&world_id)) {
                        dialog.error("玩家存档恢复失败", error);
                        continue;
                    }
                    pending.0 = Some(world_id);
                    next_state.set(AppState::WorldLoading);
                }
                if matches!(dialog.kind, Some(DialogKind::ConfirmRecoverSettings)) {
                    let result = restore_settings_backup().and_then(|()| load_settings());
                    match result {
                        Ok(restored) => {
                            *settings = restored.clone();
                            settings_persistence.last_saved = restored;
                            settings_persistence.blocked = false;
                        }
                        Err(error) => {
                            dialog.error("设置恢复失败", error);
                            continue;
                        }
                    }
                }
                if let Some(DialogKind::ConfirmDelete { world_id }) = dialog.kind.clone()
                    && valid_world_id(&world_id)
                {
                    match std::fs::remove_dir_all(RegionManager::save_root(&world_id)) {
                        Ok(()) => refresh_world_catalog(&mut catalog),
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                            refresh_world_catalog(&mut catalog);
                        }
                        Err(error) => {
                            dialog.error("删除失败", error.to_string());
                            continue;
                        }
                    }
                }
                dialog.clear();
            }
            FlowCommand::CancelDialog => dialog.clear(),
            FlowCommand::OpenSettings => *menu_page = MenuPage::Settings,
            FlowCommand::CloseSettings => *menu_page = MenuPage::Worlds,
            FlowCommand::Resume => {
                context.set_menu_open(false);
                next_state.set(AppState::InGame);
            }
            FlowCommand::SaveAndQuit => save_quit.0 = true,
            FlowCommand::QuitApplication => {
                app_exit.write(AppExit::Success);
            }
            FlowCommand::AdjustSetting(action) => adjust_setting(&mut settings, *action),
        }
    }
}
