//! Century Journey 主库。
//!
//! 当前可运行链路由 App 装配 Client、Content、Game、Shared 与 Engine。
//! Editor、Protocol 和 Server 仅保留顶层规划边界，不能视为已经实现。

#![allow(non_snake_case)]
#![allow(
    clippy::module_inception,
    clippy::needless_range_loop,
    clippy::too_many_arguments,
    clippy::type_complexity
)]

pub mod app;
pub mod client;
pub mod content;
pub mod editor;
pub mod engine;
pub mod game;
pub mod protocol;
pub mod server;
pub mod shared;
