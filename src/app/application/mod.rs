//! Application 子模块 — 程序入口与生命周期。
//!
//! 负责 Application Trait、Launcher、AppMode 及各运行模式实现。

pub mod application;
pub mod client;
pub mod editor;
pub mod launcher;
pub mod mode;
pub mod server;

pub use self::application::Application;
