//! # Protocol
//!
//! 协议层（Protocol Layer）。
//!
//! `protocol` 定义 Client 与 Server 之间的通信协议（Communication Protocol），
//! 负责描述双方如何交换数据，而不负责数据如何产生或如何使用。
//!
//! Protocol 是 Client 与 Server 共同遵守的唯一通信契约（Contract），
//! 保证双方使用一致的数据格式、序列化方式和协议版本。
//!
//! ## 职责
//!
//! Protocol 负责定义所有网络通信相关内容，包括但不限于：
//!
//! - Packet
//! - Message
//! - Codec
//! - Serialization
//! - Deserialization
//! - Channel
//! - Protocol Version
//! - Packet Identifier
//!
//! Protocol 描述的是："数据如何传输（How Data Travels）"。
//!
//! ## 设计原则
//!
//! Protocol 只负责定义通信格式，不负责通信行为。
//!
//! Client 与 Server 应通过 Protocol 交换数据，
//! 而不是互相依赖对方的数据结构。
//!
//! 所有网络数据都应通过 Protocol 进行编码、解码和版本管理。
//!
//! ## 模块组织
//!
//! ```text
//! protocol/
//! ├── packet/          // 所有数据包定义
//! ├── message/         // 通用消息结构
//! ├── codec/           // 编解码器
//! ├── serialize/       // 序列化与反序列化
//! ├── channel/         // 通信通道定义
//! ├── version/         // 协议版本
//! └── util/            // 协议工具
//! ```
//!
//! ## 架构位置
//!
//! ```text
//!                App
//!              /     \
//!         Client     Server
//!             │         │
//!             └────┬────┘
//!                  │
//!              Protocol
//!                  │
//!                Game
//!               /    \
//!         Content   Shared
//!               \    /
//!               Engine
//! ```
//!
//! Protocol 位于 Client 与 Server 之间，
//! 是双方唯一共享的网络通信定义。
//!
//! Client 与 Server 可以依赖 Protocol，
//! 但 Protocol 不应依赖 Client 或 Server。
//!
//! ## 核心设计理念
//!
//! Protocol 提供的是 **Communication Contract（通信契约）**，
//! 而不是 **Game Rules（游戏规则）**。
//!
//! 例如：
//!
//! - Protocol 定义 `PlayerMovePacket`，而不是玩家移动。
//! - Protocol 定义 `BreakBlockPacket`，而不是破坏方块。
//! - Protocol 定义 `InventorySyncPacket`，而不是背包逻辑。
//! - Protocol 定义 `ChunkDataPacket`，而不是 Chunk 加载。
//!
//! Packet 只是数据载体，
//! 真正的业务处理由 Client 或 Server 调用 Game 完成。
//!
//! ## 数据流
//!
//! ```text
//! Client
//!     │
//!     ▼
//! PlayerMovePacket
//!     │
//!     ▼
//! Protocol
//!     │
//!     ▼
//! Network
//!     │
//!     ▼
//! Server
//!     │
//!     ▼
//! Game
//! ```
//!
//! 或者：
//!
//! ```text
//! Game
//!     │
//!     ▼
//! World Changed
//!     │
//!     ▼
//! Protocol
//!     │
//!     ▼
//! ChunkDataPacket
//!     │
//!     ▼
//! Client
//! ```
//!
//! Protocol 只负责数据交换，
//! 不负责世界模拟。
//!
//! ## 协议设计原则
//!
//! - Packet 应尽可能保持简单。
//! - Packet 只传输必要的数据。
//! - Packet 不包含业务逻辑。
//! - Packet 应支持协议版本兼容。
//! - Packet 应尽量与网络实现解耦。
//!
//! 网络层负责发送 Packet，
//! Game 负责解释 Packet，
//! Protocol 只负责定义 Packet。
//!
//! ## 判断标准
//!
//! 当一个模块回答的是：
//!
//! "这份数据如何从 Client 传递到 Server（或反之）？"
//!
//! 那么它属于 Protocol。
//!
//! 如果回答的是：
//!
//! - 如何连接服务器？→ Server / Client
//! - 如何发送数据？→ Client / Server
//! - 如何处理数据？→ Game
//! - 数据应该长什么样？→ Protocol
//!
//! 那么它就不属于 Protocol。
//!
//! ## 核心原则
//!
//! Protocol 是 Client 与 Server 唯一共享的通信语言。
//!
//! 它只定义"说什么（What to Say）"，
//! 不定义"为什么说（Why）"或"收到之后做什么（How to Handle）"。
//!
//! Client 与 Server 可以独立演进，
//! 只要双方遵循同一份 Protocol，即可保持兼容。

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
