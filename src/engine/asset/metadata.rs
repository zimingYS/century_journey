//! 资源元数据模块。
//!
//! 重新导出 [`AssetMetadata`] 和 [`AssetState`]，
//! 供 Registry 和 Service 层使用。

pub use crate::engine::asset::state::{AssetMetadata, AssetState};
