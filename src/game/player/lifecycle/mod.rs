//! 聚合玩家生命周期状态、消息、规则、生成和插件入口。

pub mod components;
pub mod events;
pub mod plugin;
pub mod rules;
pub mod spawn;

pub use components::{PlayerLifeState, PlayerLifecycle, RespawnPoint};
