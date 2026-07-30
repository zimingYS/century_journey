//! 应用主流程的稳定入口。
//!
//! 内部按世界目录、菜单命令、世界会话和设置运行时拆分；对外继续从本模块
//! 暴露原有资源、消息与插件路径，避免上层表现代码依赖内部文件布局。

mod catalog;
mod commands;
mod contracts;
mod plugin;
mod settings_runtime;
mod world_session;

pub use crate::app::settings::GameSettings;
pub use contracts::{
    DialogKind, DialogState, FlowCommand, GameSession, LoadingStatus, MenuPage, PendingWorld,
    SettingAction, WorldCatalog, WorldSummary,
};
pub use plugin::GameFlowPlugin;
