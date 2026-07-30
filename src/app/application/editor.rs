//! 声明规划中的内容编辑器运行模式入口。

use crate::app::application::Application;
use crate::app::config::AppConfig;
use bevy::prelude::App;

/// 编辑器应用（规划中）。
pub struct EditorApplication;

impl Application for EditorApplication {
    fn build(_config: AppConfig) -> anyhow::Result<App> {
        anyhow::bail!("编辑器模式尚未实现，请暂时使用客户端模式启动。");
    }
}
