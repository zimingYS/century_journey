//! 控制台指令系统：解析玩家提交的指令行并修改权威玩法状态。

pub mod components;
pub mod execute;
pub mod parse;
pub mod plugin;
pub mod suggest;

pub use plugin::CommandPlugin;
