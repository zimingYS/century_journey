//! 组装区块、物品和纹理图集等客户端渲染子系统。

pub(crate) mod constants;
pub(crate) mod distant;
pub mod item;
pub mod lighting;
mod plugin;
pub mod tex_atlas;
pub mod world;

pub use plugin::ClientRenderingPlugin;
