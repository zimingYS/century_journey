//! # Engine Asset
//!
//! AssetId → AssetResolver → AssetLocation → AssetPipeline → Bevy AssetServer/AssetSource。
//!
//! - 需要 `Handle<T>` 的资源（纹理、字体……）用 [`AssetManager`]。
//! - 同步读配置/数据文件（JSON 定义等）用 [`AssetFiles`]，与 `AssetManager` 完全分离。
//! - 生命周期状态、引用计数、热重载、多来源覆盖全部复用 Bevy 自身的 Asset 系统，不重复实现。

pub mod cache;
pub mod files;
pub mod identifier;
pub mod location;
pub mod manager;
pub mod pipeline;
pub mod plugin;
pub mod resolver;
pub mod texture;

pub use cache::AssetCache;
pub use files::AssetFiles;
pub use identifier::{AssetId, asset_id};
pub use location::AssetLocation;
pub use manager::AssetManager;
pub use pipeline::AssetPipeline;
pub use plugin::AssetPlugin;
pub use resolver::AssetResolver;
pub use texture::{TextureAsset, TextureMetadata, TextureUsage};
