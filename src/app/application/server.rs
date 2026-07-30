//! 声明规划中的独立服务端运行模式入口。

use crate::app::application::Application;
use crate::app::config::AppConfig;
use bevy::prelude::App;

/// 服务端应用（规划中）。
pub struct ServerApplication;

impl Application for ServerApplication {
    fn build(_config: AppConfig) -> anyhow::Result<App> {
        anyhow::bail!("服务端模式尚未实现，请暂时使用客户端模式启动。");
    }
}
