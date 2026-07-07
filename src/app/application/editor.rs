use crate::app::application::Application;
use crate::app::config::AppConfig;
use bevy::prelude::App;

/// 编辑器应用（规划中）。
pub struct EditorApplication;

impl Application for EditorApplication {
    fn build(_config: AppConfig) -> anyhow::Result<App> {
        anyhow::bail!(
            "Editor mode is planned but not implemented yet. Please run the client mode for now."
        );
    }
}
