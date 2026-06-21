use super::application::Application;
use super::client::ClientApplication;
use crate::app::config::AppConfig;
use super::editor::EditorApplication;
use super::mode::AppMode;
use super::server::ServerApplication;

/// 程序入口。根据 AppConfig.mode 启动对应的 Application。
pub fn launch() {
    let config = AppConfig::default();
    match config.mode {
        AppMode::Client => ClientApplication::run(config),
        AppMode::Server => ServerApplication::run(config),
        AppMode::Editor => EditorApplication::run(config),
    }
}
