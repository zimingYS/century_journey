//! # Engine Asset
//!
//! 统一资源管道（Asset Pipeline）。
//!
//! 负责所有游戏资源的标识、加载、缓存与生命周期管理。
//!
//! ## 架构
//!
//! ```text
//! AssetId → AssetManager → PathResolver → Source → Loader → Cache → Registry
//! ```
//!
//! 业务代码只能通过 [`AssetManager`] 加载资源，
//! 禁止直接调用 `AssetServer::load()`。
//!
//! ## 公开 API
//!
//! | 方法 | 说明 |
//! |------|------|
//! | `AssetManager::texture(id, server, cache, reg)` | 加载纹理 |
//! | `AssetManager::json(id, server, cache, reg)` | 加载 JSON |
//! | `AssetManager::invalidate(id, cache, reg)` | 清除缓存 |

pub mod identifier;
pub mod path;
pub mod event;
pub mod source;
pub mod loader;
pub mod registry;
pub mod cache;
pub mod manager;
pub mod plugin;

pub use manager::AssetManager;
pub use cache::AssetCache;
pub use registry::AssetRegistry;
pub use identifier::AssetId;
pub use plugin::AssetPlugin;
