//! 设置资源的调整、加载、应用与自动持久化系统。

use bevy::audio::{GlobalVolume, Volume};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, PrimaryWindow, WindowMode};

use super::contracts::{DialogKind, DialogState, SettingAction};
use crate::app::settings::{
    GameSettings, load_settings, save_settings, settings_backup_available, settings_file_exists,
};
use crate::client::ui::theme::scale::UiScaleSettings;
use crate::game::world::streaming::WorldStreamingConfig;

/// 记录最近成功写盘的设置，避免每帧重复保存。
#[derive(Resource, Debug, Default)]
pub(super) struct SettingsPersistenceState {
    pub(super) last_saved: GameSettings,
    pub(super) blocked: bool,
}

/// 应用一次设置调整，并把结果限制在运行时支持范围内。
pub(super) fn adjust_setting(settings: &mut GameSettings, action: SettingAction) {
    match action {
        SettingAction::RenderDistance(delta) => {
            settings.render_distance =
                (settings.render_distance as i32 + delta).clamp(2, 24) as u32;
        }
        SettingAction::MasterVolume(delta) => {
            settings.master_volume = (settings.master_volume + delta).clamp(0.0, 1.0);
        }
        SettingAction::MouseSensitivity(delta) => {
            settings.mouse_sensitivity = (settings.mouse_sensitivity + delta).clamp(0.2, 3.0);
        }
        SettingAction::UiScale(delta) => {
            settings.ui_scale = (settings.ui_scale + delta).clamp(0.6, 1.6);
        }
        SettingAction::ToggleFullscreen => settings.fullscreen = !settings.fullscreen,
        SettingAction::ToggleVsync => settings.vsync = !settings.vsync,
    }
}

/// 将设置资源同步到各层已经存在的运行时资源。
pub(super) fn apply_settings_system(
    settings: Res<GameSettings>,
    mut ui_scale: ResMut<UiScaleSettings>,
    mut streaming: ResMut<WorldStreamingConfig>,
    mut global_volume: ResMut<GlobalVolume>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !settings.is_changed() {
        return;
    }
    ui_scale.user_scale = settings.ui_scale;
    *streaming = WorldStreamingConfig::new(
        settings.render_distance,
        settings.render_distance,
        streaming.data_vertical_radius_above as u32,
        streaming.data_vertical_radius_below as u32,
    );
    global_volume.volume = Volume::Linear(settings.master_volume);
    if let Ok(mut window) = window_query.single_mut() {
        window.mode = if settings.fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        } else {
            WindowMode::Windowed
        };
        window.present_mode = if settings.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
    }
}

/// 启动时加载设置；损坏或缺失且存在备份时暂停自动写盘并请求玩家确认。
pub(super) fn load_settings_system(
    mut settings: ResMut<GameSettings>,
    mut persistence_state: ResMut<SettingsPersistenceState>,
    mut dialog: ResMut<DialogState>,
) {
    if !settings_file_exists() {
        if settings_backup_available() {
            persistence_state.blocked = true;
            dialog.kind = Some(DialogKind::ConfirmRecoverSettings);
            dialog.title = "发现设置备份".into();
            dialog.message = "主设置文件缺失，是否恢复最近一次有效备份？".into();
            return;
        }
        if let Err(error) = save_settings(&settings) {
            persistence_state.blocked = true;
            dialog.error("设置保存失败", error);
        }
        persistence_state.last_saved = settings.clone();
        return;
    }

    match load_settings() {
        Ok(loaded) => {
            *settings = loaded.clone();
            persistence_state.last_saved = loaded;
        }
        Err(error) if settings_backup_available() => {
            persistence_state.blocked = true;
            dialog.kind = Some(DialogKind::ConfirmRecoverSettings);
            dialog.title = "设置文件损坏".into();
            dialog.message = format!("当前设置无法读取：{error}\n是否恢复最近一次有效备份？");
        }
        Err(error) => {
            persistence_state.blocked = true;
            dialog.error("设置加载失败", error);
        }
    }
}

/// 设置变更后自动写盘；失败后停止重试，等待对话框流程处理。
pub(super) fn persist_settings_system(
    settings: Res<GameSettings>,
    mut persistence_state: ResMut<SettingsPersistenceState>,
    mut dialog: ResMut<DialogState>,
) {
    if persistence_state.blocked || *settings == persistence_state.last_saved {
        return;
    }
    match save_settings(&settings) {
        Ok(()) => persistence_state.last_saved = settings.clone(),
        Err(error) => {
            persistence_state.blocked = true;
            dialog.error("设置保存失败", error);
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app/flow/settings_runtime.rs"]
mod tests;
