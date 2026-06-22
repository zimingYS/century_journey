use crate::app::config::AppConfig;
use bevy::prelude::App;

pub trait Application {
    fn build(config: AppConfig) -> App;

    fn run(config: AppConfig) {
        let mut app = Self::build(config);
        app.run();
    }
}
