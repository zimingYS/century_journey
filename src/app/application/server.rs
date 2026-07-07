use crate::app::application::Application;
use crate::app::config::AppConfig;
use bevy::prelude::App;

/// 服务端应用（规划中）。
pub struct ServerApplication;

impl Application for ServerApplication {
    fn build(_config: AppConfig) -> anyhow::Result<App> {
        anyhow::bail!(
            "Server mode is planned but not implemented yet. Please run the client mode for now."
        );
    }
}
