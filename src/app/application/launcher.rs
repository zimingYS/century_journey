use super::application::Application;
use super::client::ClientApplication;
use super::editor::EditorApplication;
use super::mode::AppMode;
use super::server::ServerApplication;
use crate::app::config::AppConfig;

/// 程序入口。根据 AppConfig.mode 启动对应的 Application。
pub fn launch() -> anyhow::Result<()> {
    let config = AppConfig::default();
    match config.mode {
        AppMode::Client => {
            let _: () = ClientApplication::run(config);
            Ok(())
        }
        AppMode::Server => {
            let _: () = ServerApplication::run(config);
            Ok(())
        }
        AppMode::Editor => {
            let _: () = EditorApplication::run(config);
            Ok(())
        }
    }
}
