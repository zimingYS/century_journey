//! 组织多层动态云的实体创建与每帧表现更新。
//!
//! 当前实现：raymarching 体积云（着色器云），通过天空球体 + 扩展材质渲染，
//! 云形由 `cloud_volume.wgsl` 的 DDA 体素 raymarching 决定。

mod components;
mod constants;
mod material;
mod plugin;
mod systems;
mod texture;
mod weather_adapter;

pub use plugin::CloudPlugin;

/// 供单元测试与天气适配器使用的类型入口。
#[doc(hidden)]
pub use components::{CloudLayer, CloudPatch, CloudWeatherState};
#[doc(hidden)]
pub use constants::CLOUD_TEXTURE_SIZE;
#[doc(hidden)]
pub use texture::generate_cloud_texture;
