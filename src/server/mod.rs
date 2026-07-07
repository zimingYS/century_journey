//! Planned server layer.
//!
//! The server crate boundary is reserved for future authoritative simulation,
//! networking, replication, persistence, permissions, and dedicated-server
//! runtime work. It is not a usable server implementation yet.
//!
//! 规划中的服务端层。
//!
//! server 边界预留给未来的权威模拟、网络、状态同步、持久化、权限和
//! dedicated server 运行时。目前它还不是可用的服务端实现。

pub mod anti_cheat;
pub mod authority;
pub mod ban;
pub mod chunk;
pub mod command;
pub mod config;
pub mod console;
pub mod logging;
pub mod metrics;
pub mod network;
pub mod permission;
pub mod player;
pub mod plugin;
pub mod replication;
pub mod save;
pub mod scheduler;
pub mod session;
pub mod simulation;
pub mod watchdog;
pub mod whitelist;
