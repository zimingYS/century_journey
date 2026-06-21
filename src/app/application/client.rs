/// 客户端应用 — Composition Root。
///
/// 仅负责组装：DefaultPlugins + ClientPluginGroup。
/// 不直接初始化任何业务资源或注册业务 System。

use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowResolution};
use super::application::Application;
use crate::app::config::AppConfig;
use crate::client::plugin_group::ClientPluginGroup;
use crate::engine::constant::window::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};

pub struct ClientApplication;

impl Application for ClientApplication {
    fn build(_config: AppConfig) -> App {
        let mut app = App::new();
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                title: WINDOW_TITLE.to_string(),
                name: None,
                resizable: true,
                ..default()
            }),
            ..default()
        }));
        app.add_plugins(ClientPluginGroup);
        app
    }
}
