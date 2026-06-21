use bevy::prelude::App;

use crate::app::application::Application;
use crate::app::config::AppConfig;

/// 编辑器应用（暂未实现）。
pub struct EditorApplication;

impl Application for EditorApplication {
    fn build(_config: AppConfig) -> App {
        unimplemented!("Editor mode is not yet implemented");
    }
}
