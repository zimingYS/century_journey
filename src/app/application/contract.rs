//! 定义各运行模式共同遵守的应用构建与启动契约。

use crate::app::config::AppConfig;
use bevy::prelude::App;

/// 可由启动器选择并构建的应用运行模式。
pub trait Application {
    fn build(config: AppConfig) -> anyhow::Result<App>;

    fn run(config: AppConfig) -> anyhow::Result<()> {
        let mut app = Self::build(config)?;
        app.run();
        Ok(())
    }
}
