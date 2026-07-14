//! 应用装配层。
//!
//! 负责选择运行模式、组装插件、管理应用状态与主流程。具体玩法和客户端表现
//! 分别由 Game 与 Client 提供，本层不重复实现业务规则。

pub mod application;
pub mod config;
pub mod flow;
pub mod plugin;
pub mod settings;
pub mod state;

pub use application::launcher::launch;
