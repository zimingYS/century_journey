use bevy::prelude::App;
use crate::app::config::AppConfig;

pub trait Application {
    fn build(config: AppConfig) -> App;

    fn run(config: AppConfig) {
        let mut app = Self::build(config);
        app.run();
    }
}
