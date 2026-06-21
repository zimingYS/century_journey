use bevy::prelude::App;

use crate::app::application::Application;
use crate::app::config::AppConfig;

/// 服务端应用（暂未实现）。
pub struct ServerApplication;

impl Application for ServerApplication {
    fn build(_config: AppConfig) -> App {
        unimplemented!("Server mode is not yet implemented");
    }
}
