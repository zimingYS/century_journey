use crate::app::config::AppConfig;
use bevy::prelude::App;

pub trait Application {
    fn build(config: AppConfig) -> anyhow::Result<App>;

    fn run(config: AppConfig) -> anyhow::Result<()> {
        let mut app = Self::build(config)?;
        app.run();
        Ok(())
    }
}
