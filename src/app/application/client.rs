use super::application::Application;
use crate::app::config::AppConfig;
use crate::client::plugin_group::ClientPluginGroup;
use crate::engine::constant::window::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use bevy::log::LogPlugin;
/// 客户端应用 — Composition Root。
///
/// 仅负责组装：DefaultPlugins + ClientPluginGroup。
/// 不直接初始化任何业务资源或注册业务 System。
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowResolution};

/// 默认日志过滤器
const DEFAULT_LOG_FILTER: &str = "info,wgpu_core=warn,wgpu_hal=warn,naga=warn";

pub struct ClientApplication;

impl Application for ClientApplication {
    fn build(_config: AppConfig) -> App {
        let mut app = App::new();
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                        title: WINDOW_TITLE.to_string(),
                        name: None,
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: format!("{DEFAULT_LOG_FILTER},icu_provider=error"),
                    ..default()
                }),
        );
        app.add_plugins(ClientPluginGroup);
        app
    }
}
