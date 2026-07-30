//! 应用实例子模块：程序入口、运行模式与生命周期。
//!
//! 负责 Application Trait、Launcher、AppMode 及各运行模式实现。

pub mod client;
mod constants;
pub mod contract;
pub mod editor;
pub mod launcher;
pub mod mode;
pub mod server;

pub use self::contract::Application;
