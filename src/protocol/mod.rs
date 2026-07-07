//! Planned network protocol boundary.
//!
//! This module is reserved for future client/server packet definitions,
//! serialization, versioning, compression, and replication contracts. It is a
//! planning boundary today; no multiplayer protocol should be treated as
//! implemented or stable yet.
//!
//! 规划中的网络协议边界。
//!
//! protocol 模块预留给未来的客户端/服务端数据包定义、序列化、版本管理、
//! 压缩和同步契约。当前它只是规划边界，多人协议尚未实现，也尚未稳定。

pub mod channel;
pub mod codec;
pub mod common;
pub mod compression;
pub mod encryption;
pub mod handshake;
pub mod message;
pub mod packet;
pub mod replication;
pub mod rpc;
pub mod serialize;
pub mod version;
